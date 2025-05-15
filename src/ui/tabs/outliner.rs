use crate::world::WorldData;

use super::Tab;

pub struct OutlinerWindow {}

impl OutlinerWindow {
    pub fn new(world: &WorldData) -> Self {
        Self {}
    }

    fn submit(&self, app_state: &mut crate::AppState) {}
}

impl Tab for OutlinerWindow {
    fn title(&self) -> egui::WidgetText {
        "Outliner".into()
    }

    fn show(&mut self, ui: &mut egui::Ui) {}
}
