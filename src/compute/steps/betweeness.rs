use crate::compute::state::{Iteration, State};

use super::ComputeStep;

pub struct Betweeness;

impl ComputeStep for Betweeness {
    fn compute(&mut self, state: &mut State) -> bool {
        let Iteration { graph, info } = state.get_mut();
        info.betweenness.fill(0.0);

        let (mut max, mut max_node) = (0.0, None);
        let (mut min, mut min_node) = (f64::MAX, None);

        for i in graph.tracker.iter_alive() {
            for (count, s) in graph.tracker.iter_alive().enumerate() {
                for t in graph.tracker.iter_alive().skip(count + 1) {
                    info.betweenness[i] += graph.path_finder.contains((s, t), i) as i64 as f64;
                }
            }

            if info.betweenness[i] > max {
                max_node = Some(i);
                max = info.betweenness[i];
            }

            if info.betweenness[i] < min {
                min_node = Some(i);
                min = info.betweenness[i];
            }
        }

        info.max_betweenness = max_node.unwrap();
        info.min_betweenness = min_node.unwrap();

        true
    }
}
