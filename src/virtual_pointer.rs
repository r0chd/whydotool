use crate::{Whydotool, remote_desktop::RemoteDesktopProxyBlocking};
use pipewire::properties::properties;
use pipewire::{self as pw, thread_loop::ThreadLoop};
use std::collections::HashMap;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    globals::GlobalList,
    protocol::{wl_output, wl_pointer, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};
use zbus::zvariant::OwnedObjectPath;

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
        if let Some(virtual_pointer) = state.virtual_pointer.as_mut()
            && let Some(output) = virtual_pointer
                .outputs
                .iter_mut()
                .find(|output| output.wl_output == *wl_output)
        {
            match event {
                wl_output::Event::Name { name } => output.name = Some(name.into()),
                wl_output::Event::Geometry { x, y, .. } => {
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
}

pub enum VirtualPointerInner {
    Wayland(zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1),
    Portal {
        thread_loop: ThreadLoop,
        stream: pw::stream::Stream,
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
    },
}

pub struct VirtualPointer {
    inner: VirtualPointerInner,
    outputs: Vec<Output>,
}

impl VirtualPointer {
    pub fn from_wayland(
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
        seat: Option<&wl_seat::WlSeat>,
    ) -> anyhow::Result<Self> {
        let virtual_pointer = globals
            .bind::<zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1, _, _>(
                &qh,
                1..=2,
                (),
            )
            .map(|virtual_pointer_manager| {
                virtual_pointer_manager.create_virtual_pointer(seat, qh, ())
            })?;

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

        Ok(Self {
            inner: VirtualPointerInner::Wayland(virtual_pointer),
            outputs,
        })
    }

    pub fn from_portal(
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
    ) -> anyhow::Result<Self> {
        let thread_loop = unsafe { pw::thread_loop::ThreadLoop::new(Some("whydotool"), None)? };
        let context = pw::context::Context::new(&thread_loop)?;
        let core = context.connect(None)?;

        let stream = pw::stream::Stream::new(
            &core,
            "whydotool",
            properties! {
                *pw::keys::MEDIA_TYPE => "Video",
                *pw::keys::MEDIA_CATEGORY => "Capture",
                *pw::keys::MEDIA_ROLE => "RemoteDesktop",
                *pw::keys::VIDEO_SIZE => "640x480",
                *pw::keys::VIDEO_FORMAT => "RGB",
            },
        )?;

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

        Ok(Self {
            outputs,
            inner: VirtualPointerInner::Portal {
                thread_loop,
                stream,
                proxy,
                session_handle,
            },
        })
    }

    pub fn button(&self, button: u32, state: wl_pointer::ButtonState) {
        match self.inner {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.button(0, button, state);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
                ..
            } => proxy
                .notify_pointer_button(
                    session_handle.to_owned(),
                    HashMap::new(),
                    button as i32,
                    if state == wl_pointer::ButtonState::Released {
                        0
                    } else {
                        1
                    },
                )
                .unwrap(),
        }
    }

    pub fn scroll(&self, xpos: f64, ypos: f64) {
        match self.inner {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.axis(0, wl_pointer::Axis::VerticalScroll, ypos);
                virtual_pointer.axis(0, wl_pointer::Axis::HorizontalScroll, xpos);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
                ..
            } => proxy
                .notify_pointer_axis(
                    session_handle.to_owned(),
                    HashMap::new(),
                    xpos as f32,
                    ypos as f32,
                )
                .unwrap(),
        }
    }

    pub fn motion(&self, xpos: f64, ypos: f64) {
        match self.inner {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.motion(0, xpos, ypos);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
                ..
            } => proxy
                .notify_pointer_motion(
                    session_handle.to_owned(),
                    HashMap::new(),
                    xpos as f32,
                    ypos as f32,
                )
                .unwrap(),
        }
    }

    pub fn motion_absolute(&self, xpos: u32, ypos: u32) {
        match self.inner {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                let (x_extent, y_extent) = self.outputs.iter().fold((0, 0), |(w, h), output| {
                    let output_right = output.x + output.width;
                    let output_bottom = output.y + output.height;
                    (w.max(output_right), h.max(output_bottom))
                });

                virtual_pointer.motion_absolute(0, xpos, ypos, x_extent as u32, y_extent as u32);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
                ref stream,
                ..
            } => proxy
                .notify_pointer_motion_absolute(
                    session_handle.to_owned(),
                    HashMap::new(),
                    stream.node_id(),
                    xpos as f32,
                    ypos as f32,
                )
                .unwrap(),
        }
    }
}
