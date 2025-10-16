use super::traits::VirtualKeyboard;
use crate::{KeymapInfo, portal::remote_desktop::RemoteDesktop};
use xkbcommon::xkb::{self, KEYMAP_COMPILE_NO_FLAGS, KeyDirection, Keycode};

pub struct PortalKeyboard {
    xkb_state: xkb::State,
    remote_desktop: RemoteDesktop,
}

impl PortalKeyboard {
    pub fn try_new(
        remote_desktop: RemoteDesktop,
        keymap_info: &KeymapInfo,
    ) -> anyhow::Result<Self> {
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

        Ok(Self {
            xkb_state,
            remote_desktop,
        })
    }
}

impl VirtualKeyboard for PortalKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State {
        &mut self.xkb_state
    }

    fn key(&mut self, key: Keycode, state: KeyDirection) {
        // xkbcommon doesn't implement Copy for KeyDirection
        #[allow(clippy::needless_match)]
        let state_2 = match state {
            KeyDirection::Down => KeyDirection::Down,
            KeyDirection::Up => KeyDirection::Up,
        };

        self.xkb_state.update_key(key, state);

        self.remote_desktop
            .notify_keyboard_keycode(key, &state_2)
            .unwrap();
    }
}
