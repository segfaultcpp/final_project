use std::{
    collections::HashMap,
    ops::{Index, IndexMut, RangeInclusive},
};

pub type GlobNodeIdx = u32;
pub type PhysNodeIdx = u32;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Node(GlobNodeIdx);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhysNode(PhysNodeIdx);

macro_rules! impl_node {
    ($t:ident, $idx:ident) => {
        impl IsNode for $t {
            type Idx = $idx;
            fn idx(self) -> $idx {
                self.0
            }
        }

        unsafe impl AdvanceNode for $t {
            fn advance(&mut self) {
                self.0 += 1;
            }
        }

        impl Default for $t {
            fn default() -> Self {
                Self($idx::MAX)
            }
        }

        impl $t {
            pub unsafe fn new(node: $idx) -> Self {
                Self(node)
            }

            pub fn is_valid(self) -> bool {
                self.0 != $idx::MAX
            }
        }
    };
}

impl_node!(Node, GlobNodeIdx);
impl_node!(PhysNode, PhysNodeIdx);

pub trait IsNode {
    type Idx: Ord;
    fn idx(self) -> Self::Idx;
}

unsafe trait AdvanceNode {
    fn advance(&mut self);
}

#[derive(Clone, Copy, Debug)]
pub struct NodeRange<T>
where
    T: Copy + Clone + IsNode + AdvanceNode,
{
    start: T,
    end: T,
}

impl<T> From<(T, T)> for NodeRange<T>
where
    T: Copy + Clone + IsNode + AdvanceNode,
{
    fn from(value: (T, T)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

impl<T> NodeRange<T>
where
    T: Copy + Clone + IsNode + AdvanceNode,
{
    pub fn start(self) -> T {
        self.start
    }

    pub fn end(self) -> T {
        self.end
    }

    pub fn range_inclusive(self) -> RangeInclusive<T::Idx> {
        self.start.idx()..=self.end.idx()
    }

    pub fn iter(self) -> NodeRangeIter<T> {
        NodeRangeIter {
            current: self.start,
            end: self.end,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeRangeIter<T>
where
    T: Copy + Clone + IsNode + AdvanceNode,
{
    current: T,
    end: T,
}

impl<T> Iterator for NodeRangeIter<T>
where
    T: Copy + Clone + IsNode + AdvanceNode,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.idx() > self.end.idx() {
            return None;
        }

        let node = self.current;
        self.current.advance();
        Some(node)
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord)]
struct NodeInfo {
    phys_idx: PhysNodeIdx,
}

#[derive(Default, Clone)]
pub struct NodeAllocator {
    nodes: HashMap<GlobNodeIdx, PhysNodeIdx>,
    phys_to_glob: Vec<GlobNodeIdx>,
    next_glob_idx: GlobNodeIdx,
}

impl NodeAllocator {
    pub fn node_count(&self) -> usize {
        self.phys_to_glob.len()
    }

    pub fn nodes(&self) -> impl Iterator<Item = Node> {
        self.phys_to_glob.iter().map(|n| Node(*n))
    }

    pub fn phys_nodes(&self) -> NodeRangeIter<PhysNode> {
        if self.phys_to_glob.is_empty() {
            NodeRange {
                start: PhysNode(1),
                end: PhysNode(0),
            }
            .iter()
        } else {
            NodeRange {
                start: PhysNode(0),
                end: PhysNode(self.phys_to_glob.len() as PhysNodeIdx - 1),
            }
            .iter()
        }
    }

    pub fn zipped_nodes(&self) -> impl Iterator<Item = (Node, PhysNode)> {
        self.phys_to_glob
            .iter()
            .enumerate()
            .map(|(p, g)| (Node(*g), PhysNode(p as PhysNodeIdx)))
    }

    pub fn phys_node(&self, node: Node) -> PhysNode {
        PhysNode(self.nodes[&node.0])
    }

    pub fn glob_node(&self, node: PhysNode) -> Node {
        Node(self.phys_to_glob[node.0 as usize])
    }

    pub fn allocate_with_idx(&mut self, glob_idx: GlobNodeIdx) -> Node {
        let phys_idx = self.phys_to_glob.len() as PhysNodeIdx;
        assert!(self.nodes.insert(glob_idx, phys_idx).is_none(),);

        if glob_idx >= self.next_glob_idx {
            self.next_glob_idx = glob_idx + 1;
        }

        self.phys_to_glob.push(glob_idx);
        Node(glob_idx)
    }

    pub fn allocate(&mut self) -> Node {
        self.allocate_range(1).start()
    }

    pub fn allocate_range(&mut self, count: usize) -> NodeRange<Node> {
        let phys_idx_start = self.phys_to_glob.len() as PhysNodeIdx;
        let phys_idx_end = phys_idx_start + count as PhysNodeIdx - 1;
        let phys_idx_range = phys_idx_start..=phys_idx_end;

        let glob_idx_start = self.next_glob_idx;
        let glob_idx_end = glob_idx_start + count as GlobNodeIdx - 1;
        let glob_idx_range = glob_idx_start..=glob_idx_end;

        for (phys_idx, glob_idx) in phys_idx_range.zip(glob_idx_range.clone()) {
            assert!(self.nodes.insert(glob_idx, phys_idx).is_none(),);
        }

        self.next_glob_idx += count as GlobNodeIdx;
        self.phys_to_glob.extend(glob_idx_range);

        (Node(glob_idx_start), Node(glob_idx_end)).into()
    }

    pub fn free(&mut self, node: Node) -> PhysNode {
        debug_assert!(!self.phys_to_glob.is_empty());

        if let Some((_, phys_idx)) = self.nodes.remove_entry(&node.0) {
            if phys_idx == self.phys_to_glob.len() as PhysNodeIdx - 1 {
                self.phys_to_glob.pop().unwrap();
            } else {
                let last_node_glob_idx = self.phys_to_glob.pop().unwrap();
                self.phys_to_glob[phys_idx as usize] = last_node_glob_idx;
                self.nodes
                    .entry(last_node_glob_idx)
                    .and_modify(|v| *v = phys_idx);
            }

            PhysNode(phys_idx)
        } else {
            panic!("Attempt to double-free a node [with id = {node:?}]. This is a bug.");
        }
    }
}
/*
#[derive(Default, Clone, Debug)]
pub struct NodeStatusTracker {
    nodes: Vec<bool>,
    alive: usize,
}

impl NodeStatusTracker {
    pub fn new(node_count: usize) -> Self {
        Self {
            nodes: vec![true; node_count],
            alive: node_count,
        }
    }

    pub fn delete(&mut self, node: Node) {
        assert!(
            node.idx() < self.nodes.len(),
            "Accessing invalid node(id = {node:?}). Maximum node id = {}",
            self.nodes.len() - 1
        );

        assert!(self.is_alive(node), "Trying to delete already deleted node");

        self.nodes[node.idx()] = false;
        self.alive -= 1;
    }

    pub fn is_alive(&self, node: Node) -> bool {
        assert!(
            node.idx() < self.nodes.len(),
            "Accessing invalid node(id = {node:?}). Maximum node id = {}",
            self.nodes.len() - 1
        );
        self.nodes[node.idx()]
    }

    pub fn alive(&self) -> usize {
        self.alive
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn iter_alive(&self) -> AliveNodeIter<'_> {
        AliveNodeIter(self.nodes.iter(), 0)
    }
}

#[derive(Debug, Clone)]
pub struct AliveNodeIter<'a>(std::slice::Iter<'a, bool>, usize);

impl AliveNodeIter<'_> {
    pub fn exclude(self, node: Node) -> impl Iterator<Item = Node> {
        self.filter(move |&n| n != node)
    }
}

impl Iterator for AliveNodeIter<'_> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let status = self.0.next()?;

            let idx = self.1;
            self.1 += 1;

            if *status {
                return Some(Node(idx as u32));
            }
        }
    }
}

/// Node-associated data vector
#[derive(Clone, Debug)]
pub struct NADVec<T: Default + Clone>(Vec<T>);

impl<T: Default + Clone> NADVec<T> {
    pub fn new(tracker: &NodeStatusTracker) -> Self {
        Self(vec![<T as Default>::default(); tracker.node_count()])
    }

    pub fn fill(&mut self, value: T) {
        self.0.fill(value);
    }

    pub fn iter(&self, tracker: &NodeStatusTracker) -> impl Iterator<Item = &T> {
        tracker.iter_alive().map(|node| &self[node])
    }

    pub fn iter_exclude(
        &self,
        tracker: &NodeStatusTracker,
        exclude: Node,
    ) -> impl Iterator<Item = &T> {
        tracker
            .iter_alive()
            .exclude(exclude)
            .map(|node| &self[node])
    }

    pub unsafe fn clone_vec(&self) -> Vec<T> {
        self.0.clone()
    }
}

impl<T: Default + Clone> Index<Node> for NADVec<T> {
    type Output = T;
    fn index(&self, index: Node) -> &Self::Output {
        &self.0[index.idx()]
    }
}

impl<T: Default + Clone> IndexMut<Node> for NADVec<T> {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        &mut self.0[index.idx()]
    }
}

mod test {
    #[allow(unused_imports)]
    use super::{Node, NodeStatusTracker};

    #[test]
    fn alive_node_iter() {
        let mut tracker = NodeStatusTracker::new(10);
        tracker.delete(Node(0));
        tracker.delete(Node(1));
        tracker.delete(Node(2));
        tracker.delete(Node(5));
        tracker.delete(Node(8));

        let alive = vec![Node(3), Node(4), Node(6), Node(7), Node(9)];
        for node in alive.iter() {
            assert!(tracker.is_alive(*node));
        }

        assert_eq!(tracker.iter_alive().count(), alive.len());
        assert_eq!(alive, tracker.iter_alive().collect::<Vec<_>>());
    }
}
*/
