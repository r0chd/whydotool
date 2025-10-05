#[cfg(feature = "portals")]
mod portal;
mod virtual_device;

#[cfg(feature = "portals")]
use portal::remote_desktop::RemoteDesktop;
#[cfg(feature = "portals")]
use virtual_device::keyboard::portal::PortalKeyboard;
#[cfg(feature = "portals")]
use virtual_device::pointer::portal::PortalPointer;
use virtual_device::{
    keyboard::{traits::VirtualKeyboard, wayland::WaylandKeyboard},
    pointer::{traits::VirtualPointer, util::Outputs, wayland::WaylandPointer},
};
use wayland_client::EventQueue;
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_dispatch, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

#[derive(Debug, Clone)]
pub struct KeyPress {
    pub keycode: u32,
    pub pressed: u32,
}

pub struct Whydotool {
    force_portal: bool,
    globals: GlobalList,
    qh: QueueHandle<Self>,
    seat: Option<wl_seat::WlSeat>,
    outputs: Outputs,
}

impl Whydotool {
    #[cfg(feature = "portals")]
    pub fn try_new() -> anyhow::Result<(EventQueue<Self>, Self)> {
        let conn = Connection::connect_to_env()?;
        let (globals, event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=4, ()).ok();

        Ok((
            event_queue,
            Self {
                outputs: Outputs::new(&globals, &qh),
                force_portal: false,
                globals,
                qh,
                seat,
            },
        ))
    }

    #[cfg(not(feature = "portals"))]
    pub fn try_new() -> anyhow::Result<(EventQueue<Self>, Self)> {
        let conn = Connection::connect_to_env()?;
        let (globals, event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=4, ()).ok();

        Ok((
            event_queue,
            Self {
                outputs: Outputs::new(&globals, &qh),
                force_portal: false,
                globals,
                qh,
                seat,
            },
        ))
    }

    pub fn force_portal(&mut self, force_portal: bool) {
        self.force_portal = force_portal;
    }

    pub fn virtual_keyboard(&self) -> anyhow::Result<Box<dyn VirtualKeyboard>> {
        #[cfg(feature = "portals")]
        {
            if !self.force_portal
                && let Some(seat) = self.seat.as_ref()
                && let Ok(ptr) = WaylandKeyboard::try_new(&self.globals, &self.qh, seat)
            {
                return Ok(Box::new(ptr));
            }

            let remote_desktop = RemoteDesktop::builder().keyboard(true).try_build()?;
            Ok(Box::new(PortalKeyboard::new(
                remote_desktop.proxy,
                remote_desktop.session_handle,
            )))
        }
        #[cfg(not(feature = "portals"))]
        {
            let seat = self
                .seat
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("No seat provided for Wayland keyboard"))?;
            Ok(Box::new(WaylandKeyboard::try_new(
                &self.globals,
                &self.qh,
                seat,
            )?))
        }
    }

    pub fn virtual_pointer(&self) -> anyhow::Result<Box<dyn VirtualPointer>> {
        #[cfg(feature = "portals")]
        {
            if !self.force_portal
                && let Ok(ptr) = WaylandPointer::try_new(
                    &self.globals,
                    &self.qh,
                    self.seat.as_ref(),
                    self.outputs.clone(),
                )
            {
                return Ok(Box::new(ptr));
            }

            let remote_desktop = RemoteDesktop::builder()
                .pointer(true)
                .screencast(true)
                .try_build()?;
            let portal_ptr = PortalPointer::new(
                remote_desktop.proxy,
                remote_desktop.session_handle,
                remote_desktop.screencast.unwrap(),
                &self.globals,
                &self.qh,
            );
            Ok(Box::new(portal_ptr))
        }
        #[cfg(not(feature = "portals"))]
        {
            Ok(Box::new(WaylandPointer::try_new(
                &self.globals,
                &self.qh,
                self.seat.as_ref(),
                self.outputs.clone(),
            )?))
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
