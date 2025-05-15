use std::{sync::Arc, time::Duration};

use app::{App, UserApp};
use compute::{
    Compute, CopyIteration, UpdatePaths,
    state::Iteration,
    steps::{
        betweeness::Betweeness,
        capacity::Capacity,
        delete::{DeleteMaxBetweenness, DeleteOverloaded},
        zmax::Zmax,
    },
};
use graph::{GraphDesc, NodeDesc, node::Node};
use input::{Input, Key};
use log::{LevelFilter, Log, SetLoggerError, info};
use renderer::Renderer;
use simple_logger::SimpleLogger;
use ui::UiTabManager;
use ui_legacy::UiState;
use winit::{
    event::{DeviceEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use world::{WorldData, camera::CameraMovement};

mod app;
mod compute;
mod graph;
mod input;
mod renderer;
mod ui;
mod ui_legacy;
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

#[derive(Default, Clone)]
pub enum AppMode {
    #[default]
    Editor,
    RunResult, // TODO:
}

pub struct AppState {
    pub mode: AppMode,
    pub compute: Compute,
    pub world: WorldData,
    pub selected_node: Option<Node>,
}

impl Drop for AppState {
    fn drop(&mut self) {
        let Iteration { graph, .. } = self.compute.state().at(0);

        let mut nodes = vec![];
        for i in graph.tracker.iter_alive() {
            nodes.push(NodeDesc {
                node_id: i.as_idx() as u32,
                nodes: {
                    let mut nodes = vec![];

                    for j in graph.tracker.iter_alive().exclude(i) {
                        if graph.is_adjacent(i, j) {
                            nodes.push(j.as_idx() as u32);
                        }
                    }

                    nodes
                },
                position: self.world.positions[i].0.into(),
            })
        }

        let desc = GraphDesc {
            alpha: self.compute.state().alpha,
            nodes,
        };

        let desc = toml::to_string(&desc).unwrap();

        std::fs::write("data/graph_desc.toml", desc.as_str()).unwrap();
    }
}

impl AppState {
    fn new() -> Self {
        let desc = std::fs::read_to_string("data/graph_desc.toml").unwrap();
        let desc: GraphDesc = toml::from_str(desc.as_str()).unwrap();

        let mut compute = Compute::new(desc.clone())
            .add_step(UpdatePaths)
            .add_step(Zmax)
            .add_step(Betweeness)
            .add_step(Capacity)
            .add_step(CopyIteration)
            .add_step(DeleteMaxBetweenness)
            .add_step(UpdatePaths)
            .add_step(Betweeness)
            .add_step(DeleteOverloaded);

        compute.run();

        let world = WorldData::new(&compute.state().at(0).graph.tracker, desc);
        Self {
            compute,
            world,
            selected_node: None,
            mode: Default::default(),
        }
    }
}

struct MyApp {
    renderer: Option<Renderer>,
    app_state: AppState,
    ui_state: UiState,
    input: Input,

    tab_manager: UiTabManager,
}

impl MyApp {
    fn init() -> Self {
        let app_state = AppState::new();
        let tab_manager = UiTabManager::new(&app_state.world);

        Self {
            renderer: None,
            app_state,
            tab_manager,
            ui_state: Default::default(),
            input: Default::default(),
        }
    }
}

impl UserApp for MyApp {
    fn init_renderer(&mut self, gl: Arc<glow::Context>) {
        self.renderer = Some(Renderer::new(gl.clone()));
    }

    fn update(&mut self, delta: Duration) {
        if self.input.is_pressed(Key::W) {
            self.app_state
                .world
                .camera
                .process_keyboard(CameraMovement::Up, delta.as_secs_f32());
        }

        if self.input.is_pressed(Key::S) {
            self.app_state
                .world
                .camera
                .process_keyboard(CameraMovement::Down, delta.as_secs_f32());
        }

        if self.input.is_pressed(Key::A) {
            self.app_state
                .world
                .camera
                .process_keyboard(CameraMovement::Left, delta.as_secs_f32());
        }

        if self.input.is_pressed(Key::D) {
            self.app_state
                .world
                .camera
                .process_keyboard(CameraMovement::Right, delta.as_secs_f32());
        }

        if self.input.is_pressed(Key::ArrowUp) {
            self.app_state
                .world
                .camera
                .process_keyboard(CameraMovement::Forward, delta.as_secs_f32());
        }

        if self.input.is_pressed(Key::ArrowDown) {
            self.app_state
                .world
                .camera
                .process_keyboard(CameraMovement::Backward, delta.as_secs_f32());
        }

        if self.input.is_pressed(Key::Lmb) && self.input.is_pressed(Key::Rmb) {
            self.app_state
                .world
                .camera
                .process_mouse_zoom(self.input.mouse_motion.1, delta.as_secs_f32());
        } else if self.input.is_pressed(Key::Lmb) {
            self.app_state
                .world
                .camera
                .process_mouse_motion(self.input.mouse_motion, delta.as_secs_f32());
        } else if self.input.is_pressed(Key::Rmb) {
            if let Some(ref renderer) = self.renderer {
                let idx = renderer.idx_from_stencil(self.input.mouse_position);
                if idx != 0 {
                    self.app_state.selected_node = Some(unsafe { Node::new(idx as u32 - 1) });
                } else {
                    self.app_state.selected_node = None;
                }
            }

            if let Some(node) = self.app_state.selected_node {
                let position = &mut self.app_state.world.positions[node].0;
                let (x, y) = self.input.mouse_motion;
                position.x += x as f32 * 3.5 * delta.as_secs_f32();
                position.y -= y as f32 * 3.5 * delta.as_secs_f32();
            }
        }

        if self.input.is_pressed(Key::Lctrl) && self.input.is_pressed(Key::Lmb) {
            let pos = self.input.mouse_to_world(&self.app_state.world);
            info!("{pos:?}");
        }

        self.input.update();
    }

    fn render(&mut self, _gl: Arc<glow::Context>) {
        let Some(ref renderer) = self.renderer else {
            return;
        };

        self.app_state
            .world
            .update_materials(self.app_state.compute.state().get());
        renderer.render(&self.app_state);
    }

    fn ui_layout(&mut self, egui_ctx: &egui::Context) {
        self.tab_manager.show(egui_ctx);
        // self.ui_state.show(egui_ctx, &mut self.app_state);
    }

    fn handle_window_events(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    if matches!(code, KeyCode::KeyW) {
                        self.input.set(Key::W, event.state);
                    }

                    if matches!(code, KeyCode::KeyS) {
                        self.input.set(Key::S, event.state);
                    }

                    if matches!(code, KeyCode::KeyA) {
                        self.input.set(Key::A, event.state);
                    }

                    if matches!(code, KeyCode::KeyD) {
                        self.input.set(Key::D, event.state);
                    }

                    if matches!(code, KeyCode::ArrowUp) {
                        self.input.set(Key::ArrowUp, event.state);
                    }

                    if matches!(code, KeyCode::ArrowDown) {
                        self.input.set(Key::ArrowDown, event.state);
                    }

                    if matches!(code, KeyCode::ControlLeft) {
                        self.input.set(Key::Lctrl, event.state);
                    }

                    if matches!(code, KeyCode::AltLeft) {
                        self.input.set(Key::Lalt, event.state);
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => {
                if matches!(button, MouseButton::Left) {
                    self.input.set(Key::Lmb, state);
                }

                if matches!(button, MouseButton::Right) {
                    self.input.set(Key::Rmb, state);
                }
            }
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                self.input.mouse_position = position.into();
            }
            _ => {}
        }
    }

    fn handle_device_events(&mut self, event: winit::event::DeviceEvent) {
        match event {
            DeviceEvent::MouseWheel { delta } => self.input.mouse_wheel = delta,
            DeviceEvent::MouseMotion { delta } => self.input.mouse_motion = delta,
            _ => {}
        }
    }
}

fn main() {
    LoggerWrapper::init().unwrap();

    let app = App::new(MyApp::init());
    app.run().expect("failed to run app");
}
