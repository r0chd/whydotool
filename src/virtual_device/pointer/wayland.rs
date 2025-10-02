use super::{traits::VirtualPointer, util::Outputs};
use crate::Whydotool;
use wayland_client::{
    QueueHandle,
    globals::GlobalList,
    protocol::{wl_pointer, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};

pub struct WaylandPointer {
    virtual_pointer: zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1,
    outputs: Outputs,
}

impl WaylandPointer {
    pub fn try_new(
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

        Ok(Self {
            virtual_pointer,
            outputs: Outputs::new(globals, qh),
        })
    }
}

impl VirtualPointer for WaylandPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState) {
        self.virtual_pointer.button(0, button, state);
        self.virtual_pointer.frame();
    }

    fn scroll(&self, xpos: f64, ypos: f64) {
        self.virtual_pointer
            .axis(0, wl_pointer::Axis::VerticalScroll, ypos);
        self.virtual_pointer
            .axis(0, wl_pointer::Axis::HorizontalScroll, xpos);
        self.virtual_pointer.frame();
    }

    fn motion(&self, xpos: f64, ypos: f64) {
        self.virtual_pointer.motion(0, xpos, ypos);
        self.virtual_pointer.frame();
    }

    fn motion_absolute(&self, xpos: u32, ypos: u32) {
        let (width, height) = self.outputs.dimensions();

        self.virtual_pointer
            .motion_absolute(0, xpos, ypos, width as u32, height as u32);
        self.virtual_pointer.frame();
    }

    fn outputs(&mut self) -> &mut Outputs {
        &mut self.outputs
    }
}
