use egui::{Button, Color32, DragValue, Frame, Modal};

use crate::world::Position;

use super::{
    TabWindow,
    editor::{EditorState, Topology},
};

pub struct OutlinerWindow {
    add_options_opened: bool,
    selected_topology: Topology,
    node_count: usize,
}

impl OutlinerWindow {
    pub fn new() -> Self {
        Self {
            add_options_opened: false,
            selected_topology: Topology::Net,
            node_count: 2,
        }
    }

    fn show_position_editor(&mut self, ui: &mut egui::Ui, pos: &mut Position) {
        egui::Grid::new("Position")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("x");
                ui.add(DragValue::new(&mut pos.0.x));
                ui.end_row();

                ui.label("y");
                ui.add(DragValue::new(&mut pos.0.y));
                ui.end_row();

                ui.label("z");
                ui.add(DragValue::new(&mut pos.0.z));
                ui.end_row();
            });
    }

    fn add_single_node(&mut self, state: &mut EditorState) {
        state.spawn_single();
    }

    fn show_add_options(
        &mut self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        state: &mut EditorState,
    ) {
        Modal::new(ui.id()).show(egui_ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Add new nodes");
            });

            ui.separator();

            ui.radio_value(&mut self.selected_topology, Topology::Net, "Net");
            ui.radio_value(&mut self.selected_topology, Topology::Line, "Line");
            ui.radio_value(&mut self.selected_topology, Topology::Ring, "Ring");
            ui.radio_value(&mut self.selected_topology, Topology::Star, "Star");

            let min = if self.selected_topology == Topology::Star {
                4
            } else if self.selected_topology == Topology::Ring {
                3
            } else {
                2
            };

            ui.add(
                egui::DragValue::new(&mut self.node_count)
                    .speed(1)
                    .range(min..=1000),
            );

            ui.separator();

            ui.vertical_centered(|ui| {
                if ui.button("Add").clicked() {
                    self.add_options_opened = false;
                    state.spawn(self.selected_topology, self.node_count);
                }
            })
        });
    }

    fn delete_nodes(&mut self, state: &mut EditorState) {}

    fn link_nodes(&mut self, state: &mut EditorState) {
        let selected = &state.selected_nodes;
        for (num, i) in selected.iter().enumerate() {
            for j in selected.iter().skip(num + 1) {
                state.adjacency.set(*i, *j);
                state.adjacency.set(*j, *i);
            }
        }
    }

    fn unlink_nodes(&mut self, state: &mut EditorState) {
        let selected = &state.selected_nodes;
        for (num, i) in selected.iter().enumerate() {
            for j in selected.iter().skip(num + 1) {
                state.adjacency.unset(*i, *j);
                state.adjacency.unset(*j, *i);
            }
        }
    }
}

impl TabWindow<EditorState> for OutlinerWindow {
    fn title(&self) -> egui::WidgetText {
        "Outliner".into()
    }

    fn show(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        ui.horizontal(|ui| {
            Frame::group(ui.style())
                .fill(Color32::DARK_GREEN)
                .corner_radius(3.0)
                .inner_margin(0.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .add(Button::new("➕").frame(false))
                            .on_hover_text("Add new node")
                            .clicked()
                        {
                            self.add_single_node(state);
                        }
                        ui.separator();

                        if ui.add(Button::new(":").frame(false)).clicked() {
                            self.add_options_opened = true;
                        }
                    });
                });

            if ui
                .button("🗑️")
                .on_hover_text("Delete selected nodes")
                .clicked()
            {
                self.delete_nodes(state);
            }

            ui.separator();

            if ui
                .button("🔗")
                .on_hover_text("Link selected nodes")
                .clicked()
            {
                self.link_nodes(state);
            }

            if ui
                .button("❌")
                .on_hover_text("Unlink selected nodes")
                .clicked()
            {
                self.unlink_nodes(state);
            }
        });

        let response = ui.separator();

        // Frame::group(ui.style()).show(ui, |ui| {
        let selected_nodes = &mut state.selected_nodes;
        for (i, _) in state.world.positions.iter_mut().enumerate().rev() {
            let response = ui.selectable_label(selected_nodes.contains(&i), format!("Node {i}"));

            if response.clicked() {
                if let Some((j, _)) = selected_nodes
                    .iter()
                    .enumerate()
                    .find(|(_, node)| **node == i)
                {
                    selected_nodes.remove(j);
                } else {
                    selected_nodes.push(i);
                }
            }
        }
        // });

        if self.add_options_opened {
            self.show_add_options(&response.ctx, ui, state);
        }
    }
}
