use nix::libc::STDIN_FILENO;
use nix::sys::termios::{LocalFlags, SetArg, Termios, tcgetattr, tcsetattr};
use nix::unistd::isatty;
use std::{fs, os::fd::FromRawFd};

pub struct Terminal {
    _old_tio: Termios,
    _stdin_fileno: fs::File,
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

        let old_tio_clone = old_tio.clone();
        {
            let stdin_fileno = stdin_fileno.try_clone()?;
            ctrlc::set_handler(move || {
                _ = tcsetattr(&stdin_fileno, SetArg::TCSANOW, &old_tio_clone);
                std::process::exit(0);
            })
        }?;

        Ok(Self {
            _old_tio: old_tio,
            _stdin_fileno: stdin_fileno,
        })
    }
}
