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
        let mut state = State::new();
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
        'outer: loop {
            let Iteration { graph, .. } = self.state.get_mut();
            if graph.alive() == 2 {
                // TODO: kostil
                assert!(self.state.iter_count() > 1, "Initial graph is invalid");
                self.state.pop();
                break 'outer;
            }

            for step in self.steps.iter_mut() {
                if !step.compute(&mut self.state) {
                    break 'outer;
                }
            }
        }
    }
}

pub struct CopyIteration;

impl ComputeStep for CopyIteration {
    fn compute(&mut self, state: &mut State) -> bool {
        let Iteration { graph, info } = state.get();

        let graph = graph.clone();
        let info = info.clone();

        state.add_iter((graph, info).into());
        state.next();

        true
    }
}

pub struct UpdatePaths;

impl ComputeStep for UpdatePaths {
    fn compute(&mut self, state: &mut State) -> bool {
        state.get_mut().graph.update_paths().is_some()
    }
}
