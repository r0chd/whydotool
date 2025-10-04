#[cfg(feature = "portals")]
mod portal;
pub mod traits;
mod util;
mod wayland;

use crate::Whydotool;
#[cfg(feature = "portals")]
use crate::portal::remote_desktop;
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_seat};

#[cfg(feature = "portals")]
pub fn virtual_pointer(
    globals: &GlobalList,
    qh: &QueueHandle<Whydotool>,
    seat: Option<&wl_seat::WlSeat>,
    force_portal: bool,
) -> anyhow::Result<Box<dyn traits::VirtualPointer>> {
    if !force_portal {
        if let Ok(ptr) = wayland::WaylandPointer::try_new(globals, qh, seat) {
            return Ok(Box::new(ptr));
        }
    }

    let remote_desktop = remote_desktop::RemoteDesktop::builder()
        .pointer(true)
        .screencast(true)
        .try_build()?;
    let portal_ptr = portal::PortalPointer::new(
        remote_desktop.proxy,
        remote_desktop.session_handle,
        remote_desktop.screencast.unwrap(),
        globals,
        qh,
    );
    Ok(Box::new(portal_ptr))
}

#[cfg(not(feature = "portals"))]
pub fn virtual_pointer(
    globals: &GlobalList,
    qh: &QueueHandle<Whydotool>,
    seat: Option<&wl_seat::WlSeat>,
) -> anyhow::Result<Box<dyn traits::VirtualPointer>> {
    let ptr = wayland::WaylandPointer::try_new(globals, qh, seat)?;
    Ok(Box::new(ptr))
}
