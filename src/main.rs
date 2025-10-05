mod cli;

use crate::cli::Commands;
use clap::Parser;
use cli::Cli;
use libwhydotool::Whydotool;
use std::{fs, io, time::Duration};
use wayland_client::protocol::wl_pointer::ButtonState;

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;

    let (mut event_queue, mut whydotool) = Whydotool::try_new()?;

    event_queue.dispatch_pending(&mut whydotool)?;
    event_queue.roundtrip(&mut whydotool)?;

    match cli.cmd {
        Commands::Click {
            repeat,
            next_delay,
            buttons,
        } => {
            let virtual_pointer = whydotool.virtual_pointer().map_err(|_| {
                anyhow::anyhow!("Virtual pointer unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            for _ in 0..repeat {
                for btn_str in &buttons {
                    let btn = if btn_str.starts_with("0x") || btn_str.starts_with("0X") {
                        u8::from_str_radix(&btn_str[2..], 16)?
                    } else {
                        btn_str.parse::<u8>()?
                    };

                    let keycode = u32::from(btn & 0x0f) | 0x110;

                    let down = btn & 0x40 != 0;
                    let up = btn & 0x80 != 0;

                    if down {
                        virtual_pointer.button(keycode, ButtonState::Pressed);
                    }

                    if up {
                        virtual_pointer.button(keycode, ButtonState::Released);
                    }

                    event_queue.roundtrip(&mut whydotool)?;

                    if (btn & 0xC0) == 0
                        && let Some(delay) = next_delay
                    {
                        std::thread::sleep(Duration::from_millis(delay));
                    }
                }
            }
        }

        Commands::Mousemove {
            wheel,
            absolute,
            xpos,
            ypos,
        } => {
            let virtual_pointer = whydotool.virtual_pointer().map_err(|_| {
                anyhow::anyhow!("Virtual pointer unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            if wheel {
                virtual_pointer.scroll(xpos, ypos);
            } else if absolute {
                virtual_pointer.motion_absolute(xpos as u32, ypos as u32);
            } else {
                virtual_pointer.motion(xpos, ypos);
            }

            event_queue.roundtrip(&mut whydotool)?;
        }
        Commands::Key {
            key_presses,
            key_delay,
        } => {
            for key_press in key_presses {
                event_queue.roundtrip(&mut whydotool)?;

                let mut virtual_keyboard = whydotool.virtual_keyboard().map_err(|_| {
                    anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
                })?;

                virtual_keyboard.key(key_press.keycode, key_press.pressed);

                if let Some(key_delay) = key_delay {
                    std::thread::sleep(Duration::from_millis(key_delay));
                }
            }
        }
        Commands::Type {
            strings,
            next_delay,
            key_delay,
            key_hold,
            file,
            ..
        } => {
            let input = match file {
                Some(file) if file.as_str() == "-" => {
                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;

                    buffer.lines().map(ToString::to_string).collect()
                }
                Some(file) => fs::read_to_string(file)?
                    .lines()
                    .map(ToString::to_string)
                    .collect(),
                None => strings,
            };

            let mut virtual_keyboard = whydotool.virtual_keyboard().map_err(|_| {
                anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            for string in input {
                for ch in string.chars() {
                    if let Some((keycode, needs_shift)) = virtual_keyboard.keycode_from_char(ch) {
                        if needs_shift {
                            virtual_keyboard.key(42, 1); // Shift down
                        }

                        virtual_keyboard.key(keycode, 1);
                        std::thread::sleep(Duration::from_millis(key_hold));
                        virtual_keyboard.key(keycode, 0);

                        if needs_shift {
                            virtual_keyboard.key(42, 0); // Shift up
                        }

                        event_queue.roundtrip(&mut whydotool)?;

                        std::thread::sleep(Duration::from_millis(key_delay));
                    }
                }

                if let Some(next_delay) = next_delay {
                    std::thread::sleep(Duration::from_millis(next_delay));
                }
            }
        }
    }

    Ok(())
}
