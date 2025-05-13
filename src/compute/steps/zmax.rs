use crate::{
    compute::state::{Iteration, State},
    graph::Graph,
};

use super::ComputeStep;

pub struct Zmax;

impl ComputeStep for Zmax {
    fn compute(&mut self, state: &mut State) -> bool {
        let beta_delta = {
            let Iteration { graph, info } = state.get_mut();

            let mut sum = 0.0;
            for i in graph.tracker.iter_alive() {
                for j in graph.tracker.iter_alive().exclude(i) {
                    sum += graph.path_finder.cost(i, j) as f64;
                }
            }

            info.zs.fill(0.0);

            for i in graph.tracker.iter_alive() {
                let mut per_node_sum = 0.0;
                for j in graph.tracker.iter_alive().exclude(i) {
                    per_node_sum += graph.path_finder.cost(i, j) as f64;
                }

                info.zs[i] = sum / (2.0 * per_node_sum);
            }

            let n = graph.alive() as f64;
            info.zmax = unsafe {
                let zs = info.zs.clone_vec();
                zs.into_iter().reduce(f64::max).unwrap()
            };

            info.beta = ((n - 1.0) * (2.0 * info.zmax - n)) / (info.zmax * (n - 2.0));
            (info.beta - 1.0).abs()
        };

        state.beta_deltas.push(beta_delta);

        true
    }
}
