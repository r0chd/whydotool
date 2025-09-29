mod cli;
mod virtual_keyboard;

use crate::cli::Commands;
use clap::Parser;
use cli::Cli;
use std::time::Duration;
use virtual_keyboard::VirtualKeyboard;
use wayland_client::Proxy;
use wayland_client::protocol::wl_pointer::ButtonState;
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_dispatch, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{wl_output, wl_pointer, wl_registry, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

#[derive(Debug)]
pub struct Output {
    name: Option<Box<str>>,
    wl_output: wl_output::WlOutput,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Output {
    fn new(wl_output: wl_output::WlOutput) -> Self {
        Self {
            name: None,
            wl_output,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Whydotool {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(output) = state
            .outputs
            .iter_mut()
            .find(|output| output.wl_output == *wl_output)
        else {
            return;
        };

        match event {
            wl_output::Event::Name { name } => output.name = Some(name.into()),
            wl_output::Event::Geometry {
                x,
                y,
                physical_width: _,
                physical_height: _,
                subpixel: _,
                make: _,
                model: _,
                transform: _,
            } => {
                output.x = x;
                output.y = y;
            }
            wl_output::Event::Mode {
                flags: _,
                width,
                height,
                refresh: _,
            } => {
                output.width = width;
                output.height = height;
            }
            _ => {}
        }
    }
}

struct Whydotool {
    virtual_pointer: Option<zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1>,
    virtual_keyboard: Option<VirtualKeyboard>,
    outputs: Vec<Output>,
}

impl Whydotool {
    fn new(globals: &GlobalList, qh: &QueueHandle<Self>) -> Self {
        let seat = globals.bind::<wl_seat::WlSeat, _, _>(qh, 1..=4, ()).ok();

        let virtual_pointer = globals
            .bind::<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, _, _>(
                &qh,
                1..=2,
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

        let mut outputs = Vec::new();
        globals.contents().with_list(|list| {
            list.iter()
                .filter(|global| global.interface == wl_output::WlOutput::interface().name)
                .for_each(|global| {
                    let wl_output = globals
                        .registry()
                        .bind(global.name, global.version, &qh, ());
                    let output = Output::new(wl_output);
                    outputs.push(output);
                });
        });

        Self {
            outputs,
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

    event_queue.dispatch_pending(&mut whydotool)?;
    event_queue.roundtrip(&mut whydotool)?;

    match cli.cmd {
        Commands::Click {} => {
            let virtual_pointer = whydotool.virtual_pointer.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual keyboard protocol is not supported by the compositor")
            })?;

            virtual_pointer.button(0, 0, ButtonState::Pressed);
            virtual_pointer.button(0, 0, ButtonState::Released);
        }
        Commands::Mousemove {
            wheel,
            absolute,
            xpos,
            ypos,
        } => {
            let virtual_pointer = whydotool.virtual_pointer.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual keyboard protocol is not supported by the compositor")
            })?;

            let (x_extent, y_extent) = whydotool.outputs.iter().fold((0, 0), |(w, h), output| {
                let output_right = output.x + output.width;
                let output_bottom = output.y + output.height;
                (w.max(output_right), h.max(output_bottom))
            });

            if wheel {
                virtual_pointer.axis(0, wl_pointer::Axis::VerticalScroll, ypos as f64);
                virtual_pointer.axis(0, wl_pointer::Axis::HorizontalScroll, xpos as f64);
            } else {
                if absolute {
                    virtual_pointer.motion_absolute(
                        0,
                        xpos,
                        ypos,
                        x_extent as u32,
                        y_extent as u32,
                    );
                } else {
                    virtual_pointer.motion(0, xpos as f64, ypos as f64);
                }
            }

            virtual_pointer.frame();
            event_queue.roundtrip(&mut whydotool)?;
        }
        Commands::Key {
            key_presses,
            key_delay,
        } => {
            let mut virtual_keyboard = whydotool.virtual_keyboard.take().ok_or_else(|| {
                anyhow::anyhow!("Virtual keyboard protocol is not supported by the compositor")
            })?;

            for key_press in key_presses {
                virtual_keyboard.key(key_press.keycode, key_press.pressed);

                event_queue.roundtrip(&mut whydotool)?;

                if let Some(key_delay) = key_delay {
                    std::thread::sleep(Duration::from_millis(key_delay));
                }
            }
        }
        Commands::Type {} => unimplemented!(),
    }

    Ok(())
}
