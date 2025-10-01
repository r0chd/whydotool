mod cli;
mod portal;
mod virtual_device;

use crate::cli::Commands;
use clap::Parser;
use cli::Cli;
use portal::remote_desktop;
use std::time::Duration;
use virtual_device::{keyboard::VirtualKeyboard, pointer::VirtualPointer};
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
    virtual_pointer: Option<VirtualPointer>,
    virtual_keyboard: Option<VirtualKeyboard>,
}

impl Whydotool {
    fn try_new(globals: &GlobalList, qh: &QueueHandle<Self>) -> anyhow::Result<Self> {
        let seat = globals.bind::<wl_seat::WlSeat, _, _>(qh, 1..=4, ()).ok();

        let mut virtual_pointer = VirtualPointer::from_wayland(globals, qh, seat.as_ref()).ok();

        let mut virtual_keyboard = seat
            .as_ref()
            .map(|seat| VirtualKeyboard::from_wayland(globals, qh, seat).ok())
            .flatten();

        if virtual_pointer.is_none() || virtual_keyboard.is_none() {
            let remote_desktop = remote_desktop::RemoteDesktop::try_new()?;

            if virtual_keyboard.is_none() {
                virtual_keyboard = Some(VirtualKeyboard::from_portal(
                    remote_desktop.proxy.clone(),
                    remote_desktop.session_handle.clone(),
                ));
            }

            if virtual_pointer.is_none() {
                virtual_pointer = Some(VirtualPointer::from_portal(
                    remote_desktop.proxy,
                    remote_desktop.session_handle,
                    globals,
                    qh,
                )?)
            }
        }

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

    let mut whydotool = Whydotool::try_new(&globals, &qh)?;

    event_queue.dispatch_pending(&mut whydotool)?;
    event_queue.roundtrip(&mut whydotool)?;

    match cli.cmd {
        Commands::Click {} => {
            let virtual_pointer = whydotool.virtual_pointer.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual pointer unavailable: both compositor protocol support AND desktop portal remote desktop support are missing")
            })?;

            virtual_pointer.button(0, ButtonState::Pressed);
            virtual_pointer.button(0, ButtonState::Released);
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
                virtual_pointer.scroll(xpos as f64, ypos as f64);
            } else {
                if absolute {
                    virtual_pointer.motion_absolute(xpos, ypos);
                } else {
                    virtual_pointer.motion(xpos as f64, ypos as f64);
                }
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
        Commands::Type { .. } => {}
    }

    Ok(())
}
