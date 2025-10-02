mod portal;
pub mod traits;
mod util;
mod wayland;

use crate::Whydotool;
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_seat};

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

    let remote_desktop = crate::portal::remote_desktop::RemoteDesktop::try_new()?;
    let portal_ptr = portal::PortalPointer::try_new(
        remote_desktop.proxy,
        remote_desktop.session_handle,
        globals,
        qh,
    )?;
    Ok(Box::new(portal_ptr))
}
