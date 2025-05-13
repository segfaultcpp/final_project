use log::info;

use crate::compute::state::{Iteration, State};

use super::ComputeStep;

pub struct DeleteMaxBetweenness;

impl ComputeStep for DeleteMaxBetweenness {
    fn compute(&mut self, state: &mut State) -> bool {
        let Iteration { graph, info } = state.get_mut();

        let value = info.betweenness[info.max_betweenness];
        info!(
            "Deleting {:?} with betweenness = {value}",
            info.max_betweenness
        );

        graph.delete(info.max_betweenness);

        true
    }
}

pub struct DeleteOverloaded;

impl ComputeStep for DeleteOverloaded {
    fn compute(&mut self, state: &mut State) -> bool {
        let Iteration { graph, info } = state.get_mut();

        let mut retired = vec![];
        for i in graph.tracker.iter_alive() {
            if info.betweenness[i] > info.capacity[i] {
                info!(
                    "Deleting {i:?}. Betweenness ({}) > Capacity ({})",
                    info.betweenness[i], info.capacity[i]
                );
                retired.push(i);
            }
        }

        for i in retired.iter() {
            graph.delete(*i);
        }

        state.ks.push(1.0 / retired.len() as f64);

        true
    }
}
