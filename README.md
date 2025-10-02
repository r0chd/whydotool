# whydotool
A Wayland-native command-line automation tool.

Inspired by [ydotool](https://github.com/ReimuNotMoe/ydotool), it leverages native Wayland protocols to simulate input without requiring low-level kernel access.

## Requirements

`whydotool` works with most major Wayland compositors through either direct protocol support or the `xdg-desktop-portal` RemoteDesktop interface.

### Protocol Support

**For keyboard commands** (`key`, `type`, `stdin`):
- `wp_virtual_keyboard` protocol ([compositor support](https://wayland.app/protocols/virtual-keyboard-unstable-v1#compositor-support))

**For pointer commands** (`click`, `mousemove`):
- `wlr_virtual_pointer` protocol ([compositor support](https://wayland.app/protocols/wlr-virtual-pointer-unstable-v1#compositor-support))

**Universal alternative:**
- `xdg-desktop-portal` with RemoteDesktop interface ([compositor support](https://wiki.archlinux.org/title/XDG_Desktop_Portal#List_of_backends_and_interfaces)) - supported by all major desktop compositors (GNOME, KDE Plasma, etc.)

If your compositor doesn't support the specific protocols above, it will likely work through the portal interface. Check the linked compatibility tables to verify support for your compositor.

## Compatibility

whydotool aims to be fully compatible with ydotool. Currently supported commands:

- [x] click - Click on mouse buttons
- [x] mousemove - Move mouse pointer to relative or absolute position (absolute position is protocol only atm)
- [x] type - Type a string
- [x] key - Press keys

## whydotool vs. ydotool

| Feature | whydotool | ydotool |
|---------|-----------|---------|
| **Compatibility** | Wayland only | Runs everywhere |
| **Architecture** | Fully Userspace | Kernelspace |
| **Security Model** | Uses compositor-granted Wayland protocols or xdg-desktop-portal | Writes directly to uinput |
| **Privileges** | Does not require root | Requires root privileges |
| **Daemon** | Daemonless | Requires a running daemon |
| **Speed** | Slower | Faster (direct kernel-level input injection) |
