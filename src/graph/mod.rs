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
    pub betweenness: NADVec<f64>,
    pub min_betweenness: f64,
    pub max_betweenness: f64,
}

impl GraphInfo {
    pub fn new(tracker: &NodeStatusTracker) -> Self {
        Self {
            zs: NADVec::new(tracker),
            betweenness: NADVec::new(tracker),
            zmax: 0.0,
            beta: 0.0,
            beta_delta: 0.0,
            min_betweenness: 0.0,
            max_betweenness: 0.0,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeDesc {
    node_id: u32,
    nodes: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphDesc {
    nodes: Vec<NodeDesc>,
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
            nodes: vec![
                NodeDesc {
                    node_id: 0,
                    nodes: vec![1, 2, 3],
                },
                NodeDesc {
                    node_id: 1,
                    nodes: vec![4, 5],
                },
                NodeDesc {
                    node_id: 2,
                    nodes: vec![4, 6],
                },
                NodeDesc {
                    node_id: 3,
                    nodes: vec![5, 6],
                },
                NodeDesc {
                    node_id: 4,
                    nodes: vec![9],
                },
                NodeDesc {
                    node_id: 5,
                    nodes: vec![8],
                },
                NodeDesc {
                    node_id: 6,
                    nodes: vec![7],
                },
                NodeDesc {
                    node_id: 7,
                    nodes: Vec::new(),
                },
                NodeDesc {
                    node_id: 8,
                    nodes: Vec::new(),
                },
                NodeDesc {
                    node_id: 9,
                    nodes: Vec::new(),
                },
            ],
        }
    }
}

mod test {
    use crate::graph::{NodeDesc, node::Node};

    use super::{Graph, GraphDesc};

    #[test]
    fn test_graph_desc() {
        let desc: GraphDesc = toml::from_str(
            r#"
            [[nodes]]
            node_id = 0
            nodes = [1, 2, 3]
            
            [[nodes]]
            node_id = 1
            nodes = [4, 5]
            
            [[nodes]]
            node_id = 2
            nodes = [4, 6]
            
            [[nodes]]
            node_id = 3
            nodes = [5, 6]
            
            [[nodes]]
            node_id = 4
            nodes = [9]
            
            [[nodes]]
            node_id = 5
            nodes = [8]
            
            [[nodes]]
            node_id = 6
            nodes = [7]

            [[nodes]]
            node_id = 7
            nodes = []
            
            [[nodes]]
            node_id = 8
            nodes = []
            
            [[nodes]]
            node_id = 9
            nodes = []
        "#,
        )
        .unwrap();

        assert_eq!(desc, GraphDesc::example());

        let graph = Graph::from(desc.clone());
        for NodeDesc { node_id: i, nodes } in desc.nodes().iter() {
            for j in nodes.iter() {
                let (i, j) = unsafe { (Node::new(*i), Node::new(*j)) };

                assert!(graph.adjacency.is_set(i, j));
                assert!(graph.adjacency.is_set(j, i));
            }
        }
    }
}
