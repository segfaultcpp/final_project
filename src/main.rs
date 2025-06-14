use std::{path::Path, sync::Arc, time::Duration};

use app::{App, UserApp};
use egui::{Frame, Modal, TextEdit};
use egui_dock::{DockArea, DockState};
use egui_glow::Painter;
use input::{Input, Key};
use log::{LevelFilter, Log, SetLoggerError, info};
use renderer::Renderer;
use simple_logger::SimpleLogger;
use ui::{MainTab, UiTabViewer, editor::EditorTab};
use winit::{
    event::{DeviceEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

mod app;
// mod compute;
// mod graph;
mod input;
mod renderer;
mod ui;
mod utils;
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

struct MyApp {
    input: Input,
    opened_file: Option<String>,

    dock_state: DockState<Box<dyn MainTab>>,

    save_as_opened: bool,
    save_as_file: String,

    project_menu_opened: bool,
    projects: Vec<String>,
    selected_project: usize,
    blank_project_name: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            opened_file: None,
            input: Default::default(),
            dock_state: DockState::new(vec![]),
            save_as_opened: false,
            save_as_file: String::new(),

            project_menu_opened: true,
            projects: Self::fetch_projects(),
            selected_project: usize::MAX,
            blank_project_name: String::new(),
        }
    }
}

impl MyApp {
    fn fetch_projects() -> Vec<String> {
        let mut files = vec![];
        for entry in std::fs::read_dir("projects/").unwrap() {
            let path = entry.unwrap().path();
            if path.is_file() {
                files.push(path.file_stem().unwrap().to_str().unwrap().to_owned());
            }
        }
        files
    }

    fn show_top_panel(&mut self, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        if let Some(ref file) = self.opened_file {
                            self.save_to(file.as_str());
                        }
                    }

                    if ui.button("Save as...").clicked() {
                        self.save_as_opened = true;
                    }

                    if ui.button("Quit").clicked() {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });

            if self.save_as_opened {
                self.show_save_as_menu(ui);
            }
        });
    }

    fn show_dock_area(&mut self, ui: &mut egui::Ui) {
        DockArea::new(&mut self.dock_state)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut UiTabViewer);
    }

    fn show_project_menu(&mut self, ui: &mut egui::Ui) {
        Modal::new(ui.id()).show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Open Project");
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                let selected = self.selected_project;

                for (i, project) in self.projects.iter().enumerate() {
                    if ui
                        .selectable_label(i == selected, project.as_str())
                        .clicked()
                    {
                        self.selected_project = i;
                    }
                }
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Open").clicked() && self.selected_project != usize::MAX {
                    self.project_menu_opened = false;
                    self.open_project();
                }
            });

            ui.separator();

            ui.vertical_centered(|ui| {
                ui.label("Create blank Project");
            });

            ui.separator();

            let response = ui.add(
                TextEdit::singleline(&mut self.blank_project_name)
                    .hint_text("Enter project name here"),
            );

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Create").clicked() {
                    if !self.blank_project_name.is_empty()
                        && !self.projects.contains(&self.blank_project_name)
                    {
                        self.project_menu_opened = false;
                        self.create_blank_project();
                    }

                    response
                        .highlight()
                        .show_tooltip_text("Project name cannot be empty!");
                }
            });
        });
    }

    fn show_save_as_menu(&mut self, ui: &mut egui::Ui) {
        Modal::new(ui.id()).show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Save Project");
            });

            ui.separator();

            let response = ui.add(
                TextEdit::singleline(&mut self.save_as_file).hint_text("Enter project name here"),
            );
            ui.label("The project will be saved in projects/ directory");

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Save").clicked() {
                    if !self.save_as_file.is_empty() {
                        self.save_as_opened = false;
                        self.save_to(self.save_as_file.as_str());
                    }

                    response
                        .highlight()
                        .show_tooltip_text("Project name cannot be empty!");
                }
            });
        });
    }

    fn create_blank_project(&mut self) {
        self.opened_file = Some(self.blank_project_name.clone());
        info!("Blank project {} created", self.blank_project_name);
    }

    fn open_project(&mut self) {
        let file = self.projects[self.selected_project].clone();

        let Some((_, tab)) = self.dock_state.iter_all_tabs_mut().next() else {
            return;
        };

        let path = String::from("projects/") + file.as_str() + ".toml";
        info!("Opening file from path {path}");

        tab.open_file(path.as_str());
        self.opened_file = Some(file);
    }

    fn save_to(&self, file: &str) {
        let Some((_, tab)) = self.dock_state.iter_all_tabs().next() else {
            return;
        };

        let file = String::from("projects/") + file + ".toml";
        tab.save_to(file.clone());

        info!("Saved from tab {} to file {}", tab.title().text(), file);
    }
}

impl UserApp for MyApp {
    fn initialize(&mut self, gl: Arc<glow::Context>, painter: &mut Painter) {
        Renderer::init(gl.clone());
        let tabs: Vec<Box<dyn MainTab>> = vec![Box::new(EditorTab::new(painter))];
        self.dock_state = DockState::new(tabs);
    }

    fn update(&mut self, delta: Duration) {
        for tab in self.dock_state.iter_all_tabs_mut() {
            tab.1.update(delta);
        }
    }

    fn render(&mut self, _gl: Arc<glow::Context>) {
        // TODO: remove fn from trait
    }

    fn ui_layout(&mut self, egui_ctx: &egui::Context) {
        self.show_top_panel(egui_ctx);

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            if self.opened_file.is_some() {
                self.show_dock_area(ui);
            } else {
                self.show_project_menu(ui);
            }
        });
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

    let app = App::new(MyApp::default());
    app.run().expect("failed to run app");
}
