pub(super) mod editor;
pub(super) mod outliner;
pub(super) mod viewport;

pub(super) trait Tab {
    fn title(&self) -> egui::WidgetText;
    fn show(&mut self, ui: &mut egui::Ui);
}
