use super::traits::VirtualKeyboard;
use crate::portal::remote_desktop::RemoteDesktop;
use xkbcommon::xkb::{self, KeyDirection, Keycode, KEYMAP_COMPILE_NO_FLAGS};

pub struct PortalKeyboard {
    xkb_state: xkb::State,
    remote_desktop: RemoteDesktop,
}

impl PortalKeyboard {
    pub fn new(remote_desktop: RemoteDesktop) -> Self {
        // Create a default US keymap for portal implementation
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
        .expect("Failed to create default keymap");
        let xkb_state = xkb::State::new(&xkb_keymap);

        Self {
            xkb_state,
            remote_desktop,
        }
    }
}

impl VirtualKeyboard for PortalKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State {
        &mut self.xkb_state
    }

    fn key(&mut self, key: Keycode, state: KeyDirection) {
        // xkbcommon doesn't implement Copy for KeyDirection
        let state_2 = match state {
            KeyDirection::Down => KeyDirection::Down,
            KeyDirection::Up => KeyDirection::Up,
        };

        self.xkb_state.update_key(key, state);

        self.remote_desktop
            .notify_keyboard_keycode(key, state_2)
            .unwrap();
    }
}
