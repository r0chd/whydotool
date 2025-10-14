use xkbcommon::xkb::{self, KeyDirection, Keycode, keysyms::KEY_Shift_L};

pub trait VirtualKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State;

    // https://lists.x.org/archives/wayland-devel/2021-December/042056.html
    fn keycode_from_char(&mut self, c: char) -> Option<(Keycode, bool)> {
        let xkb_state = self.xkb_state();
        let target_keysym = xkb::utf32_to_keysym(c as u32);

        let current_mods = xkb_state.serialize_mods(xkb::STATE_MODS_DEPRESSED);

        for keycode in 8..=255 {
            let xkb_keycode = xkb::Keycode::new(keycode);

            xkb_state.update_mask(0, 0, 0, 0, 0, 0);
            if xkb_state.key_get_one_sym(xkb_keycode) == target_keysym {
                xkb_state.update_mask(current_mods, 0, 0, 0, 0, 0);
                return Some((xkb_keycode, false));
            }

            xkb_state.update_mask(0, KEY_Shift_L, 0, 0, 0, 0);
            if xkb_state.key_get_one_sym(xkb_keycode) == target_keysym {
                xkb_state.update_mask(current_mods, 0, 0, 0, 0, 0);
                return Some((xkb_keycode, true));
            }
        }

        xkb_state.update_mask(current_mods, 0, 0, 0, 0, 0);
        None
    }

    fn is_ctrl_active(&mut self) -> bool {
        let xkb_state = self.xkb_state();
        let ctrl_mod_index = xkb_state.get_keymap().mod_get_index("Control");
        xkb_state.mod_index_is_active(ctrl_mod_index, xkb::STATE_MODS_DEPRESSED)
    }

    fn key(&mut self, key: Keycode, state: KeyDirection);
}
