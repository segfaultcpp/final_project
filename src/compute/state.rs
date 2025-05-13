use crate::graph::{Graph, GraphDesc, GraphInfo};

#[derive(Clone, Debug)]
pub struct Iteration {
    pub graph: Graph,
    pub info: GraphInfo,
}

impl Iteration {
    pub(super) fn new(desc: GraphDesc) -> Self {
        let graph = Graph::from(desc);
        let info = GraphInfo::new(&graph.tracker);

        Self { graph, info }
    }
}

impl From<(Graph, GraphInfo)> for Iteration {
    fn from((graph, info): (Graph, GraphInfo)) -> Self {
        Self { graph, info }
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub alpha: f64,
    pub ks: Vec<f64>,
    pub beta_deltas: Vec<f64>,
    iterations: Vec<Iteration>,
    current_iter: isize,
}

impl State {
    pub(super) fn new() -> Self {
        Self {
            alpha: 3.0,
            ..Default::default()
        }
    }

    pub(super) fn add_iter(&mut self, iter: Iteration) {
        self.iterations.push(iter);
    }

    pub fn get(&self) -> &Iteration {
        &self.iterations[self.current_iter()]
    }

    pub(super) fn get_mut(&mut self) -> &mut Iteration {
        &mut self.iterations[self.current_iter as usize]
    }

    pub(super) fn pop(&mut self) {
        self.iterations.pop();
        self.current_iter = 0;
    }

    pub fn iter_count(&self) -> usize {
        self.iterations.len()
    }

    pub fn at(&self, idx: usize) -> &Iteration {
        assert!(idx < self.iter_count());
        &self.iterations[idx]
    }

    pub fn set_current_iter(&mut self, idx: usize) {
        assert!(idx < self.iter_count());
        self.current_iter = idx as isize;
    }

    pub fn current_iter(&self) -> usize {
        self.current_iter as usize
    }

    pub fn next_by(&mut self, n: isize) -> &Iteration {
        self.current_iter = self.current_iter.overflowing_add(n).0 % self.iterations.len() as isize;
        self.get()
    }

    pub fn next(&mut self) -> &Iteration {
        self.next_by(1)
    }

    pub fn prev(&mut self) -> &Iteration {
        self.next_by(-1)
    }
}
