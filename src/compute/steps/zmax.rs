use crate::graph::Graph;

use super::ComputeStep;

pub struct Zmax;

impl ComputeStep for Zmax {
    fn compute(&self, graph: &Graph, info: &mut crate::graph::GraphInfo) {
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

        info.beta_delta = (info.beta - 1.0).abs();
    }
}
