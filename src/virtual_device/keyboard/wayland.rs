use super::traits::VirtualKeyboard;
use crate::{Whydotool, virtual_device::keyboard::util::xkb_init};
use std::os::fd::AsFd;
use wayland_client::{
    QueueHandle,
    globals::GlobalList,
    protocol::{wl_keyboard, wl_seat},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};
use xkbcommon::xkb;

pub struct WaylandKeyboard {
    virtual_keyboard: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    xkb_state: xkb::State,
}

impl WaylandKeyboard {
    pub fn try_new(
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
        seat: &wl_seat::WlSeat,
    ) -> anyhow::Result<Self> {
        let virtual_keyboard = globals
            .bind::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, _, _>(
                qh,
                1..=1,
                (),
            )
            .map(|virtual_keyboard| virtual_keyboard.create_virtual_keyboard(seat, qh, ()))?;

        let (xkb_state, file, size) = xkb_init();

        virtual_keyboard.keymap(wl_keyboard::KeymapFormat::XkbV1.into(), file.as_fd(), size);

        Ok(Self {
            xkb_state,
            virtual_keyboard,
        })
    }
}

impl VirtualKeyboard for WaylandKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State {
        &mut self.xkb_state
    }

    fn key(&mut self, key: u32, state: u32) {
        let direction = if state == 1 {
            xkb::KeyDirection::Down
        } else {
            xkb::KeyDirection::Up
        };

        // xkbcommon uses keycodes with an offset of 8
        let keycode = key + 8;
        let xkb_keycode = xkb::Keycode::new(keycode);
        self.xkb_state.update_key(xkb_keycode, direction);

        let depressed = self.xkb_state.serialize_mods(xkb::STATE_MODS_DEPRESSED);
        let latched = self.xkb_state.serialize_mods(xkb::STATE_MODS_LATCHED);
        let locked = self.xkb_state.serialize_mods(xkb::STATE_MODS_LOCKED);
        let group = self.xkb_state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE);

        self.virtual_keyboard.key(0, key, state);
        self.virtual_keyboard
            .modifiers(depressed, latched, locked, group);
    }
}
