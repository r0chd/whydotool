use super::{traits::VirtualKeyboard, util::xkb_init};
use crate::portal::remote_desktop::RemoteDesktopProxyBlocking;
use std::collections::HashMap;
use xkbcommon::xkb;
use zbus::zvariant::OwnedObjectPath;

pub struct PortalKeyboard {
    xkb_state: xkb::State,
    proxy: RemoteDesktopProxyBlocking<'static>,
    session_handle: OwnedObjectPath,
}

impl PortalKeyboard {
    pub fn new(
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
    ) -> Self {
        let (xkb_state, _, _) = xkb_init();

        Self {
            proxy,
            session_handle,
            xkb_state,
        }
    }
}

impl VirtualKeyboard for PortalKeyboard {
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

        let keysym = self.xkb_state.key_get_one_sym(xkb_keycode);
        self.proxy
            .notify_keyboard_keysym(
                &self.session_handle,
                HashMap::new(),
                keysym.raw() as i32,
                state,
            )
            .unwrap()
    }
}
