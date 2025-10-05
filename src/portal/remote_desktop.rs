use super::{request, screencast, util};
use std::collections::HashMap;

pub struct RemoteDesktop {
    pub screencast: Option<screencast::ScreenCast>,
    pub session_handle: zbus::zvariant::OwnedObjectPath,
    pub proxy: RemoteDesktopProxyBlocking<'static>,
}

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
            .create_session([("session_handle_token", session_token.into())].into())
            .unwrap();

        let mut request = request::Request::try_new(&conn, &session_path)?;
        let session_handle = request.get_session_handle();

        let mut selected_device_mask = 0;
        if self.enable_keyboard && keyboard_supported {
            selected_device_mask |= 1;
        }
        if self.enable_pointer && pointer_supported {
            selected_device_mask |= 2;
        }

        remote_desktop_proxy.select_devices(
            &session_handle,
            [("devices", selected_device_mask.into())].into(),
        )?;

        request.next_response().unwrap().args()?;

        let screencast = if self.enable_screencast {
            Some(screencast::ScreenCast::try_new(
                &conn,
                &mut request,
                session_handle.clone(),
            )?)
        } else {
            None
        };

        remote_desktop_proxy.start(
            &session_handle,
            "",
            [("devices", selected_device_mask.into())].into(),
        )?;

        let res = request.next_response().unwrap();

        if res.args()?.response == 0 {
            Ok(RemoteDesktop {
                screencast,
                session_handle,
                proxy: remote_desktop_proxy,
            })
        } else {
            anyhow::bail!("Remote desktop session start was rejected by the system")
        }
    }
}

impl RemoteDesktop {
    pub fn builder() -> RemoteDesktopBuilder {
        RemoteDesktopBuilder::new()
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
