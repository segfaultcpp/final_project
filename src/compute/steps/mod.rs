use super::state::State;

pub mod betweeness;
pub mod capacity;
pub mod delete;
pub mod zmax;

pub trait ComputeStep {
    fn compute(&mut self, state: &mut State) -> bool;
}
