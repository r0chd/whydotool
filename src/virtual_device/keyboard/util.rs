use std::{ffi::CString, fs, io::Write, path::PathBuf};
use xkbcommon::xkb::{self, KEYMAP_COMPILE_NO_FLAGS, KEYMAP_FORMAT_TEXT_V1};

pub fn xkb_init() -> (xkb::State, fs::File, u32) {
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
