use egui_dock::{DockState, NodeIndex};
use egui_glow::Painter;

use crate::{compute::Compute, world::SummaryWorld};

use super::{TabWindow, editor::EditorState, viewport::ViewportWindow};

pub struct SummaryState {
    // TODO: rewrite all this stuff
    pub compute: Compute,
    pub world: SummaryWorld,
}

impl SummaryState {
    pub fn new(ed_state: &EditorState, alpha: f64) -> Self {
        let compute = Compute::new(&ed_state.adjacency, alpha);
        let world = SummaryWorld::new(&compute.state().get().graph.tracker, &ed_state.world);
        Self { compute, world }
    }
}

pub struct SummaryTab {
    state: SummaryState,
    dock_state: DockState<Box<dyn TabWindow<SummaryState>>>,
}

impl SummaryTab {
    pub fn new(painter: &mut Painter, ed_state: &EditorState, alpha: f64) -> Self {
        let mut dock_state: DockState<Box<dyn TabWindow<EditorState>>> =
            DockState::new(vec![Box::new(ViewportWindow::new(painter))]);
        let surface = dock_state.main_surface_mut();
        todo!()

        /*surface.split_left(
            NodeIndex::root(),
            0.2,
            vec![Box::new(OutlinerWindow::new())],
        );

        Self {
            dock_state,
            state: SummaryState::new(ed_state, alpha),
        }*/
    }
}
