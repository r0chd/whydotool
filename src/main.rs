mod virtual_keyboard;

use calloop::EventLoop;
use calloop_wayland_source::WaylandSource;
use clap::Parser;
use virtual_keyboard::VirtualKeyboard;
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_dispatch, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

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

        if virtual_pointer.is_none() {
            log::warn!(
                "Failed to initialize virtual pointer: \
         compositor does not support zwlr-virtual-pointer-v1"
            );
        }

        if virtual_keyboard.is_none() {
            log::warn!(
                "Failed to initialize virtual keyboard: \
         compositor does not support zwp-virtual-keyboard-v1"
            );
        }

        virtual_keyboard.as_ref().unwrap().key(30, 1);
        virtual_keyboard.as_ref().unwrap().key(30, 0);

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

#[derive(Parser, Debug)]
#[command(name = "whydotool", about = "Wayland command-line automation tool")]
pub struct Cli {}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let conn = Connection::connect_to_env()?;
    let (globals, event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let mut event_loop = EventLoop::try_new()?;
    let mut whydotool = Whydotool::new(&globals, &qh);

    WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;

    event_loop.run(None, &mut whydotool, |_| {})?;

    Ok(())
}
