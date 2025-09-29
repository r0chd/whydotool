use crate::Whydotool;
use std::{ffi::CString, io::Write, os::fd::AsFd, path::PathBuf};
use wayland_client::{
    QueueHandle, delegate_noop,
    globals::GlobalList,
    protocol::{wl_keyboard, wl_seat},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};
use xkbcommon::xkb::{self, KEYMAP_COMPILE_NO_FLAGS, KEYMAP_FORMAT_TEXT_V1};

pub struct VirtualKeyboard {
    virtual_keyboard: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
    xkb_state: xkb::State,
}

impl VirtualKeyboard {
    pub fn try_new(
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
        seat: &wl_seat::WlSeat,
    ) -> anyhow::Result<Self> {
        let virtual_keyboard = globals
            .bind::<zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1, _, _>(
                &qh,
                1..=1,
                (),
            )
            .map(|virtual_keyboard| virtual_keyboard.create_virtual_keyboard(seat, qh, ()))?;

        let xkb_context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let xkb_keymap = xkb::Keymap::new_from_names(
            &xkb_context,
            "",
            "",
            "us",
            "",
            None,
            KEYMAP_COMPILE_NO_FLAGS,
        )
        .expect("xkbcommon keymap panicked!");
        let xkb_state = xkb::State::new(&xkb_keymap);

        let keymap = xkb_state.get_keymap().get_as_string(KEYMAP_FORMAT_TEXT_V1);
        let keymap = CString::new(keymap).expect("Keymap should not contain interior nul bytes");
        let keymap = keymap.as_bytes_with_nul();
        let dir = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let mut file = tempfile::tempfile_in(dir).expect("File could not be created!");
        file.write_all(keymap).unwrap();
        file.flush().unwrap();

        virtual_keyboard.keymap(
            wl_keyboard::KeymapFormat::XkbV1.into(),
            file.as_fd(),
            keymap.len() as u32,
        );

        Ok(Self {
            xkb_state,
            virtual_keyboard,
        })
    }

    pub fn key(&mut self, key: u32, state: u32) {
        let direction = if state == 1 {
            xkb::KeyDirection::Down
        } else {
            xkb::KeyDirection::Up
        };

        // xkbcommon uses keycodes with an offset of 8
        let xkb_keycode = key + 8;
        self.xkb_state
            .update_key(xkb::Keycode::new(xkb_keycode), direction);

        self.virtual_keyboard.key(0, key, state);

        let depressed = self.xkb_state.serialize_mods(xkb::STATE_MODS_DEPRESSED);
        let latched = self.xkb_state.serialize_mods(xkb::STATE_MODS_LATCHED);
        let locked = self.xkb_state.serialize_mods(xkb::STATE_MODS_LOCKED);
        let group = self.xkb_state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE);

        self.virtual_keyboard
            .modifiers(depressed, latched, locked, group);
    }
}

delegate_noop!(Whydotool: zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1);
delegate_noop!(Whydotool: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1);
