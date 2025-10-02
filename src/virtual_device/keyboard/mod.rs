mod portal;
pub mod traits;
mod util;
mod wayland;

use crate::{Whydotool, portal::remote_desktop};
use wayland_client::{QueueHandle, delegate_noop, globals::GlobalList, protocol::wl_seat};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};

pub fn virtual_keyboard(
    globals: &GlobalList,
    qh: &QueueHandle<Whydotool>,
    seat: Option<&wl_seat::WlSeat>,
    force_portal: bool,
) -> anyhow::Result<Box<dyn traits::VirtualKeyboard>> {
    if !force_portal
        && let Some(seat) = seat
        && let Ok(ptr) = wayland::WaylandKeyboard::try_new(globals, qh, seat)
    {
        return Ok(Box::new(ptr));
    }

    let remote_desktop = remote_desktop::RemoteDesktop::builder()
        .keyboard(true)
        .try_build()?;
    Ok(Box::new(portal::PortalKeyboard::new(
        remote_desktop.proxy,
        remote_desktop.session_handle,
    )))
}

delegate_noop!(Whydotool: zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1);
delegate_noop!(Whydotool: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1);
