use crate::world::{
    desc::{GraphDesc, NodeDesc},
    node::{IsNode, NodeRange, PhysNode, PhysNodeIdx},
};

/// Extensible Adjacency matrix
#[derive(Debug, Clone, Default)]
pub struct ExtAdjacency(Vec<Vec<bool>>);

impl ExtAdjacency {
    pub fn new(node_count: usize) -> Self {
        Self(vec![vec![false; node_count]; node_count])
    }

    pub fn set(&mut self, i: PhysNode, j: PhysNode) {
        let (i, j) = (i.idx() as usize, j.idx() as usize);
        assert_ne!(i, j, "You cannot access diagonal line of matrix.");
        assert!(i < self.0.len() && j < self.0.len());
        self.0[i][j] = true;
    }

    pub fn unset(&mut self, i: PhysNode, j: PhysNode) {
        let (i, j) = (i.idx() as usize, j.idx() as usize);
        assert_ne!(i, j, "You cannot access diagonal line of matrix.");
        assert!(i < self.0.len() && j < self.0.len());
        self.0[i][j] = false;
    }

    pub fn is_set(&self, i: PhysNode, j: PhysNode) -> bool {
        let (i, j) = (i.idx() as usize, j.idx() as usize);
        assert_ne!(i, j, "You cannot access diagonal line of matrix.");
        assert!(
            i < self.0.len() && j < self.0.len(),
            "i = {i}, j = {j}, len() = {}",
            self.0.len()
        );
        self.0[i][j]
    }

    pub(super) fn add_nodes(&mut self, count: usize) -> NodeRange<PhysNode> {
        let old_size = self.0.len();
        for row in self.0.iter_mut() {
            row.extend((0..count).map(|_| false));
        }

        self.0
            .extend((0..count).map(|_| vec![false; old_size + count]));

        let (begin, end) = unsafe {
            (
                PhysNode::new(old_size as PhysNodeIdx),
                PhysNode::new(self.0.len() as PhysNodeIdx - 1),
            )
        };

        (begin, end).into()
    }

    pub(super) fn add_node(&mut self) -> PhysNode {
        self.add_nodes(1).start()
    }

    pub(super) fn free(&mut self, phys_idx: PhysNode) {
        for (i, row) in self.0.iter_mut().enumerate() {
            if i == phys_idx.idx() as usize {
                continue;
            }

            let value = row.pop().unwrap();
            row[phys_idx.idx() as usize] = value;
        }
    }

    pub(super) fn free_last(&mut self) {
        self.0.pop().unwrap();
        for row in self.0.iter_mut() {
            row.pop().unwrap();
        }
    }

    pub fn node_count(&self) -> usize {
        self.0.len()
    }
}
