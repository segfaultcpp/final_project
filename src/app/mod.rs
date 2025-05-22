use std::{sync::Arc, time::Duration};

use base::AppBase;
use egui_glow::Painter;
use winit::{
    error::EventLoopError,
    event::{DeviceEvent, WindowEvent},
};

mod base;
mod context;

pub const WINDOW_WIDTH: u32 = 1920;
pub const WINDOW_HEIGHT: u32 = 1080;

pub trait UserApp {
    fn initialize(&mut self, gl: Arc<glow::Context>, painter: &mut Painter);
    fn ui_layout(&mut self, egui_ctx: &egui::Context);
    fn update(&mut self, delta: Duration);
    fn render(&mut self, gl: Arc<glow::Context>);
    fn handle_window_events(&mut self, event: WindowEvent);
    fn handle_device_events(&mut self, event: DeviceEvent);
}

pub struct App<U: UserApp> {
    base: AppBase<U>,
    event_loop: base::WinitEventLoop,
}

impl<U: UserApp> App<U> {
    pub fn new(app: U) -> Self {
        let event_loop = base::WinitEventLoop::with_user_event().build().unwrap();

        let proxy = event_loop.create_proxy();
        let base = AppBase::new(proxy, app);

        Self { base, event_loop }
    }

    pub fn run(mut self) -> Result<(), EventLoopError> {
        let event_loop = self.event_loop;
        event_loop.run_app(&mut self.base)
    }
}
