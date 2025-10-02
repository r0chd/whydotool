use xkbcommon::xkb::{self, keysyms::KEY_Shift_L};

pub trait VirtualKeyboard {
    fn xkb_state(&mut self) -> &mut xkb::State;

    fn keycode_from_char(&mut self, c: char) -> Option<(u32, bool)> {
        for keycode in 8..=255 {
            let xkb_keycode = xkb::Keycode::new(keycode);

            if self.xkb_state().key_get_one_sym(xkb_keycode) == xkb::utf32_to_keysym(c as u32) {
                return Some((keycode - 8, false));
            }

            self.xkb_state().update_mask(0, KEY_Shift_L, 0, 0, 0, 0);
            if self.xkb_state().key_get_one_sym(xkb_keycode) == xkb::utf32_to_keysym(c as u32) {
                self.xkb_state().update_mask(0, 0, 0, 0, 0, 0);
                return Some((keycode - 8, true));
            }

            self.xkb_state().update_mask(0, 0, 0, 0, 0, 0);
        }

        None
    }

    fn key(&mut self, key: u32, state: u32);
}
