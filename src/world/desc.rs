use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct NodeDesc {
    pub node_id: u32,
    pub position: [f32; 3],
    pub nodes: Vec<u32>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct GraphDesc {
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
