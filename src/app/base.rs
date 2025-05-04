use std::sync::Arc;

use egui_glow::EguiGlow;
use egui_winit::winit;
use log::info;
use winit::application::ApplicationHandler;
use winit::event_loop::ActiveEventLoop;

use super::context::{GlutinWindowContext, create_display};

#[derive(Debug)]
pub(super) enum UserEvent {
    Redraw(std::time::Duration),
}

pub type WinitEventLoop = winit::event_loop::EventLoop<UserEvent>;

pub(super) struct AppBase<U: super::UserApp> {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    gl_window: Option<GlutinWindowContext>,
    gl: Option<Arc<glow::Context>>,
    egui_glow: Option<egui_glow::EguiGlow>,
    repaint_delay: std::time::Duration,

    user_app: U,
}

impl<U: super::UserApp> AppBase<U> {
    pub fn new(proxy: winit::event_loop::EventLoopProxy<UserEvent>, user_app: U) -> Self {
        Self {
            proxy,
            gl_window: None,
            gl: None,
            egui_glow: None,
            repaint_delay: std::time::Duration::MAX,
            user_app,
        }
    }

    fn redraw(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.egui_glow
            .as_mut()
            .unwrap()
            .run(self.gl_window.as_mut().unwrap().window(), |egui_ctx| {
                self.user_app.ui_layout(egui_ctx)
            });

        event_loop.set_control_flow(if self.repaint_delay.is_zero() {
            self.gl_window.as_mut().unwrap().window().request_redraw();
            winit::event_loop::ControlFlow::Poll
        } else if let Some(repaint_after_instant) =
            std::time::Instant::now().checked_add(self.repaint_delay)
        {
            winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
        } else {
            winit::event_loop::ControlFlow::Wait
        });

        {
            unsafe {
                use glow::HasContext as _;
                self.gl.as_mut().unwrap().clear_color(1.0, 1.0, 1.0, 1.0);
                self.gl
                    .as_mut()
                    .unwrap()
                    .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
            }

            // draw things behind egui here
            if let Some(ref gl) = self.gl {
                self.user_app.render(gl.clone());
            }

            self.egui_glow
                .as_mut()
                .unwrap()
                .paint(self.gl_window.as_mut().unwrap().window());

            // draw things on top of egui here

            self.gl_window.as_mut().unwrap().swap_buffers().unwrap();
            self.gl_window.as_mut().unwrap().window().set_visible(true);
        }
    }
}

impl<U: super::UserApp> ApplicationHandler<UserEvent> for AppBase<U> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (gl_window, gl) = create_display(event_loop);
        let gl = Arc::new(gl);
        gl_window.window().set_visible(true);

        let egui_glow = EguiGlow::new(event_loop, gl.clone(), None, None, true);

        let event_loop_proxy = egui::mutex::Mutex::new(self.proxy.clone());
        egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::Redraw(info.delay))
                    .expect("Cannot send event");
            });

        self.user_app.init_renderer(gl.clone());

        self.gl_window = Some(gl_window);
        self.gl = Some(gl);
        self.egui_glow = Some(egui_glow);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::event::WindowEvent;
        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
            event_loop.exit();
            return;
        }

        if matches!(event, WindowEvent::RedrawRequested) {
            self.redraw(event_loop);
            return;
        }

        if let WindowEvent::Resized(physical_size) = &event {
            self.gl_window.as_mut().unwrap().resize(*physical_size);
        }

        let event_response = self
            .egui_glow
            .as_mut()
            .unwrap()
            .on_window_event(self.gl_window.as_mut().unwrap().window(), &event);

        if !event_response.consumed {
            self.user_app.handle_event(event);
        }

        if event_response.repaint {
            self.gl_window.as_mut().unwrap().window().request_redraw();
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Redraw(delay) => self.repaint_delay = delay,
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        cause: winit::event::StartCause,
    ) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = &cause {
            self.gl_window.as_mut().unwrap().window().request_redraw();
        }
    }

    fn exiting(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.egui_glow.as_mut().unwrap().destroy();
    }
}
