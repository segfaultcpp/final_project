use crate::graph::GraphStats;

use super::{
    Material, Position,
    camera::Camera,
    editor::EditorWorld,
    node::{NADVec, NodeStatusTracker},
};

#[derive(Clone)]
pub struct SummaryWorld {
    pub positions: NADVec<Position>,
    pub materials: NADVec<Material>,
    pub camera: Camera,
}

impl SummaryWorld {
    pub fn new(tracker: &NodeStatusTracker, ed_world: &EditorWorld) -> Self {
        Self {
            positions: {
                let mut positions = NADVec::new(tracker);
                for i in tracker.iter_alive() {
                    positions[i] = ed_world.positions[i.idx()];
                }
                positions
            },
            materials: NADVec::new(tracker),
            camera: ed_world.camera,
        }
    }

    pub fn update_materials(&mut self, tracker: &NodeStatusTracker, stats: &GraphStats) {
        for i in tracker.iter_alive() {
            let min = stats.betweenness[stats.min_betweenness];
            let max = stats.betweenness[stats.max_betweenness];

            self.materials[i].update_albedo(stats.betweenness[i], min, max);
        }
    }
}
