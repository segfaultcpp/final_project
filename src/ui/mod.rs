use std::time::Duration;

use egui::Ui;
use egui_dock::TabViewer;

pub mod editor;
pub mod summary;

pub(super) mod outliner;
pub(super) mod viewport;

pub struct UiTabViewer;

impl TabViewer for UiTabViewer {
    type Tab = Box<dyn MainTab>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.show(ui);
    }
}

pub trait MainTab {
    fn title(&self) -> egui::WidgetText;
    fn show(&mut self, ui: &mut egui::Ui);
    fn update(&mut self, delta: Duration);

    #[allow(unused_variables)]
    fn save_to(&self, file: String) {}

    #[allow(unused_variables)]
    fn open_file(&mut self, file: &str) {}
}

pub trait TabWindow<T> {
    fn title(&self) -> egui::WidgetText;
    fn show(&mut self, ui: &mut egui::Ui, state: &mut T);

    #[allow(unused_variables)]
    fn update(&mut self, state: &mut T, delta: Duration) {}
}
