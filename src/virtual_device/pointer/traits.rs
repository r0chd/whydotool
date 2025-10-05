use crate::output::Outputs;
use wayland_client::protocol::wl_pointer;

pub trait VirtualPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState);

    fn scroll(&self, xpos: f64, ypos: f64);

    fn motion(&self, xpos: f64, ypos: f64);

    fn motion_absolute(&self, xpos: u32, ypos: u32);

    fn outputs(&mut self) -> &mut Outputs;
}
