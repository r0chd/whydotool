use super::traits::VirtualPointer;
use crate::{
    Whydotool, portal::remote_desktop::RemoteDesktopProxyBlocking,
    virtual_device::pointer::util::Outputs,
};
use std::collections::HashMap;
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_pointer};
use zbus::zvariant::OwnedObjectPath;

pub struct PortalPointer {
    outputs: Outputs,
    proxy: RemoteDesktopProxyBlocking<'static>,
    session_handle: OwnedObjectPath,
}

impl PortalPointer {
    pub fn try_new(
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            outputs: Outputs::new(globals, qh),
            session_handle,
            proxy,
        })
    }
}

impl VirtualPointer for PortalPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState) {
        self.proxy
            .notify_pointer_button(
                &self.session_handle,
                HashMap::new(),
                button as i32,
                if state == wl_pointer::ButtonState::Released {
                    0
                } else {
                    1
                },
            )
            .unwrap();
    }

    fn scroll(&self, xpos: f64, ypos: f64) {
        self.proxy
            .notify_pointer_axis(
                &self.session_handle,
                HashMap::new(),
                xpos as f32,
                ypos as f32,
            )
            .unwrap()
    }

    fn motion(&self, xpos: f64, ypos: f64) {
        self.proxy
            .notify_pointer_motion(
                &self.session_handle,
                HashMap::new(),
                xpos as f32,
                ypos as f32,
            )
            .unwrap()
    }

    fn motion_absolute(&self, xpos: u32, ypos: u32) {
        let (_, _) = self.outputs.dimensions();

        self.proxy
            .notify_pointer_motion_absolute(
                &self.session_handle,
                HashMap::new(),
                0,
                xpos as f32,
                ypos as f32,
            )
            .unwrap()
    }

    fn outputs(&mut self) -> &mut Outputs {
        &mut self.outputs
    }
}
