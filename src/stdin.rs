use nix::libc::STDIN_FILENO;
use nix::sys::termios::{LocalFlags, SetArg, Termios, tcgetattr, tcsetattr};
use nix::unistd::isatty;
use std::{fs, os::fd::FromRawFd};

pub struct Terminal {
    old_tio: Termios,
    stdin_fileno: fs::File,
}

impl Terminal {
    pub fn configure() -> anyhow::Result<Self> {
        let stdin_fileno = unsafe { fs::File::from_raw_fd(STDIN_FILENO) };
        if !isatty(&stdin_fileno).is_ok_and(|isatty| isatty) {
            return Err(anyhow::anyhow!("Not a terminal"));
        }

        let old_tio = tcgetattr(&stdin_fileno)?;

        let mut new_tio = old_tio.clone();
        new_tio
            .local_flags
            .remove(LocalFlags::ICANON | LocalFlags::ECHO);
        tcsetattr(&stdin_fileno, SetArg::TCSANOW, &new_tio)?;

        Ok(Self {
            old_tio,
            stdin_fileno,
        })
    }

    pub fn set_ctrlc_handler(&self) -> anyhow::Result<()> {
        let old_tio = self.old_tio.clone();
        let stdin_fileno = self.stdin_fileno.try_clone()?;

        ctrlc::set_handler(move || {
            _ = tcsetattr(&stdin_fileno, SetArg::TCSANOW, &old_tio);
            std::process::exit(0);
        })?;

        Ok(())
    }

    pub fn restore(&self) -> anyhow::Result<()> {
        tcsetattr(&self.stdin_fileno, SetArg::TCSANOW, &self.old_tio)?;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
