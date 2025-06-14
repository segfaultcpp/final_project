use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use egui_glow::Painter;

use crate::world::{
    Material,
    desc::GraphDesc,
    editor::{EditorWorld, topology::Topology},
    node::Node,
};

use super::{MainTab, TabWindow, outliner::OutlinerWindow, viewport::ViewportWindow};

#[derive(Clone, Default)]
pub struct EditorState {
    pub world: EditorWorld,
    pub selected_nodes: Vec<Node>,
}

impl EditorState {
    fn new(desc: GraphDesc) -> Self {
        Self {
            world: desc.into(),
            selected_nodes: vec![],
        }
    }

    pub fn select_node(&mut self, node: Node) {
        self.selected_nodes.push(node);
        *self.world.material_mut(node) = Material::default().with_albedo([0.0, 1.0, 0.0]);
    }

    pub fn select_nodes(&mut self, nodes: impl Clone + Iterator<Item = Node>) {
        self.selected_nodes.extend(nodes.clone());
        for node in nodes {
            *self.world.material_mut(node) = Material::default().with_albedo([0.0, 1.0, 0.0]);
        }
    }

    pub fn throw_node(&mut self, idx: usize) {
        let node = self.selected_nodes.remove(idx);
        *self.world.material_mut(node) = Material::default();
    }

    pub fn throw_all_nodes(&mut self) {
        for node in self.selected_nodes.iter() {
            *self.world.material_mut(*node) = Material::default();
        }
        self.selected_nodes.clear();
    }

    pub fn spawn_single(&mut self) {
        let node = self.world.spawn_single();

        self.throw_all_nodes();
        self.select_node(node);
    }

    pub fn spawn(&mut self, topology: Topology, count: usize) {
        let nodes = self.world.spawn_topology(topology, count);

        self.throw_all_nodes();
        self.select_nodes(nodes.iter());
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
        let desc = GraphDesc::from(self.state.world.clone());
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
