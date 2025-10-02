use clap::Parser;
use std::num::ParseIntError;

#[derive(Debug, Clone)]
pub struct KeyPress {
    pub keycode: u32,
    pub pressed: u32,
}

#[derive(Parser, Debug)]
#[command(name = "whydotool")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,

    /// Force input injection via xdg-desktop-portal even if native Wayland virtual devices are available.
    #[arg(
        short = 'f',
        long,
        env = "WHYDOTOOL_FORCE_PORTAL",
        default_value_t = false
    )]
    pub force_portal: bool,
}

#[derive(Parser, Debug)]
pub enum Commands {
    Click {
        /// Buttons to click (hex values like 0xC0 for left click)
        #[arg(num_args = 1..)]
        buttons: Vec<String>,

        /// Repeat the sequence N times
        #[arg(short = 'r', long = "repeat", default_value_t = 1)]
        repeat: u32,

        /// Delay between input events in ms
        #[arg(short = 'D', long = "next-delay")]
        next_delay: Option<u64>,
    },
    Mousemove {
        /// Move mouse wheel relatively
        #[arg(short = 'w', long = "wheel")]
        wheel: bool,

        /// Use absolute position, not applicable to wheel.
        /// You need to disable mouse speed acceleration for correct absolute movement.
        #[arg(short = 'a', long = "absolute")]
        absolute: bool,

        /// X position
        #[arg(short = 'x', long = "xpos", allow_hyphen_values = true)]
        xpos: f64,

        /// Y position
        #[arg(short = 'y', long = "ypos", allow_hyphen_values = true)]
        ypos: f64,
    },
    Type {
        /// Delay N ms between key down/up
        #[arg(short = 'd', long = "key-delay", default_value_t = 20)]
        key_delay: u64,

        /// Hold each key for N ms
        #[arg(short = 'H', long = "key-hold", default_value_t = 20)]
        key_hold: u64,

        /// Delay N ms between command line strings
        #[arg(short = 'D', long = "next-delay")]
        next_delay: Option<u64>,

        /// Input file (or "-" for stdin)
        #[arg(short = 'f', long = "file")]
        file: Option<String>,

        /// Escape enable (1) or disable (0)
        #[arg(short = 'e', long = "escape")]
        escape: Option<u8>,

        /// Strings to type
        #[arg(num_args = 0..)]
        strings: Vec<String>,
    },
    Key {
        #[arg(value_delimiter = ' ', num_args = 1.., value_parser = parse_keypress)]
        key_presses: Vec<KeyPress>,

        #[arg(short = 'd', long = "key-delay")]
        key_delay: Option<u64>,
    },
}

fn parse_keypress(s: &str) -> Result<KeyPress, String> {
    let mut parts = s.split(':');

    let keycode_str = parts.next().ok_or("Missing keycode")?;
    let keycode: u32 = keycode_str
        .parse()
        .map_err(|_: ParseIntError| format!("Invalid keycode '{}'", keycode_str))?;

    let pressed_str = parts.next().ok_or("Missing pressed state")?;
    let pressed = match pressed_str {
        "0" => 0,
        "1" => 1,
        _ => {
            return Err(format!(
                "Pressed state must be 0 or 1, got '{}'",
                pressed_str
            ));
        }
    };

    Ok(KeyPress { keycode, pressed })
}
