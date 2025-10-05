use super::{traits::VirtualKeyboard, util::xkb_init};
use crate::portal::remote_desktop::RemoteDesktopProxyBlocking;
use std::collections::HashMap;
use xkbcommon::xkb::{self, KeyDirection, Keycode};
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
            xkb_state,
            proxy,
            session_handle,
        }
    }
}

impl VirtualKeyboard for PortalKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State {
        &mut self.xkb_state
    }

    fn key(&mut self, key: Keycode, state: KeyDirection) {
        let raw_state = match state {
            KeyDirection::Down => 1,
            _ => 0,
        };

        self.xkb_state.update_key(key, state);

        self.proxy
            .notify_keyboard_keycode(
                &self.session_handle,
                HashMap::new(),
                key.raw() as i32 - 8,
                raw_state,
            )
            .unwrap();
    }
}
