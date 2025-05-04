use std::sync::Arc;

use base::AppBase;
use winit::{error::EventLoopError, event::WindowEvent};

mod base;
mod context;

pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

pub trait UserApp {
    fn init_renderer(&mut self, gl: Arc<glow::Context>);
    fn ui_layout(&mut self, egui_ctx: &egui::Context);
    fn render(&mut self, gl: Arc<glow::Context>);
    fn handle_event(&mut self, event: WindowEvent);
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
