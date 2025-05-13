use cgmath::{Matrix4, Quaternion, Vector3};
use egui::{Label, Slider};
use egui_extras::{Column, TableBuilder};
use transform_gizmo_egui::{
    Gizmo, GizmoConfig, GizmoExt, GizmoMode, GizmoOrientation, math::Transform, mint,
};

use crate::{
    AppState,
    compute::state::Iteration,
    world::{Material, Position},
};

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
        let selected_node = app_state.selected_node;
        let mut alpha = app_state.compute.state().alpha;

        egui::Window::new("Compute Info")
            .resizable(true)
            .default_width(1000.0)
            .show(egui_ctx, |ui| {
                ui.add(Slider::new(&mut alpha, 0.0..=5.0).text("Alpha"));

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
                    .columns(Column::auto(), 4)
                    .header(40.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Node Idx");
                        });
                        header.col(|ui| {
                            ui.strong("Z");
                        });
                        header.col(|ui| {
                            ui.strong("Betweenness");
                        });
                        header.col(|ui| {
                            ui.strong("Capacity");
                        });
                    })
                    .body(|mut body| {
                        let Iteration { graph, info } = app_state.compute.state().get();
                        for i in graph.tracker.iter_alive() {
                            body.row(30.0, |mut row| {
                                if let Some(selected_node) = selected_node {
                                    row.set_selected(selected_node == i);
                                }

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

                                row.col(|ui| {
                                    let c = info.capacity[i];
                                    ui.label(c.to_string());
                                });
                            });
                        }
                    });
            });

        app_state.compute.state_mut().alpha = alpha;
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

    fn show_gizmo(&self, ui: &mut egui::Ui, app_state: &mut AppState) {
        let Some(selected_node) = app_state.selected_node else {
            return;
        };

        let view_mat: [[f64; 4]; 4] = app_state.world.camera.view_mat().cast().unwrap().into();
        let proj_mat: [[f64; 4]; 4] = app_state.world.projection.cast().unwrap().into();

        let mut gizmo = Gizmo::default();
        gizmo.update_config(GizmoConfig {
            view_matrix: view_mat.into(),
            projection_matrix: proj_mat.into(),
            modes: GizmoMode::TranslateXZ.into(),
            orientation: GizmoOrientation::Local,
            ..Default::default()
        });

        let translation: mint::Vector3<f64> = app_state.world.positions[selected_node]
            .0
            .cast()
            .unwrap()
            .into();

        let rotation: mint::Quaternion<f64> =
            Quaternion::<f64>::from_sv(0.0, (0.0, 1.0, 0.0).into()).into();
        let scale: mint::Vector3<f64> = Vector3::<f64>::new(1.0, 1.0, 1.0).into();

        let mut transform =
            Transform::from_scale_rotation_translation(scale, rotation, translation);

        if let Some((result, new_transforms)) = gizmo.interact(ui, &[transform]) {
            for (new_transform, transform) in
                new_transforms.iter().zip(std::iter::once(&mut transform))
            {
                *transform = *new_transform;
            }
        }

        let translation: Vector3<f64> = translation.into();
        let translation: Vector3<f32> = translation.cast().unwrap();

        app_state.world.positions[selected_node] = Position(translation);
    }
}
