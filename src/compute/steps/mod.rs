use crate::graph::{Graph, GraphInfo};

pub mod betweeness;
pub mod zmax;

pub trait ComputeStep {
    fn compute(&self, graph: &Graph, info: &mut GraphInfo);
}
