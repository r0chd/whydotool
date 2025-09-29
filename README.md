# whydotool

A Wayland-native command-line automation tool.

Inspired by [ydotool](https://github.com/ReimuNotMoe/ydotool), it leverages native Wayland protocols to simulate input without requiring low-level kernel access.

# Dependencies

`whydotool` requires a Wayland compositor that supports one or both of the following protocols depending on what you want to do:

- [`wp_virtual_keyboard`](https://wayland.app/protocols/virtual-keyboard-unstable-v1#compositor-support) - required for keyboard-related commands (e.g. `key`, `type`, `stdin`)
- [`wlr_virtual_pointer`](https://wayland.app/protocols/wlr-virtual-pointer-unstable-v1#compositor-support) - required for pointer commands (e.g. `click`, `mousemove`)

# Compatibility

whydotool tries to be fully compatible with ydotool, list of supported ydotool commands:

- [ ] click - Click on mouse buttons
- [x] mousemove - Move mouse pointer to absolute position
- [ ] type - Type a string
- [x] key - Press keys
- [ ] stdin - Sends the key presses as it was a keyboard (i.e from ssh)

# whydotool vs. ydotool

| Feature | whydotool | ydotool |
|---------|-----------|---------|
| **Compatibility** | Wayland only, depends on specific protocols | Runs everywhere |
| **Architecture** | Fully Userspace | Kernelspace |
| **Security Model** | Uses compositor-granted Wayland protocols | Writes directly to uinput |
| **Privileges** | Does not require root | Requires root privileges |
| **Daemon** | Daemonless | Requires a running daemon |
| **Speed** | Slower | Faster (direct kernel-level input injection) |
