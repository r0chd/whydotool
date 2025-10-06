mod cli;
mod stdin;

use clap::Parser;
use cli::Cli;
use cli::Commands;
use libwhydotool::Whydotool;
use std::io::Read;
use std::{fs, io, time::Duration};
use wayland_client::protocol::wl_pointer::ButtonState;
use xkbcommon::xkb;

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;

    let (mut event_queue, mut whydotool) = Whydotool::try_new()?;
    #[cfg(feature = "portals")]
    whydotool.force_portal(cli.force_portal);

    event_queue.dispatch_pending(&mut whydotool)?;
    event_queue.roundtrip(&mut whydotool)?;

    match cli.cmd {
        Commands::Click {
            repeat,
            next_delay,
            buttons,
        } => {
            let virtual_pointer = whydotool.virtual_pointer().map_err(|_| {
                #[cfg(feature = "portals")]
                return anyhow::anyhow!("Virtual pointer unavailable: both compositor protocol support AND desktop portal remote desktop support are missing");
                #[cfg(not(feature = "portals"))]
                return anyhow::anyhow!("Virtual pointer unavailable: compositor lacks protocol support, consider compiling with `portals` feature");
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
                #[cfg(feature = "portals")]
                return anyhow::anyhow!("Virtual pointer unavailable: both compositor protocol support AND desktop portal remote desktop support are missing");
                #[cfg(not(feature = "portals"))]
                return anyhow::anyhow!("Virtual pointer unavailable: compositor lacks protocol support, consider compiling with `portals` feature");
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
                    #[cfg(feature = "portals")]
                    return anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing");
                    #[cfg(not(feature = "portals"))]
                    return anyhow::anyhow!("Virtual keyboard unavailable: compositor lacks protocol support, consider compiling with `portals` feature");
                })?;

                // xkbcommon uses keycodes with an offset of 8
                let keycode = xkb::Keycode::new(key_press.keycode + 8);
                virtual_keyboard.key(keycode, key_press.pressed);

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
                #[cfg(feature = "portals")]
                return anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing");
                #[cfg(not(feature = "portals"))]
                return anyhow::anyhow!("Virtual keyboard unavailable: compositor lacks protocol support, consider compiling with `portals` feature");
            })?;

            for string in input {
                for ch in string.chars() {
                    if let Some((keycode, needs_shift)) = virtual_keyboard.keycode_from_char(ch) {
                        if needs_shift {
                            // xkbcommon uses keycodes with an offset of 8
                            let keycode = xkb::Keycode::new(42 + 8);
                            virtual_keyboard.key(keycode, xkb::KeyDirection::Down); // Shift down
                        }

                        virtual_keyboard.key(keycode, xkb::KeyDirection::Down);
                        std::thread::sleep(Duration::from_millis(key_hold));
                        virtual_keyboard.key(keycode, xkb::KeyDirection::Up);

                        if needs_shift {
                            // xkbcommon uses keycodes with an offset of 8
                            let keycode = xkb::Keycode::new(42 + 8);
                            virtual_keyboard.key(keycode, xkb::KeyDirection::Up); // Shift up
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
        Commands::Stdin => {
            let mut virtual_keyboard = whydotool.virtual_keyboard().map_err(|_| {
                #[cfg(feature = "portals")]
                return anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing");
                #[cfg(not(feature = "portals"))]
                return anyhow::anyhow!("Virtual keyboard unavailable: compositor lacks protocol support, consider compiling with `portals` feature");
            })?;
            let _terminal = stdin::Terminal::configure()?;

            println!("Type anything (CTRL-C to exit):");

            let stdin = std::io::stdin();
            let mut handle = stdin.lock();

            loop {
                let mut buffer = [0u8; 3];
                handle.read(&mut buffer[..3])?;

                println!("Key code: {} {} {}", buffer[0], buffer[1], buffer[2]);

                let Some((keycode, is_uppercase)) =
                    (if buffer[0] == 27 && buffer[1] == 91 && buffer[2] >= 65 && buffer[2] <= 76 {
                        let key = match buffer[2] {
                            65 => 103, // KEY_UP
                            66 => 108, // KEY_DOWN
                            67 => 106, // KEY_RIGHT
                            68 => 105, // KEY_LEFT
                            53 => 104, // KEY_PAGEUP
                            54 => 109, // KEY_PAGEDOWN
                            70 => 107, // KEY_END
                            72 => 102, // KEY_HOME
                            _ => continue,
                        };

                        Some((xkb::Keycode::new(key + 8), false))
                    } else {
                        virtual_keyboard.keycode_from_char(buffer[0] as char)
                    })
                else {
                    continue;
                };

                if is_uppercase {
                    // xkbcommon uses keycodes with an offset of 8
                    let keycode = xkb::Keycode::new(42 + 8);
                    virtual_keyboard.key(keycode, xkb::KeyDirection::Down); // Shift down
                }

                {
                    if let Some(name) = virtual_keyboard.xkb_state().key_get_one_sym(keycode).name()
                    {
                        println!("  Maps to: {name}");
                    }
                }

                virtual_keyboard.key(keycode, xkb::KeyDirection::Down);
                virtual_keyboard.key(keycode, xkb::KeyDirection::Up);

                if is_uppercase {
                    // xkbcommon uses keycodes with an offset of 8
                    let keycode = xkb::Keycode::new(42 + 8);
                    virtual_keyboard.key(keycode, xkb::KeyDirection::Up); // Shift up
                }
            }
        }
    }

    Ok(())
}
