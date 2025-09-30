use rand::{Rng, distr::Alphanumeric};
use std::collections::HashMap;

pub struct RemoteDesktop {
    pub session_handle: zbus::zvariant::OwnedObjectPath,
    pub proxy: RemoteDesktopProxyBlocking<'static>,
}

impl RemoteDesktop {
    pub fn try_new() -> anyhow::Result<Self> {
        let conn = zbus::blocking::Connection::session()?;
        let remote_desktop_proxy = RemoteDesktopProxyBlocking::new(&conn)?;

        let device_types = remote_desktop_proxy.available_device_types()?;

        let keyboard_supported = (device_types & 1) != 0;
        let pointer_supported = (device_types & 2) != 0;

        let token: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let session_token = format!("whydotool_{token}");

        let session_path = {
            let session_options = [("session_handle_token", session_token.clone().into())];
            remote_desktop_proxy
                .create_session(session_options.into())
                .unwrap()
        };

        let request_proxy = RequestProxyBlocking::builder(&conn)
            .path(&session_path)?
            .build()?;

        let mut stream = request_proxy.receive_response()?;

        let session_handle = request_proxy
            .receive_response()?
            .next()
            .unwrap()
            .args()
            .ok()
            .and_then(|response| response.results.get("session_handle").cloned())
            .and_then(|value| {
                let zbus::zvariant::Value::Str(s) = value else {
                    unreachable!()
                };
                zbus::zvariant::OwnedObjectPath::try_from(s.as_str()).ok()
            })
            .unwrap();

        let selected_device_mask = if keyboard_supported && pointer_supported {
            1 | 2
        } else if keyboard_supported {
            1
        } else if pointer_supported {
            2
        } else {
            0
        };

        stream.next().unwrap().args()?;

        let devices_options = [("devices", selected_device_mask.into())];
        remote_desktop_proxy.select_devices(session_handle.clone(), devices_options.into())?;

        stream.next().unwrap().args()?;

        let start_options = [("devices", selected_device_mask.into())];
        remote_desktop_proxy.start(session_handle.clone(), "", start_options.into())?;

        let res = stream.next().unwrap().args()?.response;

        if res == 0 {
            Ok(Self {
                session_handle,
                proxy: remote_desktop_proxy,
            })
        } else {
            anyhow::bail!("Remote desktop session start was rejected by the system")
        }
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
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn start(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        parent_window: &str,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn notify_pointer_motion(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        dx: f32,
        dy: f32,
    ) -> zbus::Result<()>;

    fn notify_pointer_motion_absolute(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        stream: u32,
        dx: f32,
        dy: f32,
    ) -> zbus::Result<()>;

    fn notify_pointer_button(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        button: i32,
        state: u32,
    ) -> zbus::Result<()>;

    fn notify_pointer_axis(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        dx: f32,
        dy: f32,
    ) -> zbus::Result<()>;

    fn notify_keyboard_keysym(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        keycode: i32,
        state: u32,
    ) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.Request",
    default_service = "org.freedesktop.portal.Desktop"
)]
trait Request {
    #[zbus(signal)]
    fn response(
        &self,
        response: u32,
        results: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> Result<u32>;
}
