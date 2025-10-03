use std::collections::HashMap;
use zbus::zvariant::OwnedObjectPath;

pub struct Request {
    stream: ResponseIterator,
    session_handle: OwnedObjectPath,
}

impl Request {
    pub fn try_new(
        conn: &zbus::blocking::Connection,
        session_path: &OwnedObjectPath,
    ) -> anyhow::Result<Self> {
        let proxy = RequestProxyBlocking::builder(conn)
            .path(session_path)?
            .build()?;

        let mut stream = proxy.receive_response()?;
        let session_handle = stream
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

        Ok(Self {
            stream,
            session_handle,
        })
    }

    pub fn get_session_handle(&self) -> OwnedObjectPath {
        self.session_handle.clone()
    }

    pub fn next_response(&mut self) -> Option<Response> {
        self.stream.next()
    }
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.Request",
    default_service = "org.freedesktop.portal.Desktop"
)]
pub trait Request {
    #[zbus(signal)]
    fn response(
        &self,
        response: u32,
        results: HashMap<&str, zbus::zvariant::Value<'_>>,
    ) -> Result<u32>;
}
