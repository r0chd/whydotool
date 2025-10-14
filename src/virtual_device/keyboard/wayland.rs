use super::traits::VirtualKeyboard;
use crate::{Whydotool, KeymapInfo};
use std::os::fd::AsFd;
use wayland_client::{
    QueueHandle,
    globals::GlobalList,
    protocol::wl_seat,
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};
use xkbcommon::xkb::Keycode;
use xkbcommon::xkb::{self, KeyDirection, KEYMAP_COMPILE_NO_FLAGS};

pub struct WaylandKeyboard {
    virtual_keyboard: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    xkb_state: xkb::State,
}

impl WaylandKeyboard {
    pub fn try_new(
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
        seat: &wl_seat::WlSeat,
        keymap_info: &KeymapInfo,
    ) -> anyhow::Result<Self> {
        let virtual_keyboard = globals
            .bind::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, _, _>(
                qh,
                1..=1,
                (),
            )
            .map(|virtual_keyboard| virtual_keyboard.create_virtual_keyboard(seat, qh, ()))
            .map_err(|_| anyhow::anyhow!("Compositor does not support Virtual Keyboard protocol, compile whydotool with `portals` feature"))?;

        // Create xkb_state from the keymap fd
        let xkb_context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let xkb_keymap = unsafe {
            xkb::Keymap::new_from_fd(
                &xkb_context,
                keymap_info.fd.try_clone().unwrap(),
                keymap_info.size as usize,
                keymap_info.format.into(),
                KEYMAP_COMPILE_NO_FLAGS,
            )?
        };
        let xkb_state = xkb::State::new(xkb_keymap.as_ref().unwrap());

        virtual_keyboard.keymap(keymap_info.format.into(), keymap_info.fd.as_fd(), keymap_info.size);

        Ok(Self {
            virtual_keyboard,
            xkb_state,
        })
    }
}

impl VirtualKeyboard for WaylandKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State {
        &mut self.xkb_state
    }

    fn key(&mut self, key: Keycode, state: KeyDirection) {
        let raw_state = match state {
            KeyDirection::Down => 1,
            _ => 0,
        };

        self.xkb_state.update_key(key, state);

        let depressed = self.xkb_state.serialize_mods(xkb::STATE_MODS_DEPRESSED);
        let latched = self.xkb_state.serialize_mods(xkb::STATE_MODS_LATCHED);
        let locked = self.xkb_state.serialize_mods(xkb::STATE_MODS_LOCKED);
        let group = self.xkb_state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE);

        self.virtual_keyboard.key(0, key.raw() - 8, raw_state);
        self.virtual_keyboard
            .modifiers(depressed, latched, locked, group);
    }
}
