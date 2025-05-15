use super::Tab;

pub struct ViewportWindow;

impl ViewportWindow {
    pub fn new() -> Self {
        Self
    }
}

impl Tab for ViewportWindow {
    fn title(&self) -> egui::WidgetText {
        "Viewport".into()
    }

    fn show(&mut self, ui: &mut egui::Ui) {}
}
