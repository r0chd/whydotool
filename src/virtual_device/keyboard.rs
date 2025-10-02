use crate::{Whydotool, portal::remote_desktop::RemoteDesktopProxyBlocking};
use std::{collections::HashMap, ffi::CString, fs, io::Write, os::fd::AsFd, path::PathBuf};
use wayland_client::{
    QueueHandle, delegate_noop,
    globals::GlobalList,
    protocol::{wl_keyboard, wl_seat},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1, zwp_virtual_keyboard_v1,
};
use xkbcommon::xkb::{self, KEYMAP_COMPILE_NO_FLAGS, KEYMAP_FORMAT_TEXT_V1, keysyms::KEY_Shift_L};
use zbus::zvariant::OwnedObjectPath;

pub enum VirtualKeyboardInner {
    Wayland(zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1),
    Portal {
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
    },
}

pub struct VirtualKeyboard {
    inner: VirtualKeyboardInner,
    xkb_state: xkb::State,
}

impl VirtualKeyboard {
    fn xkb() -> (xkb::State, fs::File, u32) {
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

        (xkb_state, file, keymap.len() as u32)
    }

    pub fn keycode_from_char(&mut self, c: char) -> Option<(u32, bool)> {
        for keycode in 8..=255 {
            let xkb_keycode = xkb::Keycode::new(keycode);

            if self.xkb_state.key_get_one_sym(xkb_keycode) == xkb::utf32_to_keysym(c as u32) {
                return Some((keycode - 8, false));
            }

            self.xkb_state.update_mask(0, KEY_Shift_L, 0, 0, 0, 0);
            if self.xkb_state.key_get_one_sym(xkb_keycode) == xkb::utf32_to_keysym(c as u32) {
                self.xkb_state.update_mask(0, 0, 0, 0, 0, 0);
                return Some((keycode - 8, true));
            }

            self.xkb_state.update_mask(0, 0, 0, 0, 0, 0);
        }

        None
    }

    pub fn from_wayland(
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

        let (xkb_state, file, size) = Self::xkb();

        virtual_keyboard.keymap(wl_keyboard::KeymapFormat::XkbV1.into(), file.as_fd(), size);

        Ok(Self {
            xkb_state,
            inner: VirtualKeyboardInner::Wayland(virtual_keyboard),
        })
    }

    pub fn from_portal(
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
    ) -> Self {
        let (xkb_state, _, _) = Self::xkb();

        Self {
            inner: VirtualKeyboardInner::Portal {
                proxy,
                session_handle,
            },
            xkb_state,
        }
    }

    pub fn key(&mut self, key: u32, state: u32) {
        let direction = if state == 1 {
            xkb::KeyDirection::Down
        } else {
            xkb::KeyDirection::Up
        };

        // xkbcommon uses keycodes with an offset of 8
        let keycode = key + 8;
        let xkb_keycode = xkb::Keycode::new(keycode);
        self.xkb_state.update_key(xkb_keycode, direction);

        match self.inner {
            VirtualKeyboardInner::Wayland(ref virtual_keyboard) => {
                let depressed = self.xkb_state.serialize_mods(xkb::STATE_MODS_DEPRESSED);
                let latched = self.xkb_state.serialize_mods(xkb::STATE_MODS_LATCHED);
                let locked = self.xkb_state.serialize_mods(xkb::STATE_MODS_LOCKED);
                let group = self.xkb_state.serialize_layout(xkb::STATE_LAYOUT_EFFECTIVE);

                virtual_keyboard.key(0, key, state);
                virtual_keyboard.modifiers(depressed, latched, locked, group);
            }
            VirtualKeyboardInner::Portal {
                ref proxy,
                ref session_handle,
            } => {
                let keysym = self.xkb_state.key_get_one_sym(xkb_keycode);
                proxy
                    .notify_keyboard_keysym(
                        session_handle,
                        HashMap::new(),
                        keysym.raw() as i32,
                        state,
                    )
                    .unwrap()
            }
        }
    }
}

delegate_noop!(Whydotool: zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1);
delegate_noop!(Whydotool: zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1);
