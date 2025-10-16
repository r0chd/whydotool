use crate::portal::screencast::ScreenCast;

use super::{request, screencast, util};
use anyhow::Context;
use std::collections::HashMap;
use wayland_client::protocol::wl_pointer;
use xkbcommon::xkb::{KeyDirection, Keycode};
use zbus::zvariant::{self, OwnedFd};

#[derive(Default)]
pub struct RemoteDesktopBuilder {
    enable_keyboard: bool,
    enable_pointer: bool,
    enable_screencast: bool,
}

impl RemoteDesktopBuilder {
    fn new() -> Self {
        Self::default()
    }

    pub fn keyboard(mut self, enable: bool) -> Self {
        self.enable_keyboard = enable;
        self
    }

    pub fn pointer(mut self, enable: bool) -> Self {
        self.enable_pointer = enable;
        self
    }

    pub fn screencast(mut self, enable: bool) -> Self {
        self.enable_screencast = enable;
        self
    }

    pub fn try_build(self) -> anyhow::Result<RemoteDesktop> {
        let conn = zbus::blocking::Connection::session()?;
        let remote_desktop_proxy = RemoteDesktopProxyBlocking::new(&conn)?;

        let device_types = remote_desktop_proxy.available_device_types()?;

        let keyboard_supported = (device_types & 1) != 0;
        let pointer_supported = (device_types & 2) != 0;

        let session_token = util::SessionToken::default();

        let session_path = remote_desktop_proxy
            .create_session([("session_handle_token", session_token.into())].into())?;

        let mut request = request::Request::try_new(&conn, &session_path)?;
        let session_handle = request.get_session_handle();

        let mut selected_device_mask: u32 = 0;
        if self.enable_keyboard && keyboard_supported {
            selected_device_mask |= 1;
        }
        if self.enable_pointer && pointer_supported {
            selected_device_mask |= 2;
        }

        remote_desktop_proxy.select_devices(
            &session_handle,
            [("types", selected_device_mask.into())].into(),
        )?;

        request.next_response().unwrap().args()?;

        let screencast = if self.enable_screencast {
            let screencast =
                screencast::ScreenCast::try_new(&conn, &mut request, session_handle.clone())?;

            Some(screencast)
        } else {
            None
        };

        remote_desktop_proxy.start(
            &session_handle,
            "",
            [("devices", selected_device_mask.into())].into(),
        )?;

        let res = request.next_response().unwrap();
        let args = res.args()?;

        let streams: Option<Vec<(u32, HashMap<String, zvariant::OwnedValue>)>> = args
            .results()
            .get("streams")
            .and_then(|v| v.to_owned().try_into().ok());

        if args.response == 0 {
            Ok(RemoteDesktop {
                streams,
                screencast,
                session_handle,
                proxy: remote_desktop_proxy,
            })
        } else {
            anyhow::bail!("Remote desktop session start was rejected by the system")
        }
    }
}

pub struct RemoteDesktop {
    streams: Option<Vec<(u32, HashMap<String, zvariant::OwnedValue>)>>,
    screencast: Option<screencast::ScreenCast>,
    session_handle: zbus::zvariant::OwnedObjectPath,
    proxy: RemoteDesktopProxyBlocking<'static>,
}

impl RemoteDesktop {
    pub fn builder() -> RemoteDesktopBuilder {
        RemoteDesktopBuilder::new()
    }

    pub fn streams(&self) -> Option<&Vec<(u32, HashMap<String, zvariant::OwnedValue>)>> {
        self.streams.as_ref()
    }

    pub fn notify_keyboard_keycode(
        &self,
        key: Keycode,
        state: &KeyDirection,
    ) -> anyhow::Result<()> {
        let raw_state = match state {
            KeyDirection::Down => 1,
            KeyDirection::Up => 0,
        };

        self.proxy.notify_keyboard_keycode(
            &self.session_handle,
            HashMap::new(),
            key.raw() as i32 - 8,
            raw_state,
        )?;

        Ok(())
    }

    pub fn notify_pointer_button(
        &self,
        button: i32,
        state: wl_pointer::ButtonState,
    ) -> anyhow::Result<()> {
        self.proxy.notify_pointer_button(
            &self.session_handle,
            HashMap::new(),
            button,
            u32::from(state != wl_pointer::ButtonState::Released),
        )?;

        Ok(())
    }

    pub fn notify_pointer_axis(&self, xpos: f32, ypos: f32) -> anyhow::Result<()> {
        self.proxy
            .notify_pointer_axis(&self.session_handle, HashMap::new(), xpos, ypos)?;

        Ok(())
    }

    pub fn notify_pointer_motion(&self, xpos: f32, ypos: f32) -> anyhow::Result<()> {
        self.proxy
            .notify_pointer_motion(&self.session_handle, HashMap::new(), xpos, ypos)?;

        Ok(())
    }

    pub fn notify_pointer_motion_absolute(
        &self,
        xpos: f32,
        ypos: f32,
        node_id: u32,
    ) -> anyhow::Result<()> {
        self.proxy.notify_pointer_motion_absolute(
            &self.session_handle,
            HashMap::new(),
            node_id,
            xpos,
            ypos,
        )?;

        Ok(())
    }

    pub fn open_pipewire_remote(&self) -> anyhow::Result<OwnedFd> {
        self.screencast
            .as_ref()
            .map(ScreenCast::open_pipewire_remote)
            .context("")
            .flatten()
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.RemoteDesktop",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
pub trait RemoteDesktop {
    #[zbus(property)]
    fn available_device_types(&self) -> zbus::Result<u32>;

    fn create_session(
        &self,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn select_devices(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn start(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        parent_window: &str,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn notify_pointer_motion(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        dx: f32,
        dy: f32,
    ) -> zbus::Result<()>;

    fn notify_pointer_motion_absolute(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        stream: u32,
        dx: f32,
        dy: f32,
    ) -> zbus::Result<()>;

    fn notify_pointer_button(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        button: i32,
        state: u32,
    ) -> zbus::Result<()>;

    fn notify_pointer_axis(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        dx: f32,
        dy: f32,
    ) -> zbus::Result<()>;

    fn notify_keyboard_keycode(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        keycode: i32,
        state: u32,
    ) -> zbus::Result<()>;
}
