use std::collections::HashMap;
use zbus::zvariant::OwnedObjectPath;

pub struct ScreenCast {}

impl ScreenCast {
    pub fn try_new(session_handle: &OwnedObjectPath) -> anyhow::Result<Self> {
        let conn = zbus::blocking::Connection::session()?;
        let screencast_proxy = ScreenCastProxyBlocking::new(&conn)?;

        screencast_proxy
            .select_sources(
                &session_handle,
                [("types", 1u32.into()), ("multiple", true.into())].into(),
            )
            .unwrap();

        //screencast_proxy.start(&session_handle, "", HashMap::new())?;

        Ok(Self {})
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.ScreenCast",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
pub trait ScreenCast {
    #[zbus(property)]
    fn available_source_types(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn create_session(
        &self,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn select_sources(
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

    fn open_pipe_wire_remote(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedFd>;
}
