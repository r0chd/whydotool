use super::traits::VirtualPointer;
use crate::portal::remote_desktop::RemoteDesktop;
use anyhow::Context;
use pipewire as pw;
use pw::{context, main_loop, properties::properties, spa, stream::StreamState};
use std::process;
use wayland_client::protocol::wl_pointer;

pub struct PortalPointer {}

impl PortalPointer {
    pub fn new() -> Self {
        Self {}
    }

    fn motion_absolute_impl(
        &self,
        xpos: u32,
        ypos: u32,
        remote_desktop: RemoteDesktop,
        node_id: u32,
    ) -> anyhow::Result<()> {
        pw::init();

        let pw_fd = remote_desktop
            .open_pipewire_remote()
            .context("Failed to open PipeWire remote")?;

        let mainloop =
            main_loop::MainLoopRc::new(None).context("Failed to create PipeWire main loop")?;

        let context = context::ContextRc::new(&mainloop, None)
            .context("Failed to create PipeWire context")?;

        let core = context
            .connect_fd_rc(pw_fd.into(), None)
            .context("Failed to connect PipeWire core")?;

        let stream = pw::stream::StreamRc::new(
            core,
            "whydotool",
            properties! {
                *pipewire::keys::MEDIA_TYPE => "Video",
                *pipewire::keys::MEDIA_CATEGORY => "Capture",
                *pipewire::keys::MEDIA_ROLE => "Screen",
            },
        )
        .context("Failed to create PipeWire stream")?;

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
                &mut [],
            )
            .context("Failed to connect PipeWire stream")?;

        remote_desktop
            .notify_pointer_motion_absolute(xpos as f32, ypos as f32, node_id)
            .context("Failed to notify pointer motion absolute")?;

        mainloop.run();
        Ok(())
    }
}

impl VirtualPointer for PortalPointer {
    fn button(&self, button: u32, state: wl_pointer::ButtonState) -> anyhow::Result<()> {
        let remote_desktop = RemoteDesktop::builder().pointer(true).try_build()?;

        remote_desktop.notify_pointer_button(button as i32, state)?;

        Ok(())
    }

    fn scroll(&self, xpos: f64, ypos: f64) -> anyhow::Result<()> {
        let remote_desktop = RemoteDesktop::builder().pointer(true).try_build()?;

        remote_desktop.notify_pointer_axis(xpos as f32, ypos as f32)?;

        Ok(())
    }

    fn motion(&self, xpos: f64, ypos: f64) -> anyhow::Result<()> {
        let remote_desktop = RemoteDesktop::builder().pointer(true).try_build()?;

        remote_desktop.notify_pointer_motion(xpos as f32, ypos as f32)?;

        Ok(())
    }

    fn motion_absolute(&self, xpos: u32, ypos: u32) -> anyhow::Result<()> {
        let remote_desktop = RemoteDesktop::builder()
            .pointer(true)
            .screencast(true)
            .try_build()?;

        if let Some(node_id) = remote_desktop
            .streams()
            .as_ref()
            .and_then(|streams| streams.first().map(|stream| stream.0))
        {
            if let Err(e) = self.motion_absolute_impl(xpos, ypos, remote_desktop, node_id) {
                eprintln!("motion_absolute failed: {e:#}");
                process::exit(1);
            }
        } else {
            eprintln!("No PipeWire node found for pointer motion_absolute");
            process::exit(1);
        }

        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
