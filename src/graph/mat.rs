use std::{
    fmt,
    ops::{Index, IndexMut},
};

use super::{
    GraphDesc, NodeDesc,
    node::{Node, NodeStatusTracker},
};

#[derive(Debug, Clone, Default)]
pub struct Mat<T: Default + Clone + fmt::Display + ToString> {
    array: Vec<T>,
    node_count: usize,
}

impl<T: Default + Clone + fmt::Display + ToString> Mat<T> {
    pub fn new(node_count: usize) -> Self {
        Self {
            array: vec![<T as Default>::default(); node_count * node_count],
            node_count,
        }
    }

    pub fn delete(&mut self, tracker: &NodeStatusTracker, i: Node) {
        for j in tracker.iter_alive() {
            if let Some(value) = self.get_mut(j, i) {
                *value = <T as Default>::default();
            }
        }
    }

    pub fn row(&self, node: Node) -> Row<'_, T> {
        let start = self.node_count * node.as_idx();
        Row {
            row: &self.array[start..start + self.node_count],
            row_id: node.as_idx(),
        }
    }

    pub fn get(&self, i: Node, j: Node) -> Option<&T> {
        if i != j {
            Some(&self.array[self.node_count * i.as_idx() + j.as_idx()])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, i: Node, j: Node) -> Option<&mut T> {
        if i != j {
            Some(&mut self.array[self.node_count * i.as_idx() + j.as_idx()])
        } else {
            None
        }
    }
}

impl Mat<bool> {
    pub fn set(&mut self, i: Node, j: Node) {
        self[(i, j)] = true;
    }

    pub fn unset(&mut self, i: Node, j: Node) {
        self[(i, j)] = false;
    }

    pub fn is_set(&self, i: Node, j: Node) -> bool {
        self[(i, j)]
    }
}

impl From<GraphDesc> for Mat<bool> {
    fn from(value: GraphDesc) -> Self {
        let count = value.node_count();
        let mut mat = Self::new(count);

        for NodeDesc { node_id: i, nodes } in value.nodes().iter() {
            for j in nodes.iter() {
                assert_ne!(*i, *j);
                assert!(*i < count as u32);
                assert!(*j < count as u32);

                let (i, j) = unsafe { (Node::new(*i), Node::new(*j)) };

                mat.set(i, j);
                mat.set(j, i);
            }
        }

        mat
    }
}

impl<T: Default + Clone + fmt::Display + ToString> fmt::Display for Mat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.node_count {
            write!(f, "[").unwrap();
            for j in 0..self.node_count {
                let (i, j) = unsafe { (Node::new(i as u32), Node::new(j as u32)) };
                write!(
                    f,
                    "\t{}",
                    if let Some(n) = self.get(i, j) {
                        n.to_string()
                    } else {
                        String::from("*")
                    }
                )?;
            }
            writeln!(f, " ]")?;
        }

        Ok(())
    }
}

impl<T: Default + Clone + fmt::Display + ToString> Index<(Node, Node)> for Mat<T> {
    type Output = T;

    fn index(&self, (i, j): (Node, Node)) -> &Self::Output {
        self.get(i, j)
            .expect("You cannot access diagonal line of matrix.")
    }
}

impl<T: Default + Clone + fmt::Display + ToString> IndexMut<(Node, Node)> for Mat<T> {
    fn index_mut(&mut self, (i, j): (Node, Node)) -> &mut Self::Output {
        self.get_mut(i, j)
            .expect("You cannot access diagonal line of matrix.")
    }
}

#[derive(Debug, Clone)]
pub struct Row<'a, T> {
    row: &'a [T],
    row_id: usize,
}

impl<T> Row<'_, T> {
    pub fn get(&self, i: Node) -> Option<&T> {
        if i.as_idx() != self.row_id {
            Some(&self.row[i.as_idx()])
        } else {
            None
        }
    }

    pub fn iter(&self) -> RowIter<'_, T> {
        RowIter {
            iter: self.row.iter(),
            row_id: self.row_id as u32,
            current_column: 0,
        }
    }
}

impl<T> Index<Node> for Row<'_, T> {
    type Output = T;

    fn index(&self, i: Node) -> &Self::Output {
        self.get(i)
            .expect("You cannot access diagonal line of matrix.")
    }
}

#[derive(Debug, Clone)]
pub struct RowIter<'a, T> {
    iter: std::slice::Iter<'a, T>,
    row_id: u32,
    current_column: u32,
}

impl<'a, T> Iterator for RowIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.row_id != self.current_column {
            self.iter.next()
        } else {
            None
        }
    }
}

mod test {
    use crate::graph::node::{Node, NodeStatusTracker};

    use super::Mat;

    #[test]
    fn mat_test() {
        let node_count = 5;
        let mut tracker = NodeStatusTracker::new(node_count);
        let mut mat_i32 = Mat::<i32>::new(node_count);

        for i in 0..node_count {
            unsafe {
                assert!(
                    mat_i32
                        .get(Node::new(i as u32), Node::new(i as u32))
                        .is_none()
                );
            }
        }

        for i in 0..node_count {
            for j in 0..node_count {
                unsafe {
                    if let Some(value) = mat_i32.get_mut(Node::new(i as u32), Node::new(j as u32)) {
                        let idx = (i * node_count + j) as i32;
                        *value = idx;

                        assert_eq!(mat_i32[(Node::new(i as u32), Node::new(j as u32))], idx);
                    }
                }
            }
        }

        unsafe {
            let deleted = &[Node::new(2), Node::new(4)];
            for &node in deleted.iter() {
                tracker.delete(node);
                mat_i32.delete(&tracker, node);
            }

            assert_eq!(
                tracker.iter_alive().collect::<Vec<_>>(),
                vec![Node::new(0), Node::new(1), Node::new(3)]
            );

            for node in tracker.iter_alive() {
                assert_eq!(mat_i32[(node, deleted[0])], 0);
                assert_eq!(mat_i32[(node, deleted[1])], 0);
            }
        }
    }
}
