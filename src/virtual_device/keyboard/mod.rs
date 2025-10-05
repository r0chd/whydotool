#[cfg(feature = "portals")]
mod portal;
pub mod traits;
mod util;
mod wayland;

use crate::Whydotool;
#[cfg(feature = "portals")]
use crate::portal::remote_desktop;
use wayland_client::{QueueHandle, delegate_noop, globals::GlobalList, protocol::wl_seat};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};

#[cfg(feature = "portals")]
pub fn virtual_keyboard(
    globals: &GlobalList,
    qh: &QueueHandle<Whydotool>,
    seat: Option<&wl_seat::WlSeat>,
    force_portal: bool,
) -> anyhow::Result<Box<dyn traits::VirtualKeyboard>> {
    if !force_portal {
        if let Some(seat) = seat {
            if let Ok(ptr) = wayland::WaylandKeyboard::try_new(globals, qh, seat) {
                return Ok(Box::new(ptr));
            }
        }
    }

    let remote_desktop = remote_desktop::RemoteDesktop::builder()
        .keyboard(true)
        .try_build()?;
    Ok(Box::new(portal::PortalKeyboard::new(
        remote_desktop.proxy,
        remote_desktop.session_handle,
    )))
}

#[cfg(not(feature = "portals"))]
pub fn virtual_keyboard(
    globals: &GlobalList,
    qh: &QueueHandle<Whydotool>,
    seat: Option<&wl_seat::WlSeat>,
) -> anyhow::Result<Box<dyn traits::VirtualKeyboard>> {
    let seat = seat.ok_or_else(|| anyhow::anyhow!("No seat provided for Wayland keyboard"))?;
    let ptr = wayland::WaylandKeyboard::try_new(globals, qh, seat)?;
    Ok(Box::new(ptr))
}

delegate_noop!(Whydotool: zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1);
delegate_noop!(Whydotool: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1);
