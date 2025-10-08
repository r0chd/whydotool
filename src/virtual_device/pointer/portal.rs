use super::traits::VirtualPointer;
use crate::{Outputs, Whydotool, portal::remote_desktop::RemoteDesktop};
use pipewire::{self as pw, stream::StreamState};
use pw::{properties::properties, spa, spa::pod::Pod};
use wayland_client::{QueueHandle, globals::GlobalList, protocol::wl_pointer};

pub struct PortalPointer {
    outputs: Outputs,
    remote_desktop: RemoteDesktop,
}

impl PortalPointer {
    pub fn new(
        remote_desktop: RemoteDesktop,
        globals: &GlobalList,
        qh: &QueueHandle<Whydotool>,
    ) -> Self {
        Self {
            outputs: Outputs::new(globals, qh),
            remote_desktop,
        }
    }
}

impl VirtualPointer for PortalPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState) {
        self.remote_desktop
            .notify_pointer_button(button as i32, state)
            .unwrap();
    }

    fn scroll(&self, xpos: f64, ypos: f64) {
        self.remote_desktop
            .notify_pointer_axis(xpos as f32, ypos as f32)
            .unwrap();
    }

    fn motion(&self, xpos: f64, ypos: f64) {
        self.remote_desktop
            .notify_pointer_motion(xpos as f32, ypos as f32)
            .unwrap();
    }

    fn motion_absolute(&self, xpos: u32, ypos: u32) {
        let Some(node_id) = self
            .remote_desktop
            .streams()
            .as_ref()
            .and_then(|streams| streams.first().map(|stream| stream.0))
        else {
            return;
        };

        pw::init();

        let pw_fd = self.remote_desktop.open_pipewire_remote().unwrap();
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

        let (width, height) = self.outputs.dimensions();

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
                Id,
                pw::spa::param::video::VideoFormat::RGB
            ),
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::VideoSize,
                Rectangle,
                pw::spa::utils::Rectangle {
                    width: width as u32,
                    height: height as u32,
                }
            ),
            pw::spa::pod::property!(
                pw::spa::param::format::FormatProperties::VideoFramerate,
                Fraction,
                pw::spa::utils::Fraction { num: 25, denom: 1 }
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

        self.remote_desktop
            .notify_pointer_motion_absolute(xpos as f32, ypos as f32, node_id)
            .unwrap();

        mainloop.run();
    }

    fn outputs(&mut self) -> &mut Outputs {
        &mut self.outputs
    }
}
