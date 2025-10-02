mod cli;
mod portal;
mod virtual_device;

use crate::cli::Commands;
use crate::virtual_device::{keyboard, pointer};
use clap::Parser;
use cli::Cli;
use std::time::Duration;
use std::{fs, io};
use virtual_device::{keyboard::traits::VirtualKeyboard, pointer::traits::VirtualPointer};
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_dispatch, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

struct Whydotool {
    virtual_pointer: Option<Box<dyn VirtualPointer>>,
    virtual_keyboard: Option<Box<dyn VirtualKeyboard>>,
}

impl Whydotool {
    fn try_new(cli: &Cli, globals: &GlobalList, qh: &QueueHandle<Self>) -> anyhow::Result<Self> {
        let seat = globals.bind::<wl_seat::WlSeat, _, _>(qh, 1..=4, ()).ok();

        let virtual_pointer: Option<Box<dyn VirtualPointer>> =
            if matches!(cli.cmd, Commands::Click { .. } | Commands::Mousemove { .. }) {
                pointer::virtual_pointer(globals, qh, seat.as_ref(), cli.force_portal).ok()
            } else {
                None
            };

        let virtual_keyboard: Option<Box<dyn VirtualKeyboard>> =
            if matches!(cli.cmd, Commands::Type { .. } | Commands::Key { .. }) {
                keyboard::virtual_keyboard(globals, qh, seat.as_ref(), cli.force_portal).ok()
            } else {
                None
            };

        Ok(Self {
            virtual_pointer,
            virtual_keyboard,
        })
    }
}

impl Dispatch<wl_seat::WlSeat, ()> for Whydotool {
    fn event(
        _: &mut Self,
        _: &wl_seat::WlSeat,
        _: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

delegate_dispatch!(Whydotool: [wl_registry::WlRegistry: GlobalListContents] => Whydotool);
delegate_noop!(Whydotool: zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1);
delegate_noop!(Whydotool: zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1);

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;

    let conn = Connection::connect_to_env()?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let mut whydotool = Whydotool::try_new(&cli, &globals, &qh)?;

    event_queue.dispatch_pending(&mut whydotool)?;
    event_queue.roundtrip(&mut whydotool)?;

    match cli.cmd {
        Commands::Click {
            repeat,
            next_delay,
            buttons,
        } => {
            let virtual_pointer = whydotool.virtual_pointer.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual pointer unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            for _ in 0..repeat {
                for btn_str in buttons.iter() {
                    let btn = if btn_str.starts_with("0x") || btn_str.starts_with("0X") {
                        u8::from_str_radix(&btn_str[2..], 16)?
                    } else {
                        btn_str.parse::<u8>()?
                    };

                    let keycode = (btn & 0x0f) as u32 | 0x110;

                    let down = btn & 0x40 != 0;
                    let up = btn & 0x80 != 0;

                    if down {
                        virtual_pointer.button(keycode, ButtonState::Pressed);
                    }

                    if up {
                        virtual_pointer.button(keycode, ButtonState::Released);
                    }

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
            let virtual_pointer = whydotool.virtual_pointer.take().ok_or_else(|| {
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
            let mut virtual_keyboard = whydotool.virtual_keyboard.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            for key_press in key_presses {
                virtual_keyboard.key(key_press.keycode, key_press.pressed);

                event_queue.roundtrip(&mut whydotool)?;

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
            let mut virtual_keyboard = whydotool.virtual_keyboard.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual keyboard unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            let input = match file {
                Some(file) if file.as_str() == "-" => {
                    let mut buffer = String::new();
                    io::stdin().read_line(&mut buffer)?;

                    buffer.lines().map(|s| s.to_string()).collect()
                }
                Some(file) => fs::read_to_string(file)?
                    .lines()
                    .map(|s| s.to_string())
                    .collect(),
                None => strings,
            };

            for string in input.iter() {
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
