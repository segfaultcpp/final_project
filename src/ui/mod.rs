use egui::Ui;
use egui_dock::{DockArea, DockState, Style, TabViewer};
use tabs::editor::EditorTab;

use crate::{AppState, world::WorldData};

mod tabs;

struct UiTabViewer;

impl TabViewer for UiTabViewer {
    type Tab = Box<dyn tabs::Tab>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.show(ui);
    }
}

pub struct UiTabManager {
    dock_state: DockState<Box<dyn tabs::Tab>>,
}

impl UiTabManager {
    pub fn new(world: &WorldData) -> Self {
        let tabs: Vec<Box<dyn tabs::Tab>> = vec![Box::new(EditorTab::new(world))];
        Self {
            dock_state: DockState::new(tabs),
        }
    }

    pub fn show(&mut self, egui_ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(egui_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        egui_ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(egui_ctx, |ui| {
            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ui.style().as_ref()))
                .show_inside(ui, &mut UiTabViewer);
        });
    }

    pub fn submit(&self, app_state: &mut AppState) {
        for (_, tab) in self.dock_state.iter_all_tabs() {}
    }
}
