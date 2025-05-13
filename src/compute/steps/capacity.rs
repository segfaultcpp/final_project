use crate::compute::state::{Iteration, State};

use super::ComputeStep;

pub struct Capacity;

impl ComputeStep for Capacity {
    fn compute(&mut self, state: &mut State) -> bool {
        let alpha = state.alpha;
        let Iteration { graph, info } = state.get_mut();

        let (mut max, mut max_node) = (0.0, None);
        let (mut min, mut min_node) = (f64::MAX, None);

        for i in graph.tracker.iter_alive() {
            info.capacity[i] = (1.0 + alpha) * info.betweenness[i];

            if info.capacity[i] > max {
                max_node = Some(i);
                max = info.capacity[i];
            } else if info.capacity[i] < min {
                min_node = Some(i);
                min = info.capacity[i];
            }
        }

        info.max_capacity = max_node.unwrap();
        info.min_capacity = min_node.unwrap();

        true
    }
}
