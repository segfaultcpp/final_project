use crate::graph::Graph;

use std::fmt;

use super::{
    mat::Mat,
    node::{Node, NodeStatusTracker},
};

#[derive(Clone, Debug, Default)]
pub struct PathFinder {
    costs: Mat<i32>,
    paths: Vec<Mat<bool>>,
}

impl PathFinder {
    pub fn new(node_count: usize) -> Self {
        Self {
            costs: Mat::new(node_count),
            paths: vec![Mat::new(node_count); node_count],
        }
    }

    pub fn cost(&self, i: Node, j: Node) -> i32 {
        self.costs[(i, j)]
    }

    pub fn contains(&self, path: (Node, Node), node: Node) -> bool {
        self.paths[node.idx()][path]
    }

    pub fn find_shortest_path_for(
        &mut self,
        tracker: &NodeStatusTracker,
        adj: &Mat<bool>,
        src: Node,
    ) -> Option<()> {
        let mut spt = vec![false; tracker.node_count()];
        let mut dist = vec![i32::MAX; tracker.node_count()];

        dist[src.idx()] = 0;

        let min_dist = |spt: &[bool], dist: &[i32]| {
            let mut min_node = Node::default();
            let mut min_path = i32::MAX;

            for node in tracker.iter_alive() {
                let idx = node.idx();
                if !spt[idx] && dist[idx] < min_path {
                    min_path = dist[idx];
                    min_node = node;
                }
            }

            if min_node.is_valid() {
                Some(min_node)
            } else {
                None
            }
        };

        for _ in 0..tracker.alive() {
            let min_node = min_dist(&spt, &dist)?;
            let min_idx = min_node.idx();
            assert!(min_node.is_valid());

            spt[min_idx] = true;

            for j in tracker.iter_alive() {
                if !spt[j.idx()]
                    && adj[(min_node, j)]
                    && dist[min_idx] != i32::MAX
                    && dist[min_idx] + Graph::CONNECTION_COST < dist[j.idx()]
                {
                    dist[j.idx()] = dist[min_idx] + Graph::CONNECTION_COST;
                }
            }
        }

        for target in tracker.iter_alive() {
            if target == src {
                continue;
            }

            let path = Self::reconstruct_path(tracker, adj, dist.as_slice(), src, target);
            for node in path.iter() {
                self.paths[node.idx()].set(src, target);
                self.paths[node.idx()].set(target, src);
            }
        }

        for node in tracker.iter_alive().exclude(src) {
            self.costs[(src, node)] = dist[node.idx()];
        }

        Some(())
    }

    fn reconstruct_path(
        tracker: &NodeStatusTracker,
        adj: &Mat<bool>,
        dist: &[i32],
        src: Node,
        mut target: Node,
    ) -> Vec<Node> {
        let mut path = vec![target];

        let mut weight = dist[target.idx()];

        while target != src {
            for node in tracker.iter_alive().exclude(target) {
                if adj[(node, target)] {
                    let diff = weight - Graph::CONNECTION_COST;
                    if diff == dist[node.idx()] {
                        weight = diff;
                        target = node;
                        path.push(target);
                    }
                }
            }
        }

        path
    }
}

impl fmt::Display for PathFinder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "costs:\n{}", self.costs)?;
        for (i, path) in self.paths.iter().enumerate() {
            writeln!(f, "node {i}:\n{path}")?;
        }

        Ok(())
    }
}
