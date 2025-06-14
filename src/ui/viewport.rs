use std::time::Duration;

use egui::{PointerButton, Pos2, Rect, Response, Sense, Vec2, load::SizedTexture};
use egui_glow::Painter;
use transform_gizmo_egui::{
    Gizmo, GizmoConfig, GizmoExt, GizmoMode, GizmoOrientation, math::Transform,
};

use crate::{
    app::{WINDOW_HEIGHT, WINDOW_WIDTH},
    renderer::{
        Renderer,
        framebuffer::{Framebuffer, FramebufferBuilder},
    },
    world::{
        PROJECTION, World,
        camera::Camera,
        node::{IsNode, Node, PhysNode, PhysNodeIdx},
    },
};

use super::{TabWindow, editor::EditorState};

#[derive(Copy, Clone)]
enum InputAction {
    Drag(PointerButton, Vec2),
    Click(PointerButton, Vec2),
}

pub struct ViewportWindow {
    framebuffer: Framebuffer,
    color_buffer_id: egui::TextureId,

    actions: Vec<InputAction>,

    gizmo: Gizmo,
}

impl ViewportWindow {
    pub fn new(painter: &mut Painter) -> Self {
        let framebuffer =
            FramebufferBuilder::new(crate::app::WINDOW_WIDTH, crate::app::WINDOW_HEIGHT)
                .with_depth(true)
                .build(Renderer::gl());

        let color_buffer_id = painter.register_native_texture(framebuffer.color_buffer);

        Self {
            framebuffer,
            color_buffer_id,
            actions: vec![],
            gizmo: Gizmo::default(),
        }
    }

    fn render(&mut self, ui: &mut egui::Ui, world: &impl World) -> Response {
        let size = ui.min_size();

        Renderer::render(
            world.camera(),
            &self.framebuffer,
            (size.x as i32, size.y as i32),
            world.build_node_draw_items(),
            world.build_edge_draw_items(),
        );

        ui.add(
            egui::Image::from_texture(SizedTexture::new(self.color_buffer_id, size))
                .uv(Rect::from_min_max(Pos2::new(0.0, 1.0), Pos2::new(1.0, 0.0)))
                .sense(Sense::CLICK | Sense::DRAG),
        )
    }

    fn update(&mut self, state: &mut EditorState, delta: Duration) {
        for action in self.actions.iter() {
            if let InputAction::Click(PointerButton::Primary, pos) = action {
                self.process_click_action(*pos, state);
            }

            self.process_camera(*action, &mut state.world.camera, delta);
        }

        self.actions.clear();
    }

    fn process_input(&mut self, response: Response) {
        let motion = response.drag_motion();
        if response.dragged_by(PointerButton::Primary) {
            self.actions
                .push(InputAction::Drag(PointerButton::Primary, motion));
        } else if response.dragged_by(PointerButton::Secondary) {
            self.actions
                .push(InputAction::Drag(PointerButton::Secondary, motion));
        }

        let Some(pos) = response.hover_pos() else {
            return;
        };

        let pos = ((pos - response.rect.min) / response.rect.size())
            * Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32);

        if response.clicked() {
            self.actions
                .push(InputAction::Click(PointerButton::Primary, pos));
        } else if response.secondary_clicked() {
            self.actions
                .push(InputAction::Click(PointerButton::Secondary, pos));
        }
    }

    fn process_click_action(&self, pos: Vec2, state: &mut EditorState) {
        let idx =
            Renderer::idx_from_stencil(&self.framebuffer, (pos.x, WINDOW_HEIGHT as f32 - pos.y));

        if idx != 0 {
            let phys_node = unsafe { PhysNode::new(idx as PhysNodeIdx - 1) };
            if let Some((idx, _)) = state
                .selected_nodes
                .iter()
                .enumerate()
                .find(|(_, node)| state.world.phys_node(**node) == phys_node)
            {
                state.throw_node(idx);
            } else {
                state.select_node(state.world.glob_node(phys_node));
            }
        } else {
            state.throw_all_nodes();
        }
    }

    fn process_camera(&self, action: InputAction, camera: &mut Camera, delta: Duration) {
        match action {
            InputAction::Drag(PointerButton::Primary, motion) => {
                camera.process_mouse_motion((motion.x, motion.y), delta.as_secs_f32());
            }
            InputAction::Drag(PointerButton::Secondary, motion) => {
                camera.process_mouse_zoom(motion.y, delta.as_secs_f32());
            }

            _ => {}
        }
    }

    fn show_gizmo(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        if state.selected_nodes.is_empty() || ui.input(|i| i.modifiers.ctrl) {
            return;
        }

        let view_mat: [[f64; 4]; 4] = state.world.camera.view_mat().cast().unwrap().into();
        let proj_mat: [[f64; 4]; 4] = PROJECTION.cast().unwrap().into();

        let view_mat: cgmath::mint::ColumnMatrix4<f64> = view_mat.into();
        let proj_mat: cgmath::mint::ColumnMatrix4<f64> = proj_mat.into();

        self.gizmo.update_config(GizmoConfig {
            view_matrix: view_mat.into(),
            projection_matrix: proj_mat.into(),
            modes: GizmoMode::all_translate(),
            orientation: GizmoOrientation::Local,
            ..Default::default()
        });

        let translations: Vec<cgmath::mint::Vector3<f64>> = state
            .selected_nodes
            .iter()
            .map(|node| state.world.position(*node).0.cast().unwrap().into())
            .collect();

        let rotation: cgmath::mint::Quaternion<f64> =
            cgmath::Quaternion::<f64>::from_sv(0.0, (0.0, 1.0, 0.0).into()).into();

        let scale: cgmath::mint::Vector3<f64> = cgmath::Vector3::<f64>::new(1.0, 1.0, 1.0).into();

        let mut transforms = translations
            .into_iter()
            .map(|t| Transform::from_scale_rotation_translation(scale, rotation, t))
            .collect::<Vec<_>>();

        if let Some((_result, new_transforms)) = self.gizmo.interact(ui, transforms.as_slice()) {
            for (new_transform, transform) in new_transforms.iter().zip(transforms.iter_mut()) {
                *transform = *new_transform;
            }

            for (node, transform) in state.selected_nodes.iter().zip(transforms.iter()) {
                let pos = transform.translation;
                state.world.position_mut(*node).0 =
                    cgmath::Vector3::new(pos.x as f32, pos.y as f32, 0.0);
            }
        }
    }
}

impl TabWindow<EditorState> for ViewportWindow {
    fn title(&self) -> egui::WidgetText {
        "Viewport".into()
    }

    fn show(&mut self, ui: &mut egui::Ui, state: &mut EditorState) {
        let response = self.render(ui, &state.world);
        self.process_input(response);

        self.show_gizmo(ui, state);
    }

    fn update(&mut self, state: &mut EditorState, delta: Duration) {
        self.update(state, delta);
    }
}
/*
impl TabWindow<SummaryState> for ViewportWindow {
    fn title(&self) -> egui::WidgetText {
        "Viewport".into()
    }

    fn show(&mut self, ui: &mut egui::Ui, state: &mut SummaryState) {}
    fn update(&mut self, state: &mut SummaryState, delta: Duration) {}
}
*/
