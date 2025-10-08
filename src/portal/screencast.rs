use super::request;
use anyhow::Context;
use std::collections::HashMap;
use zbus::zvariant::{OwnedFd, OwnedObjectPath};

pub struct ScreenCast {
    proxy: ScreenCastProxyBlocking<'static>,
    session_handle: OwnedObjectPath,
}

impl ScreenCast {
    pub fn try_new(
        conn: &zbus::blocking::Connection,
        request: &mut request::Request,
        session_handle: OwnedObjectPath,
    ) -> anyhow::Result<Self> {
        let screencast_proxy = ScreenCastProxyBlocking::new(conn)?;

        screencast_proxy.select_sources(
            &session_handle,
            [("types", 1u32.into()), ("multiple", true.into())].into(),
        )?;

        request.next_response().context("Response not found")?;

        Ok(Self {
            proxy: screencast_proxy,
            session_handle,
        })
    }

    pub fn open_pipewire_remote(&self) -> anyhow::Result<OwnedFd> {
        Ok(self
            .proxy
            .open_pipe_wire_remote(&self.session_handle, HashMap::new())?)
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

    fn select_sources(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    fn open_pipe_wire_remote(
        &self,
        session_handle: &zbus::zvariant::OwnedObjectPath,
        options: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> zbus::Result<zbus::zvariant::OwnedFd>;
}
