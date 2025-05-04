use log::info;
use state::{Iteration, State};
use steps::ComputeStep;

use crate::graph::{GraphDesc, GraphInfo};

pub mod state;
pub mod steps;

#[derive(Default)]
pub struct Compute {
    state: State,
    steps: Vec<Box<dyn ComputeStep>>,
}

impl Compute {
    pub fn new(desc: GraphDesc) -> Self {
        let mut state = State::default();
        state.add_iter(Iteration::new(desc));

        Self {
            state,
            steps: vec![],
        }
    }

    pub fn add_step<T: ComputeStep + 'static>(mut self, step: T) -> Self {
        self.steps.push(Box::new(step));
        self
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn run(&mut self) {
        loop {
            let Iteration { graph, info } = self.state.get_mut();
            if graph.update_paths().is_none() || graph.alive() == 2 {
                // TODO: kostil
                assert!(self.state.iter_count() > 1, "Initial graph is invalid");
                self.state.pop();
                break;
            }

            for step in self.steps.iter() {
                step.compute(graph, info);
            }

            let mut max_b = 0.0;
            let mut min_b = 0.0;
            let mut max_node = None;
            for i in graph.tracker.iter_alive() {
                assert!(!info.betweenness[i].is_nan());

                if max_b < info.betweenness[i] {
                    max_b = info.betweenness[i];
                    max_node = Some(i);
                }

                if min_b > info.betweenness[i] {
                    min_b = info.betweenness[i];
                }
            }

            let max_node = max_node.unwrap();
            info.max_betweenness = max_b;
            info.min_betweenness = min_b;
            info!("Deleting {max_node:?} with betweenness = {max_b}");

            let mut graph = graph.clone();
            let info = GraphInfo::new(&graph.tracker);

            graph.delete(max_node);

            self.state.add_iter((graph, info).into());
            self.state.next();
        }
    }
}
