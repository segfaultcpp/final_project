use super::ComputeStep;

pub struct Betweeness;

impl ComputeStep for Betweeness {
    fn compute(&self, graph: &crate::graph::Graph, info: &mut crate::graph::GraphInfo) {
        info.betweenness.fill(0.0);

        for i in graph.tracker.iter_alive() {
            for (count, s) in graph.tracker.iter_alive().enumerate() {
                for t in graph.tracker.iter_alive().skip(count + 1) {
                    info.betweenness[i] += graph.path_finder.contains((s, t), i) as i64 as f64;
                }
            }
        }
    }
}
