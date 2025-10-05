use super::traits::VirtualPointer;
use crate::{
    Whydotool,
    portal::{remote_desktop::RemoteDesktopProxyBlocking, screencast::ScreenCast},
    virtual_device::pointer::util::Outputs,
};
use pipewire::{
    self as pw,
    core::PW_ID_CORE,
    spa::{pod::Pod, utils::Direction},
    stream::StreamFlags,
};
use pw::properties::properties;
use std::collections::HashMap;
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_pointer};
use zbus::zvariant::OwnedObjectPath;

pub struct PortalPointer {
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
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
    ) -> Self {
        Self {
            outputs: Outputs::new(globals, qh),
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
        let (width, height) = self.outputs.dimensions();

        pw::init();

        let pw_fd = self.screencast.open_pipewire_remote().unwrap();
        let thread_loop =
            unsafe { pw::thread_loop::ThreadLoopRc::new(Some("whydotool"), None).unwrap() };
        let _lock = thread_loop.lock();
        let context = pw::context::ContextRc::new(&thread_loop, None).unwrap();
        let core = context.connect_fd_rc(pw_fd.into(), None).unwrap();

        let stream = pw::stream::StreamRc::new(
            core.clone(),
            "whydotool",
            properties! {
                *pipewire::keys::MEDIA_CLASS => "Video/Source",
                *pipewire::keys::MEDIA_TYPE => "Video",
                *pipewire::keys::MEDIA_CATEGORY => "Capture",
                *pipewire::keys::REMOTE_INTENTION => "screencast",
                *pipewire::keys::APP_NAME => "whydotool",
            },
        )
        .unwrap();

        let _ = stream
            .add_local_listener()
            .process(|stream, _: &mut ()| {
                println!("{:?}", stream.state());
            })
            .register()
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
                Rectangle,
                pw::spa::utils::Rectangle {
                    width: width as u32,
                    height: height as u32
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

        stream
            .connect(
                Direction::Input,
                None,
                StreamFlags::AUTOCONNECT,
                &mut params,
            )
            .unwrap();

        let thread_clone = thread_loop.clone();
        let pending = core.sync(0).expect("sync failed");
        let _listener_core = core
            .add_listener_local()
            .done(move |id, seq| {
                if id == PW_ID_CORE && seq == pending {
                    thread_clone.signal(false);
                }
            })
            .register();

        thread_loop.start();
        thread_loop.wait();

        self.proxy
            .notify_pointer_motion_absolute(
                &self.session_handle,
                HashMap::new(),
                stream.node_id(),
                xpos as f32,
                ypos as f32,
            )
            .unwrap();
    }

    fn outputs(&mut self) -> &mut Outputs {
        &mut self.outputs
    }
}
