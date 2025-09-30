use std::collections::HashMap;

use crate::{Whydotool, remote_desktop::RemoteDesktopProxyBlocking};
use wayland_client::{
    QueueHandle,
    globals::GlobalList,
    protocol::{wl_pointer, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};
use zbus::zvariant::OwnedObjectPath;

pub enum VirtualPointerInner {
    Wayland(zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1),
    Portal {
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
    },
}

pub struct VirtualPointer(VirtualPointerInner);

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

        Ok(Self(VirtualPointerInner::Wayland(virtual_pointer)))
    }

    pub fn from_portal(
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
    ) -> Self {
        Self(VirtualPointerInner::Portal {
            proxy,
            session_handle,
        })
    }

    pub fn button(&self, button: u32, state: wl_pointer::ButtonState) {
        match self.0 {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.button(0, button, state);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
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
        match self.0 {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.axis(0, wl_pointer::Axis::VerticalScroll, ypos);
                virtual_pointer.axis(0, wl_pointer::Axis::HorizontalScroll, xpos);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
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
        match self.0 {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.motion(0, xpos, ypos);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
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

    pub fn motion_absolute(&self, xpos: u32, ypos: u32, x_extent: u32, y_extent: u32) {
        match self.0 {
            VirtualPointerInner::Wayland(ref virtual_pointer) => {
                virtual_pointer.motion_absolute(0, xpos, ypos, x_extent, y_extent);
                virtual_pointer.frame();
            }
            VirtualPointerInner::Portal {
                ref proxy,
                ref session_handle,
            } => proxy
                .notify_pointer_motion_absolute(
                    session_handle.to_owned(),
                    HashMap::new(),
                    0,
                    xpos as f32,
                    ypos as f32,
                )
                .unwrap(),
        }
    }
}
