use std::collections::HashMap;

#[zbus::proxy(
    interface = "org.freedesktop.portal.RemoteDesktop",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait RemoteDesktop {
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

    fn notify_keyboard_keycode(
        &self,
        session_handle: zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
        keycode: i32,
        state: u32,
    ) -> zbus::Result<()>;
}
