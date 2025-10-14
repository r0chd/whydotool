use wayland_client::protocol::wl_pointer;

pub trait VirtualPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState);

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;

    fn scroll(&self, xpos: f64, ypos: f64);

    fn motion(&self, xpos: f64, ypos: f64);

    fn motion_absolute(&self, xpos: u32, ypos: u32);
}
