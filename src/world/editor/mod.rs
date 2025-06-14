use cgmath::Vector3;
use ext_adj::ExtAdjacency;
use log::info;
use topology::Topology;

use crate::{
    renderer::draw::{EdgeDrawItem, NodeDrawItem},
    world::node::{IsNode, PhysNodeIdx},
};

use super::{
    Material, Position, World,
    camera::Camera,
    desc::{GraphDesc, NodeDesc},
    node::{GlobNodeIdx, Node, NodeAllocator, NodeRange, NodeRangeIter, PhysNode},
};

pub mod ext_adj;
pub mod topology;

#[derive(Clone, Default)]
pub struct EditorWorld {
    node_alloc: NodeAllocator,
    pub positions: Vec<Position>,
    pub materials: Vec<Material>,
    pub ext_adj: ExtAdjacency,
    pub camera: Camera,
}

impl EditorWorld {
    pub fn nodes(&self) -> impl Iterator<Item = Node> {
        self.node_alloc.nodes()
    }

    pub fn phys_nodes(&self) -> NodeRangeIter<PhysNode> {
        self.node_alloc.phys_nodes()
    }

    pub fn zipped_nodes(&self) -> impl Iterator<Item = (Node, PhysNode)> {
        self.node_alloc.zipped_nodes()
    }

    pub fn glob_node(&self, node: PhysNode) -> Node {
        self.node_alloc.glob_node(node)
    }

    pub fn phys_node(&self, node: Node) -> PhysNode {
        self.node_alloc.phys_node(node)
    }

    pub fn spawn(&mut self, pos: &[Vector3<f32>]) -> NodeRange<Node> {
        let count = pos.len();
        let node_range = self.node_alloc.allocate_range(count);

        let start = self.positions.len();
        self.positions.extend(pos.iter().map(|p| Position(*p)));
        self.materials
            .extend((0..pos.len()).map(|_| Material::default()));
        let end = self.positions.len() - 1;

        let pos_range = start..=end;
        let adj_range = self.ext_adj.add_nodes(count);

        {
            debug_assert_eq!(
                self.node_alloc.phys_node(node_range.start()).idx(),
                *pos_range.start() as PhysNodeIdx
            );
            debug_assert_eq!(*pos_range.start() as PhysNodeIdx, adj_range.start().idx());

            debug_assert_eq!(
                self.node_alloc.phys_node(node_range.end()).idx(),
                *pos_range.end() as PhysNodeIdx
            );
            debug_assert_eq!(*pos_range.end() as PhysNodeIdx, adj_range.end().idx());

            debug_assert_eq!(self.positions.len(), self.ext_adj.node_count());
        }

        node_range
    }

    pub fn spawn_single(&mut self) -> Node {
        let mut camera_pos = self.camera.position_as_vec();
        camera_pos.z = 0.0;

        self.spawn(&[camera_pos]).start()
    }

    pub fn spawn_topology(&mut self, topology: Topology, count: usize) -> NodeRange<Node> {
        let mut camera_pos = self.camera.position_as_vec();
        camera_pos.z = 0.0;

        let positions = topology.positions(camera_pos, count);

        let node_range = self.spawn(positions.as_slice());

        let start = self.node_alloc.phys_node(node_range.start());
        let end = self.node_alloc.phys_node(node_range.end());
        topology.connect(&mut self.ext_adj, (start, end).into());

        node_range
    }

    pub fn despawn(&mut self, node: Node) {
        let phys_idx = self.node_alloc.free(node);

        if phys_idx.idx() == self.positions.len() as PhysNodeIdx - 1 {
            self.positions.pop().unwrap();
            self.ext_adj.free_last();
        } else {
            let last_pos = self.positions.pop().unwrap();
            self.positions[phys_idx.idx() as usize] = last_pos;

            self.ext_adj.free(phys_idx);
        }
    }

    pub fn connect(&mut self, i: Node, j: Node) {
        let i = self.node_alloc.phys_node(i);
        let j = self.node_alloc.phys_node(j);

        self.ext_adj.set(i, j);
    }

    pub fn disconnect(&mut self, i: Node, j: Node) {
        let i = self.node_alloc.phys_node(i);
        let j = self.node_alloc.phys_node(j);

        self.ext_adj.unset(i, j);
    }

    pub fn is_connected(&self, i: Node, j: Node) -> bool {
        let i = self.node_alloc.phys_node(i);
        let j = self.node_alloc.phys_node(j);

        self.ext_adj.is_set(i, j)
    }

    pub fn position(&self, node: Node) -> &Position {
        let node = self.node_alloc.phys_node(node);
        &self.positions[node.idx() as usize]
    }

    pub fn position_mut(&mut self, node: Node) -> &mut Position {
        let node = self.node_alloc.phys_node(node);
        &mut self.positions[node.idx() as usize]
    }

    pub fn material(&self, node: Node) -> &Material {
        let node = self.node_alloc.phys_node(node);
        &self.materials[node.idx() as usize]
    }

    pub fn material_mut(&mut self, node: Node) -> &mut Material {
        let node = self.node_alloc.phys_node(node);
        &mut self.materials[node.idx() as usize]
    }
}

impl World for EditorWorld {
    fn camera(&self) -> &Camera {
        &self.camera
    }

    fn build_node_draw_items(&self) -> Vec<NodeDrawItem> {
        NodeDrawItem::build(self.positions.iter(), self.materials.iter())
    }

    fn build_edge_draw_items(&self) -> Vec<EdgeDrawItem> {
        let mut ret = vec![];

        for i in self.phys_nodes() {
            // NOTE: The physical order of nodes in memory is not guaranteed
            // as well as symmetry of ExtAdjacency,
            // so we iterate through all nodes.
            for j in self.phys_nodes() {
                if i == j {
                    continue;
                }

                if self.ext_adj.is_set(i, j) {
                    ret.push(EdgeDrawItem {
                        positions: [
                            self.positions[i.idx() as usize],
                            self.positions[j.idx() as usize],
                        ],
                    });
                }
            }
        }
        ret
    }
}

impl From<GraphDesc> for EditorWorld {
    fn from(desc: GraphDesc) -> Self {
        let mut positions = vec![];

        let mut node_alloc = NodeAllocator::default();
        for node_desc in desc.nodes.iter() {
            node_alloc.allocate_with_idx(node_desc.node_id);
            positions.push(Position(node_desc.position.into()));
        }

        let mut ext_adj = ExtAdjacency::new(desc.node_count());
        for (i, node_desc) in desc.nodes.iter().enumerate() {
            let i = unsafe { PhysNode::new(i as PhysNodeIdx) };
            for j in node_desc.nodes.iter() {
                let j = unsafe { Node::new(*j as GlobNodeIdx) };
                let j = node_alloc.phys_node(j);

                ext_adj.set(i, j);
                ext_adj.set(j, i);
            }
        }

        let node_count = positions.len();
        Self {
            node_alloc,
            positions,
            ext_adj,
            materials: vec![Material::default(); node_count],
            camera: Camera::new(),
        }
    }
}

impl From<EditorWorld> for GraphDesc {
    fn from(value: EditorWorld) -> Self {
        let mut nodes = vec![];
        for i in value.nodes() {
            nodes.push(NodeDesc {
                node_id: i.idx(),
                nodes: {
                    let mut nodes = vec![];

                    for j in value.nodes() {
                        if i == j {
                            continue;
                        }

                        if value.is_connected(i, j) {
                            nodes.push(j.idx());
                        }
                    }

                    nodes
                },
                position: value.position(i).0.into(),
            })
        }
        Self { nodes }
    }
}
