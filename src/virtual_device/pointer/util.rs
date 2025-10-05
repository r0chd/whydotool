use crate::Whydotool;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle, globals::GlobalList, protocol::wl_output,
};

#[derive(Clone)]
pub struct Outputs(Vec<Output>);

impl Outputs {
    pub fn new(globals: &GlobalList, qh: &QueueHandle<Whydotool>) -> Self {
        let mut outputs = Vec::new();
        globals.contents().with_list(|list| {
            list.iter()
                .filter(|global| global.interface == wl_output::WlOutput::interface().name)
                .for_each(|global| {
                    let wl_output = globals.registry().bind(global.name, global.version, qh, ());
                    let output = Output::new(wl_output);
                    outputs.push(output);
                });
        });

        Self(outputs)
    }

    pub fn dimensions(&self) -> (i32, i32) {
        self.0.iter().fold((0, 0), |(w, h), output| {
            let output_right = output.x + output.width;
            let output_bottom = output.y + output.height;
            (w.max(output_right), h.max(output_bottom))
        })
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, Output> {
        self.0.iter_mut()
    }
}

#[derive(Clone)]
pub struct Output {
    pub name: Option<Box<str>>,
    wl_output: wl_output::WlOutput,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Output {
    pub const fn new(wl_output: wl_output::WlOutput) -> Self {
        Self {
            name: None,
            wl_output,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for Whydotool {
    fn event(
        state: &mut Self,
        wl_output: &wl_output::WlOutput,
        event: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let Some(output) = state
            .outputs
            .iter_mut()
            .find(|output| output.wl_output == *wl_output)
        {
            match event {
                wl_output::Event::Name { name } => output.name = Some(name.into()),
                wl_output::Event::Geometry { x, y, .. } => {
                    output.x = x;
                    output.y = y;
                }
                wl_output::Event::Mode {
                    flags: _,
                    width,
                    height,
                    refresh: _,
                } => {
                    output.width = width;
                    output.height = height;
                }
                _ => {}
            }
        }
    }
}
