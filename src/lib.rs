mod output;
#[cfg(feature = "portals")]
mod portal;
mod virtual_device;

use output::Outputs;
#[cfg(feature = "portals")]
use portal::remote_desktop::RemoteDesktop;
use std::fmt;
use std::sync::{Arc, Mutex};
#[cfg(feature = "portals")]
use virtual_device::{keyboard::portal::PortalKeyboard, pointer::portal::PortalPointer};
use virtual_device::{
    keyboard::{traits::VirtualKeyboard, wayland::WaylandKeyboard},
    pointer::{traits::VirtualPointer, wayland::WaylandPointer},
};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, delegate_dispatch, delegate_noop,
    globals::{GlobalList, GlobalListContents, registry_queue_init},
    protocol::{wl_keyboard, wl_registry, wl_seat},
};
use wayland_protocols_wlr::virtual_pointer::v1::client::{
    zwlr_virtual_pointer_manager_v1, zwlr_virtual_pointer_v1,
};
use xkbcommon::xkb::KeyDirection;

pub struct KeymapInfo {
    pub format: wl_keyboard::KeymapFormat,
    pub fd: std::os::fd::OwnedFd,
    pub size: u32,
}

pub struct KeyPress {
    pub keycode: u32,
    pub pressed: KeyDirection,
}

impl fmt::Debug for KeyPress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyPress")
            .field("keycode", &self.keycode)
            .field("pressed", &"<KeyDirection>")
            .finish()
    }
}

impl Clone for KeyPress {
    fn clone(&self) -> Self {
        Self {
            keycode: self.keycode,
            pressed: match self.pressed {
                KeyDirection::Up => KeyDirection::Up,
                KeyDirection::Down => KeyDirection::Down,
            },
        }
    }
}

struct State {
    outputs: Outputs,
    key_delay: i32,
    keymap_info: Arc<Mutex<Option<KeymapInfo>>>,
}

pub struct Whydotool {
    seat: Option<wl_seat::WlSeat>,
    event_queue: EventQueue<State>,
    state: State,
    force_portal: bool,
    globals: GlobalList,
    qh: QueueHandle<State>,
    _wl_keyboard: Option<wl_keyboard::WlKeyboard>,
}

impl Whydotool {
    #[cfg(feature = "portals")]
    /// # Errors
    ///
    /// Connection to wayland socket failed
    pub fn try_new() -> anyhow::Result<Self> {
        let conn = Connection::connect_to_env()?;
        let (globals, mut event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=4, ()).ok();
        let wl_keyboard = seat.as_ref().map(|seat| seat.get_keyboard(&qh, ()));

        let mut state = State {
            key_delay: 0,
            outputs: Outputs::new(&globals, &qh),
            keymap_info: Arc::new(Mutex::new(None)),
        };

        event_queue.dispatch_pending(&mut state)?;
        event_queue.roundtrip(&mut state)?;

        Ok(Self {
            seat,
            state,
            force_portal: false,
            globals,
            qh,
            _wl_keyboard: wl_keyboard,
            event_queue,
        })
    }

    #[cfg(not(feature = "portals"))]
    pub fn try_new() -> anyhow::Result<Self> {
        let conn = Connection::connect_to_env()?;
        let (globals, mut event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let seat = globals.bind::<wl_seat::WlSeat, _, _>(&qh, 1..=4, ()).ok();
        let wl_keyboard = seat.as_ref().map(|seat| seat.get_keyboard(&qh, ()));

        let mut state = State {
            key_delay: 0,
            outputs: Outputs::new(&globals, &qh),
            keymap_info: Arc::new(Mutex::new(None)),
        };

        event_queue.dispatch_pending(&mut state)?;
        event_queue.roundtrip(&mut state)?;

        Ok(Self {
            force_portal: false,
            globals,
            qh,
            seat,
            state,
            _wl_keyboard: wl_keyboard,
            event_queue,
        })
    }

    #[must_use]
    pub fn key_delay(&self) -> i32 {
        self.state.key_delay
    }

    pub fn roundtrip(&mut self) -> anyhow::Result<usize> {
        self.event_queue
            .roundtrip(&mut self.state)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    pub fn force_portal(&mut self, force_portal: bool) {
        self.force_portal = force_portal;
    }

    #[cfg(feature = "portals")]
    /// # Errors
    /// Lack of virtual keyboard support in compositor
    /// Lack of `RemoteDesktop` interface support in xdg-desktop-portal
    pub fn virtual_keyboard(&self) -> anyhow::Result<Box<dyn VirtualKeyboard>> {
        let keymap_guard = self
            .state
            .keymap_info
            .lock()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        let Some(keymap_info) = keymap_guard.as_ref() else {
            return Err(anyhow::anyhow!("something went horribly wrong"));
        };

        if !self.force_portal
            && let Some(seat) = self.seat.as_ref()
            && let Ok(ptr) = WaylandKeyboard::try_new(&self.globals, &self.qh, seat, keymap_info)
        {
            return Ok(Box::new(ptr));
        }

        let remote_desktop = RemoteDesktop::builder().keyboard(true).try_build()?;
        Ok(Box::new(PortalKeyboard::try_new(
            remote_desktop,
            keymap_info,
        )?))
    }

    #[cfg(not(feature = "portals"))]
    /// # Errors
    ///
    /// No seat was found
    /// Keymap information was unavailable
    pub fn virtual_keyboard(&self) -> anyhow::Result<Box<dyn VirtualKeyboard>> {
        let Some(seat) = self.state.seat.as_ref() else {
            return Err(anyhow::anyhow!("No seat provided for Wayland keyboard"));
        };

        let keymap_guard = self.state.keymap_info.lock().unwrap();
        let Some(keymap_info) = keymap_guard.as_ref() else {
            return Err(anyhow::anyhow!(
                "No keymap information available. Make sure a keyboard is connected and the keymap event has been received."
            ));
        };
        Ok(Box::new(WaylandKeyboard::try_new(
            &self.globals,
            &self.qh,
            seat,
            keymap_info,
        )?))
    }

    #[cfg(feature = "portals")]
    /// # Errors
    ///
    /// Lack of support for virtual pointer protocol in compositor
    /// Lack of `RemoteDesktop` interface support in xdg-desktop-portal
    pub fn virtual_pointer(&self) -> anyhow::Result<Box<dyn VirtualPointer>> {
        if !self.force_portal
            && let Ok(ptr) = WaylandPointer::try_new(
                &self.globals,
                &self.qh,
                self.seat.as_ref(),
                self.state.outputs.clone(),
            )
        {
            return Ok(Box::new(ptr));
        }

        let remote_desktop = RemoteDesktop::builder()
            .pointer(true)
            .screencast(true)
            .try_build()?;

        let portal_ptr = PortalPointer::new(remote_desktop);
        Ok(Box::new(portal_ptr))
    }

    #[cfg(not(feature = "portals"))]
    /// # Errors
    ///
    /// Lack of support for virtual pointer protocol in compositor
    pub fn virtual_pointer(&self) -> anyhow::Result<Box<dyn VirtualPointer>> {
        Ok(Box::new(WaylandPointer::try_new(
            &self.globals,
            &self.qh,
            self.seat.as_ref(),
            self.state.outputs.clone(),
        )?))
    }
}

impl Dispatch<wl_keyboard::WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &wl_keyboard::WlKeyboard,
        event: <wl_keyboard::WlKeyboard as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Keymap { format, fd, size } => {
                let keymap_info = KeymapInfo {
                    format: format.into_result().unwrap(),
                    fd,
                    size,
                };
                if let Ok(mut keymap_guard) = state.keymap_info.lock() {
                    *keymap_guard = Some(keymap_info);
                }
            }
            wl_keyboard::Event::RepeatInfo { rate, delay: _ } => {
                state.key_delay = ((1.0 / rate as f32) * 1000.) as i32;
            }
            _ => {}
        }
    }
}

delegate_noop!(State: ignore wl_seat::WlSeat);
delegate_dispatch!(State: [wl_registry::WlRegistry: GlobalListContents] => State);
delegate_noop!(State: zwlr_virtual_pointer_manager_v1::ZwlrVirtualPointerManagerV1);
delegate_noop!(State: zwlr_virtual_pointer_v1::ZwlrVirtualPointerV1);
