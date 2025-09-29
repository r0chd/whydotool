mod cli;
mod virtual_keyboard;

use clap::Parser;
use cli::Cli;
use virtual_keyboard::VirtualKeyboard;
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_dispatch, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

use crate::cli::Commands;

struct Whydotool {
    virtual_pointer: Option<zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1>,
    virtual_keyboard: Option<VirtualKeyboard>,
}

impl Whydotool {
    fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> Self {
        let seat = globals.bind::<wl_seat::WlSeat, _, _>(qh, 1..=4, ()).ok();

        let virtual_pointer = globals
            .bind::<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, _, _>(
                &qh,
                1..=1,
                (),
            )
            .map(|virtual_pointer_manager| {
                virtual_pointer_manager.create_virtual_pointer(seat.as_ref(), qh, ())
            })
            .ok();

        let virtual_keyboard = seat
            .as_ref()
            .map(|seat| VirtualKeyboard::try_new(globals, qh, seat).ok())
            .flatten();

        Self {
            virtual_pointer,
            virtual_keyboard,
        }
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

    let mut whydotool = Whydotool::new(&globals, &qh);

    match cli.cmd {
        Commands::Click {} => unimplemented!(),
        Commands::MouseMove {} => unimplemented!(),
        Commands::Key { key } => match whydotool.virtual_keyboard.as_ref() {
            Some(virtual_keyboard) => {
                for key_press in key {
                    virtual_keyboard.key(key_press.keycode, key_press.pressed);
                }
            }
            None => {}
        },
        Commands::Type {} => unimplemented!(),
    }

    event_queue.blocking_dispatch(&mut whydotool)?;

    Ok(())
}
