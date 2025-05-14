use mat::Mat;
use node::{NADVec, Node, NodeStatusTracker};
use path_finder::PathFinder;
use serde::{Deserialize, Serialize};
use std::fmt;

pub mod mat;
pub mod node;
pub mod path_finder;

#[derive(Clone, Debug)]
pub struct GraphInfo {
    pub zs: NADVec<f64>,
    pub zmax: f64,
    pub beta: f64,
    pub beta_delta: f64,
    pub capacity: NADVec<f64>,
    pub min_capacity: Node,
    pub max_capacity: Node,
    pub betweenness: NADVec<f64>,
    pub min_betweenness: Node,
    pub max_betweenness: Node,
}

impl GraphInfo {
    pub fn new(tracker: &NodeStatusTracker) -> Self {
        Self {
            zs: NADVec::new(tracker),
            capacity: NADVec::new(tracker),
            betweenness: NADVec::new(tracker),
            zmax: 0.0,
            beta: 0.0,
            beta_delta: 0.0,
            min_capacity: Node::default(),
            max_capacity: Node::default(),
            min_betweenness: Node::default(),
            max_betweenness: Node::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Graph {
    pub tracker: NodeStatusTracker,
    pub adjacency: Mat<bool>,
    pub path_finder: PathFinder,
}

impl Graph {
    pub const CONNECTION_COST: i32 = 2;

    pub fn example() -> Self {
        GraphDesc::example().into()
    }

    pub fn node_count(&self) -> usize {
        self.tracker.node_count()
    }

    pub fn alive(&self) -> usize {
        self.tracker.alive()
    }

    pub fn is_adjacent(&self, i: Node, j: Node) -> bool {
        self.adjacency[(i, j)]
    }

    pub fn update_paths(&mut self) -> Option<()> {
        for src in self.tracker.iter_alive() {
            self.path_finder
                .find_shortest_path_for(&self.tracker, &self.adjacency, src)?;
        }
        Some(())
    }

    pub fn delete(&mut self, node: Node) {
        self.tracker.delete(node);
        self.adjacency.delete(&self.tracker, node);
    }
}

impl fmt::Display for Graph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "tracker: {:?}", self.tracker)?;
        writeln!(f, "adjacency:\n{}", self.adjacency)?;
        writeln!(f, "path_finder:\n{}", self.path_finder)
    }
}

impl From<GraphDesc> for Graph {
    fn from(value: GraphDesc) -> Self {
        let node_count = value.node_count();

        let tracker = NodeStatusTracker::new(node_count);
        let path_finder = PathFinder::new(node_count);
        let adjacency = Mat::<bool>::from(value);

        Self {
            tracker,
            adjacency,
            path_finder,
        }
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct NodeDesc {
    pub node_id: u32,
    pub position: [f32; 3],
    pub nodes: Vec<u32>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct GraphDesc {
    pub alpha: f64,
    pub nodes: Vec<NodeDesc>,
}

impl GraphDesc {
    pub fn nodes(&self) -> &[NodeDesc] {
        self.nodes.as_slice()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn example() -> Self {
        Self {
            alpha: 3.0,
            nodes: vec![
                NodeDesc {
                    node_id: 0,
                    position: [0.0, 0.0, 0.0],
                    nodes: vec![1, 2, 3],
                },
                NodeDesc {
                    node_id: 1,
                    position: [1.0, 0.0, 0.0],
                    nodes: vec![4, 5],
                },
                NodeDesc {
                    node_id: 2,
                    position: [2.0, 0.0, 0.0],
                    nodes: vec![4, 6],
                },
                NodeDesc {
                    node_id: 3,
                    position: [3.0, 0.0, 0.0],
                    nodes: vec![5, 6],
                },
                NodeDesc {
                    node_id: 4,
                    position: [0.0, 1.0, 0.0],
                    nodes: vec![9],
                },
                NodeDesc {
                    node_id: 5,
                    position: [1.0, 1.0, 0.0],
                    nodes: vec![8],
                },
                NodeDesc {
                    node_id: 6,
                    position: [2.0, 1.0, 0.0],
                    nodes: vec![7],
                },
                NodeDesc {
                    node_id: 7,
                    position: [3.0, 1.0, 0.0],
                    nodes: Vec::new(),
                },
                NodeDesc {
                    node_id: 8,
                    position: [0.0, 2.0, 0.0],
                    nodes: Vec::new(),
                },
                NodeDesc {
                    node_id: 9,
                    position: [1.0, 2.0, 0.0],
                    nodes: Vec::new(),
                },
            ],
        }
    }
}

mod test {
    #![allow(unused_imports)]
    use super::{Graph, GraphDesc};
    use crate::graph::{NodeDesc, node::Node};

    #[test]
    fn test_graph_desc() {
        let desc = GraphDesc::example();
        let graph = Graph::from(desc.clone());
        for NodeDesc {
            node_id: i, nodes, ..
        } in desc.nodes().iter()
        {
            for j in nodes.iter() {
                let (i, j) = unsafe { (Node::new(*i), Node::new(*j)) };

                assert!(graph.adjacency.is_set(i, j));
                assert!(graph.adjacency.is_set(j, i));
            }
        }
    }
}
