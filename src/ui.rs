use egui::{Label, Slider};
use egui_extras::{Column, TableBuilder};
use log::info;

use crate::{AppState, compute::state::Iteration, world::Material};

#[derive(Default)]
pub struct UiState {
    selected_item: usize,
    material: Material,
}

impl UiState {
    pub fn show(&mut self, egui_ctx: &egui::Context, app_state: &mut AppState) {
        self.show_compute_info(egui_ctx, app_state);
        self.show_material_editor(egui_ctx, app_state);
    }

    fn show_compute_info(&mut self, egui_ctx: &egui::Context, app_state: &mut AppState) {
        app_state
            .compute
            .state_mut()
            .set_current_iter(self.selected_item);

        let compute_state = app_state.compute.state();

        egui::Window::new("Compute Info")
            .resizable([true, false])
            .default_width(1000.0)
            .show(egui_ctx, |ui| {
                egui::ComboBox::from_label("Iterations")
                    .selected_text(format!("Iteration {}", self.selected_item))
                    .show_ui(ui, |ui| {
                        for i in 0..compute_state.iter_count() {
                            ui.selectable_value(
                                &mut self.selected_item,
                                i,
                                format!("Iteration {i}"),
                            );
                        }
                    });

                TableBuilder::new(ui)
                    .striped(true)
                    .columns(Column::auto(), 3)
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Node Idx");
                        });
                        header.col(|ui| {
                            ui.strong("Z");
                        });
                        header.col(|ui| {
                            ui.strong("Betweenness");
                        });
                    })
                    .body(|mut body| {
                        let Iteration { graph, info } = app_state.compute.state().get();
                        for i in graph.tracker.iter_alive() {
                            body.row(18.0, |mut row| {
                                row.col(|ui| {
                                    ui.label(i.as_idx().to_string());
                                });

                                row.col(|ui| {
                                    let z = info.zs[i];
                                    ui.label(format!("{z:.3}"));
                                });

                                row.col(|ui| {
                                    let b = info.betweenness[i];
                                    ui.label(b.to_string());
                                });
                            });
                        }
                    });
            });
    }

    fn show_material_editor(&mut self, egui_ctx: &egui::Context, app_state: &mut AppState) {
        for i in app_state.compute.state().get().graph.tracker.iter_alive() {
            app_state.world.materials[i] = self.material;
        }

        egui::Window::new("Material Editor")
            .resizable([true, false])
            .default_open(false)
            .show(egui_ctx, |ui| {
                egui::Grid::new("Material Editor grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.add(Label::new("Roughness"));
                        ui.add(Slider::new(&mut self.material.roughness, 0.0..=1.0));
                        ui.end_row();

                        ui.add(Label::new("Metallic"));
                        ui.add(Slider::new(&mut self.material.metallic, 0.0..=1.0));
                        ui.end_row();

                        ui.add(Label::new("Ambient Occlusion"));
                        ui.add(Slider::new(&mut self.material.ao, 0.0..=1.0));
                        ui.end_row();
                    });
            });
    }
}
