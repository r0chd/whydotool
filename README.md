# whydotool

A Wayland-native command-line automation tool.

Inspired by [ydotool](https://github.com/ReimuNotMoe/ydotool), it simulates keyboard and mouse input using native Wayland protocols, no root privileges, no daemons, no kernel hacks.

## Features

- `click` - simulate mouse button presses
- `mousemove` - Move the pointer (relative or absolute)
- `type` - type strings of text
- `key`- press and release individual keys
- `stdin` - stream key events from standard input in real time
- no root required
- no daemon required

## Requirements

`whydotool` works on most major Wayland compositors via either:

- Native Wayland protocols

- `xdg-desktop-portal` RemoteDesktop interface

### Protocol Support

**Keyboard input** (`key`, `type`):
- [`wp_virtual_keyboard`](https://wayland.app/protocols/virtual-keyboard-unstable-v1#compositor-support)

**Pointer input** (`click`, `mousemove`):
-  [`wlr_virtual_pointer`](https://wayland.app/protocols/wlr-virtual-pointer-unstable-v1#compositor-support)

**Universal alternative:**
If your compositor doesn’t support the above protocols, whydotool can use the xdg-desktop-portal RemoteDesktop interface.
See the [list of supported backends](https://wiki.archlinux.org/title/XDG_Desktop_Portal#List_of_backends_and_interfaces)

If your compositor doesn't support the specific protocols above, it will likely work through the portal interface. Check the linked compatibility tables to verify support for your compositor.

## Optional Dependency

`portal` (disabled by default)
The `portal` backend enables input injection via the `xdg-desktop-portal` RemoteDesktop interface.
Use this if your compositor lacks native virtual input protocol support.

Build with:

```
cargo build --features portal
```

This increases compatibility with desktop environments like GNOME (doesn't support neither of protocols) and KDE Plasma (doesn't support virtual-keyboard protocol)

## Examples

Type text:

```
whydotool type "Hello Wayland"
```

Press a key:

```
whydotool key 56:1 62:1 62:0 56:0
```

Relatively move mouse pointer by -100,100:

```
whydotool mousemove -x -100 -y 100
```

Move mouse pointer to 100,100:

```
whydotool mousemove --absolute -x 100 -y 100
```

Mouse right click:

```
whydotool click 0xC1
```

Mouse repeating left click:

```
whydotool click --repeat 5 --next-delay 25 0xC0
```

## whydotool vs. ydotool

| Feature | whydotool | ydotool |
|---------|-----------|---------|
| **Compatibility** | Wayland only | Runs everywhere |
| **Security Model** | Uses compositor-granted Wayland protocols or xdg-desktop-portal | Writes directly to uinput |
| **Privileges** | Does not require root | Requires root privileges |
| **Daemon** | Daemonless | Requires a running daemon |
