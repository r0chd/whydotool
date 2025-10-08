use super::traits::VirtualPointer;
use crate::{
    Outputs, Whydotool,
    portal::{remote_desktop::RemoteDesktopProxyBlocking, screencast::ScreenCast},
};
use pipewire::{self as pw, stream::StreamState};
use pw::{properties::properties, spa, spa::pod::Pod};
use std::collections::HashMap;
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_pointer};
use zbus::zvariant;
use zbus::zvariant::OwnedObjectPath;

pub struct PortalPointer {
    streams: Option<Vec<(u32, HashMap<String, zvariant::OwnedValue>)>>,
    outputs: Outputs,
    proxy: RemoteDesktopProxyBlocking<'static>,
    session_handle: OwnedObjectPath,
    screencast: ScreenCast,
}

impl PortalPointer {
    pub fn new(
        proxy: RemoteDesktopProxyBlocking<'static>,
        session_handle: OwnedObjectPath,
        screencast: ScreenCast,
        streams: Option<Vec<(u32, HashMap<String, zvariant::OwnedValue>)>>,
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
    ) -> Self {
        Self {
            outputs: Outputs::new(globals, qh),
            streams,
            proxy,
            session_handle,
            screencast,
        }
    }
}

impl VirtualPointer for PortalPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState) {
        self.proxy
            .notify_pointer_button(
                &self.session_handle,
                HashMap::new(),
                button as i32,
                u32::from(state != wl_pointer::ButtonState::Released),
            )
            .unwrap();
    }

    fn scroll(&self, xpos: f64, ypos: f64) {
        self.proxy
            .notify_pointer_axis(
                &self.session_handle,
                HashMap::new(),
                xpos as f32,
                ypos as f32,
            )
            .unwrap();
    }

    fn motion(&self, xpos: f64, ypos: f64) {
        self.proxy
            .notify_pointer_motion(
                &self.session_handle,
                HashMap::new(),
                xpos as f32,
                ypos as f32,
            )
            .unwrap();
    }

    fn motion_absolute(&self, xpos: u32, ypos: u32) {
        let Some(node_id) = self
            .streams
            .as_ref()
            .map(|streams| streams.first().map(|stream| stream.0))
            .flatten()
        else {
            return;
        };

        pw::init();

        let pw_fd = self.screencast.open_pipewire_remote().unwrap();
        let mainloop = pw::main_loop::MainLoopRc::new(None).unwrap();
        let context = pw::context::ContextRc::new(&mainloop, None).unwrap();
        let core = context.connect_fd_rc(pw_fd.into(), None).unwrap();

        let stream = pw::stream::StreamRc::new(
            core,
            "whydotool",
            properties! {
                *pipewire::keys::MEDIA_TYPE => "Video",
                *pipewire::keys::MEDIA_CATEGORY => "Capture",
                *pipewire::keys::MEDIA_ROLE => "Screen",
            },
        )
        .unwrap();

        let obj = pw::spa::pod::object!(
            pw::spa::utils::SpaTypes::ObjectParamFormat,
            pw::spa::param::ParamType::EnumFormat,
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::MediaType,
                Id,
                pw::spa::param::format::MediaType::Video
            ),
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::MediaSubtype,
                Id,
                pw::spa::param::format::MediaSubtype::Raw
            ),
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::VideoFormat,
                Choice,
                Enum,
                Id,
                pw::spa::param::video::VideoFormat::RGB,
                pw::spa::param::video::VideoFormat::RGB,
                pw::spa::param::video::VideoFormat::RGBA,
                pw::spa::param::video::VideoFormat::RGBx,
                pw::spa::param::video::VideoFormat::BGRx,
                pw::spa::param::video::VideoFormat::YUY2,
                pw::spa::param::video::VideoFormat::I420,
            ),
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::VideoSize,
                Choice,
                Range,
                Rectangle,
                pw::spa::utils::Rectangle {
                    width: 320,
                    height: 240
                },
                pw::spa::utils::Rectangle {
                    width: 1,
                    height: 1
                },
                pw::spa::utils::Rectangle {
                    width: 4096,
                    height: 4096
                }
            ),
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::VideoFramerate,
                Choice,
                Range,
                Fraction,
                pw::spa::utils::Fraction { num: 25, denom: 1 },
                pw::spa::utils::Fraction { num: 0, denom: 1 },
                pw::spa::utils::Fraction {
                    num: 1000,
                    denom: 1
                }
            ),
        );
        let values: Vec<u8> = pw::spa::pod::serialize::PodSerializer::serialize(
            std::io::Cursor::new(Vec::new()),
            &pw::spa::pod::Value::Object(obj),
        )
        .unwrap()
        .0
        .into_inner();

        let mut params = [Pod::from_bytes(&values).unwrap()];

        let mainloop_ref = mainloop.clone();
        let _listener = stream
            .add_local_listener()
            .state_changed(move |_, _: &mut (), _, new| {
                if new == StreamState::Streaming {
                    mainloop_ref.quit();
                }
            })
            .register();

        stream
            .connect(
                spa::utils::Direction::Input,
                Some(node_id),
                pw::stream::StreamFlags::AUTOCONNECT | pw::stream::StreamFlags::MAP_BUFFERS,
                &mut params,
            )
            .unwrap();

        self.proxy
            .notify_pointer_motion_absolute(
                &self.session_handle,
                HashMap::new(),
                node_id,
                xpos as f32,
                ypos as f32,
            )
            .unwrap();

        mainloop.run();
    }

    fn outputs(&mut self) -> &mut Outputs {
        &mut self.outputs
    }
}
