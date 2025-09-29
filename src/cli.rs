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
}

#[derive(Parser, Debug)]
pub enum Commands {
    Click {},
    Mousemove {
        /// Move mouse wheel relatively
        #[arg(short = 'w', long = "wheel")]
        wheel: bool,

        /// Use absolute position, not applicable to wheel.
        /// You need to disable mouse speed acceleration for correct absolute movement.
        #[arg(short = 'a', long = "absolute")]
        absolute: bool,

        /// X position
        #[arg(short = 'x', long = "xpos")]
        xpos: u32,

        /// Y position
        #[arg(short = 'y', long = "ypos")]
        ypos: u32,
    },
    Type {},
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
