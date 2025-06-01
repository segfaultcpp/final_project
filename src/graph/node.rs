use std::ops::{Index, IndexMut};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Node(u32);

impl Default for Node {
    fn default() -> Self {
        Self(u32::MAX)
    }
}

impl Node {
    pub unsafe fn new(idx: u32) -> Self {
        Self(idx)
    }

    pub fn as_idx(&self) -> usize {
        self.0 as usize
    }

    pub fn is_valid(&self) -> bool {
        self.0 != u32::MAX
    }
}

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
            node.as_idx() < self.nodes.len(),
            "Accessing invalid node(id = {node:?}). Maximum node id = {}",
            self.nodes.len() - 1
        );

        assert!(self.is_alive(node), "Trying to delete already deleted node");

        self.nodes[node.as_idx()] = false;
        self.alive -= 1;
    }

    pub fn is_alive(&self, node: Node) -> bool {
        assert!(
            node.as_idx() < self.nodes.len(),
            "Accessing invalid node(id = {node:?}). Maximum node id = {}",
            self.nodes.len() - 1
        );
        self.nodes[node.as_idx()]
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
        &self.0[index.as_idx()]
    }
}

impl<T: Default + Clone> IndexMut<Node> for NADVec<T> {
    fn index_mut(&mut self, index: Node) -> &mut Self::Output {
        &mut self.0[index.as_idx()]
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
