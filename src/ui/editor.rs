use std::ops::RangeInclusive;

use cgmath::Vector3;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use egui_glow::Painter;
use log::info;

use crate::{
    graph::{GraphDesc, NodeDesc},
    world::EditorWorld,
};

use super::{MainTab, TabWindow, outliner::OutlinerWindow, viewport::ViewportWindow};

/// Extensible Adjacency matrix
#[derive(Debug, Clone, Default)]
pub struct ExtAdjacency(Vec<Vec<bool>>);

impl ExtAdjacency {
    pub fn new(node_count: usize) -> Self {
        Self(vec![vec![false; node_count]; node_count])
    }

    pub fn set(&mut self, i: usize, j: usize) {
        assert_ne!(i, j, "You cannot access diagonal line of matrix.");
        assert!(i < self.0.len() && j < self.0.len());
        self.0[i][j] = true;
    }

    pub fn unset(&mut self, i: usize, j: usize) {
        assert_ne!(i, j, "You cannot access diagonal line of matrix.");
        assert!(i < self.0.len() && j < self.0.len());
        self.0[i][j] = false;
    }

    pub fn is_set(&self, i: usize, j: usize) -> bool {
        assert_ne!(i, j, "You cannot access diagonal line of matrix.");
        assert!(i < self.0.len() && j < self.0.len());
        self.0[i][j]
    }

    pub fn add_nodes(&mut self, count: usize) -> RangeInclusive<usize> {
        let old_size = self.0.len();
        for row in self.0.iter_mut() {
            row.extend((0..count).map(|_| false));
        }

        self.0
            .extend((0..count).map(|_| vec![false; old_size + count]));

        let begin = old_size;
        let end = self.0.len() - 1;

        begin..=end
    }

    pub fn add_node(&mut self) -> usize {
        self.add_nodes(1).next().unwrap()
    }

    pub fn node_count(&self) -> usize {
        self.0.len()
    }
}

impl From<GraphDesc> for ExtAdjacency {
    fn from(value: GraphDesc) -> Self {
        let count = value.node_count();
        let mut mat = Self::new(count);

        for NodeDesc {
            node_id: i, nodes, ..
        } in value.nodes().iter()
        {
            for j in nodes.iter() {
                assert_ne!(*i, *j);
                assert!(*i < count as u32);
                assert!(*j < count as u32);

                let i = *i as usize;
                let j = *j as usize;

                mat.set(i, j);
                mat.set(j, i);
            }
        }

        mat
    }
}

#[derive(Clone, Default)]
pub struct EditorState {
    pub world: EditorWorld,
    pub adjacency: ExtAdjacency,
    pub selected_nodes: Vec<usize>,
}

impl From<EditorState> for GraphDesc {
    fn from(value: EditorState) -> Self {
        let mut nodes = vec![];
        for (i, pos) in value.world.positions.iter().enumerate() {
            nodes.push(NodeDesc {
                node_id: i as u32,
                nodes: {
                    let mut nodes = vec![];

                    for j in 0..value.world.positions.len() {
                        if i == j {
                            continue;
                        }

                        if value.adjacency.is_set(i, j) {
                            nodes.push(j as u32);
                        }
                    }

                    nodes
                },
                position: pos.0.into(),
            })
        }
        Self { nodes }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub(super) enum Topology {
    Net,
    Ring,
    Line,
    Star,
}

impl Topology {
    const STEP: f32 = 6.0;

    fn positions(self, origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let origin = Vector3::new(origin.x, origin.y, 0.0);

        match self {
            Self::Net => Self::positions_net(origin, count),
            Self::Star => Self::positions_star(origin, count),
            Self::Line => Self::positions_line(origin, count),
            Self::Ring => Self::positions_ring(origin, count),
        }
    }

    fn positions_net(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let width = (count as f64).sqrt().ceil() as usize;
        let mut positions = vec![(0.0, 0.0, 0.0).into(); count];

        let mut row = 0;
        let mut column = 0;

        for (i, pos) in positions.iter_mut().enumerate() {
            if i != 0 && i % width == 0 {
                row += 1;
                column = 0;
            }

            let x = column as f32 * Self::STEP;
            let y = row as f32 * Self::STEP;

            *pos = (origin.x + x, origin.y + y, 0.0).into();
            column += 1;
        }

        positions
    }

    fn positions_star(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        todo!()
    }

    fn positions_line(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let mut positions = vec![(0.0, 0.0, 0.0).into(); count];

        let mut start = origin;
        for pos in positions.iter_mut() {
            *pos = start;
            start.x += Self::STEP;
        }

        positions
    }

    fn positions_ring(origin: Vector3<f32>, count: usize) -> Vec<Vector3<f32>> {
        let mut positions = vec![];
        let width = (count as f32 / 4.0).ceil() as usize;

        let mut idx = 0;
        let mut x_scale = 1;
        let mut y_scale = 0;

        let mut pos = origin;
        info!("count = {count}");
        while idx < count {
            if idx != 0 && idx % width == 0 {
                let temp = y_scale;
                y_scale = -x_scale;
                x_scale = temp;
            }

            pos.x += x_scale as f32 * Self::STEP;
            pos.y += y_scale as f32 * Self::STEP;
            positions.push(pos);

            idx += 1;
        }

        positions
    }

    fn connect(self, adjacency: &mut ExtAdjacency, nodes: RangeInclusive<usize>) {
        match self {
            Self::Net => Self::connect_net(adjacency, nodes),
            Self::Star => Self::connect_star(adjacency, nodes),
            Self::Line => Self::connect_line(adjacency, nodes),
            Self::Ring => Self::connect_ring(adjacency, nodes),
        }
    }

    fn connect_net(adjacency: &mut ExtAdjacency, nodes: RangeInclusive<usize>) {
        let start = *nodes.start();

        for i in nodes.clone() {
            for j in nodes.clone().skip(i - start + 1) {
                adjacency.set(i, j);
                adjacency.set(j, i);
            }
        }
    }

    fn connect_star(adjacency: &mut ExtAdjacency, nodes: RangeInclusive<usize>) {
        todo!()
    }

    fn connect_line(adjacency: &mut ExtAdjacency, nodes: RangeInclusive<usize>) {
        for i in nodes.skip(1) {
            let j = i - 1;
            adjacency.set(i, j);
            adjacency.set(j, i);
        }
    }

    fn connect_ring(adjacency: &mut ExtAdjacency, nodes: RangeInclusive<usize>) {
        for i in nodes.clone().skip(1) {
            let j = i - 1;
            adjacency.set(i, j);
            adjacency.set(j, i);
        }

        let start = *nodes.start();
        let end = *nodes.end();

        adjacency.set(start, end);
        adjacency.set(end, start);
    }
}

impl EditorState {
    fn new(desc: GraphDesc) -> Self {
        Self {
            world: EditorWorld::new(desc.clone()),
            adjacency: desc.into(),
            selected_nodes: vec![],
        }
    }

    pub fn spawn_single(&mut self) {
        self.selected_nodes.clear();

        let mut camera_pos = self.world.camera.position_as_vec();
        camera_pos.z = 0.0;

        let world_id = self.world.spawn_single(camera_pos);
        let adj_id = self.adjacency.add_node();
        assert_eq!(world_id, adj_id);

        self.selected_nodes.push(world_id);
    }

    pub fn spawn(&mut self, topology: Topology, count: usize) {
        self.selected_nodes.clear();

        let camera_pos = self.world.camera.position_as_vec();
        let positions = topology.positions(camera_pos, count);

        let world_ids = self.world.spawn(positions.iter());
        let adj_ids = self.adjacency.add_nodes(count);

        assert_eq!(world_ids.clone().count(), adj_ids.clone().count());
        assert_eq!(
            world_ids
                .clone()
                .zip(adj_ids.clone())
                .filter(|(a, b)| a != b)
                .count(),
            0
        );

        topology.connect(&mut self.adjacency, adj_ids);

        self.selected_nodes.extend(world_ids);
    }
}

pub struct EditorTab {
    state: EditorState,
    dock_state: DockState<Box<dyn TabWindow<EditorState>>>,
}

impl EditorTab {
    pub fn new(painter: &mut Painter) -> Self {
        let mut dock_state: DockState<Box<dyn TabWindow<EditorState>>> =
            DockState::new(vec![Box::new(ViewportWindow::new(painter))]);
        let surface = dock_state.main_surface_mut();

        surface.split_left(
            NodeIndex::root(),
            0.2,
            vec![Box::new(OutlinerWindow::new())],
        );

        Self {
            dock_state,
            state: Default::default(),
        }
    }
}

impl MainTab for EditorTab {
    fn title(&self) -> egui::WidgetText {
        "Editor".into()
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut EditorTabViewer(&mut self.state));
    }

    fn update(&mut self, delta: std::time::Duration) {
        for tab in self.dock_state.iter_all_tabs_mut() {
            tab.1.update(&mut self.state, delta);
        }
    }

    fn save_to(&self, file: String) {
        let desc = GraphDesc::from(self.state.clone());
        let desc = toml::to_string(&desc).unwrap();

        std::fs::write(file, desc.as_str()).unwrap();
    }

    fn open_file(&mut self, file: &str) {
        let desc = std::fs::read_to_string(file).unwrap();
        let desc: GraphDesc = toml::from_str(desc.as_str()).unwrap();

        self.state = EditorState::new(desc);
    }
}

struct EditorTabViewer<'a>(&'a mut EditorState);

impl TabViewer for EditorTabViewer<'_> {
    type Tab = Box<dyn TabWindow<EditorState>>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.show(ui, self.0);
    }
}
