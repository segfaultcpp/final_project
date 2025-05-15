use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::{AppState, ui::UiTabViewer, world::WorldData};

use super::{Tab, outliner::OutlinerWindow, viewport::ViewportWindow};

pub struct EditorTab {
    dock_state: DockState<Box<dyn Tab>>,
}

impl EditorTab {
    pub fn new(world: &WorldData) -> Self {
        let mut dock_state: DockState<Box<dyn Tab>> =
            DockState::new(vec![Box::new(ViewportWindow::new())]);
        let surface = dock_state.main_surface_mut();

        surface.split_left(
            NodeIndex::root(),
            0.2,
            vec![Box::new(OutlinerWindow::new(world))],
        );

        Self { dock_state }
    }
}

impl Tab for EditorTab {
    fn title(&self) -> egui::WidgetText {
        "Editor".into()
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut UiTabViewer);
    }
}
