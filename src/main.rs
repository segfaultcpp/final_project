use std::sync::Arc;

use app::{App, UserApp};
use compute::{
    Compute,
    state::Iteration,
    steps::{betweeness::Betweeness, zmax::Zmax},
};
use graph::GraphDesc;
use log::{LevelFilter, Log, SetLoggerError};
use renderer::Renderer;
use simple_logger::SimpleLogger;
use ui::UiState;
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};
use world::{WorldData, camera::CameraMovement};

mod app;
mod compute;
mod graph;
mod renderer;
mod ui;
mod world;

struct LoggerWrapper(SimpleLogger);

impl LoggerWrapper {
    fn init() -> Result<(), SetLoggerError> {
        let simple_logger = SimpleLogger::new();

        log::set_max_level(if cfg!(debug_assertions) {
            LevelFilter::Debug
        } else {
            LevelFilter::Error
        });

        log::set_boxed_logger(Box::new(LoggerWrapper(simple_logger)))
    }
}

impl Log for LoggerWrapper {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.0.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        self.0.log(record)
    }

    fn flush(&self) {
        self.0.flush()
    }
}

pub struct AppState {
    pub compute: Compute,
    pub world: WorldData,
}

impl AppState {
    fn new() -> Self {
        let mut compute = Compute::new(GraphDesc::example())
            .add_step(Zmax)
            .add_step(Betweeness);

        compute.run();

        let world = WorldData::new(&compute.state().at(0).graph.tracker);
        Self { compute, world }
    }
}

struct MyApp {
    renderer: Option<Renderer>,
    app_state: AppState,
    ui_state: UiState,
}

impl MyApp {
    fn init() -> Self {
        Self {
            renderer: None,
            app_state: AppState::new(),
            ui_state: Default::default(),
        }
    }
}

impl UserApp for MyApp {
    fn init_renderer(&mut self, gl: Arc<glow::Context>) {
        self.renderer = Some(Renderer::new(gl.as_ref()));
    }

    fn render(&mut self, gl: Arc<glow::Context>) {
        let gl = gl.as_ref();

        let Some(ref renderer) = self.renderer else {
            return;
        };

        self.app_state
            .world
            .update_materials(self.app_state.compute.state().get());
        renderer.render(gl, &self.app_state);
    }

    fn ui_layout(&mut self, egui_ctx: &egui::Context) {
        self.ui_state.show(egui_ctx, &mut self.app_state);
    }

    fn handle_event(&mut self, event: WindowEvent) {
        if let WindowEvent::KeyboardInput {
            device_id: _,
            event,
            is_synthetic: _,
        } = event
        {
            if let PhysicalKey::Code(code) = event.physical_key {
                if matches!(code, KeyCode::KeyW) {
                    self.app_state
                        .world
                        .camera
                        .process_keyboard(CameraMovement::Forward, 0.166);
                }

                if matches!(code, KeyCode::KeyS) {
                    self.app_state
                        .world
                        .camera
                        .process_keyboard(CameraMovement::Backward, 0.166);
                }

                if matches!(code, KeyCode::KeyA) {
                    self.app_state
                        .world
                        .camera
                        .process_keyboard(CameraMovement::Left, 0.166);
                }

                if matches!(code, KeyCode::KeyD) {
                    self.app_state
                        .world
                        .camera
                        .process_keyboard(CameraMovement::Right, 0.166);
                }

                if matches!(code, KeyCode::ArrowUp) {
                    self.app_state
                        .world
                        .camera
                        .process_keyboard(CameraMovement::Up, 0.166);
                }

                if matches!(code, KeyCode::ArrowDown) {
                    self.app_state
                        .world
                        .camera
                        .process_keyboard(CameraMovement::Down, 0.166);
                }
            }
        }
    }
}

fn main() {
    LoggerWrapper::init().unwrap();

    let app = App::new(MyApp::init());
    app.run().expect("failed to run app");
}
