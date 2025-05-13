use cgmath::Matrix4;
use glow::HasContext;
use log::info;

use crate::{
    AppState,
    compute::state::Iteration,
    graph::{Graph, node::NodeStatusTracker},
    world::{Material, Position, WorldData, mat4_to_vec},
};

use super::shader::Shader;

pub(super) struct NodeDrawItem {
    pub position: Position,
    pub material: Material,
    pub idx: i32,
}

impl NodeDrawItem {
    pub(super) fn build(world: &WorldData, tracker: &NodeStatusTracker) -> Vec<Self> {
        tracker
            .iter_alive()
            .map(|i| Self {
                position: world.positions[i],
                material: world.materials[i],
                idx: i.as_idx() as i32,
            })
            .collect::<Vec<_>>()
    }

    pub(super) fn set_per_pass_uniforms(gl: &glow::Context, shader: Shader, world: &WorldData) {
        let view_proj_loc = shader.uniform_location(gl, "ViewProj").unwrap();
        let view_proj = mat4_to_vec(world.projection * world.camera.view_mat());

        let camera_pos_loc = shader.uniform_location(gl, "CameraPos").unwrap();
        let camera_pos = world.camera.position();

        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&view_proj_loc), false, view_proj.as_slice());
            gl.uniform_3_f32(
                Some(&camera_pos_loc),
                camera_pos.x,
                camera_pos.y,
                camera_pos.z,
            );
        }
    }

    pub(super) fn set_uniforms(&self, gl: &glow::Context, shader: Shader) {
        let NodeDrawItem {
            position: pos,
            material: mat,
            idx: _idx,
        } = self;

        let model_loc = shader.uniform_location(gl, "Model").unwrap();
        let model = mat4_to_vec(Matrix4::<f32>::from_translation(pos.0));

        let albedo_loc = shader.uniform_location(gl, "Albedo").unwrap();
        let albedo = mat.albedo;

        let roughness_loc = shader.uniform_location(gl, "Roughness").unwrap();
        let roughness = mat.roughness;

        let metallic_loc = shader.uniform_location(gl, "Metallic").unwrap();
        let metallic = mat.metallic;

        let ao_loc = shader.uniform_location(gl, "Ao").unwrap();
        let ao = mat.ao;

        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&model_loc), false, model.as_slice());
            gl.uniform_3_f32(Some(&albedo_loc), albedo.x, albedo.y, albedo.z);
            gl.uniform_1_f32(Some(&roughness_loc), roughness);
            gl.uniform_1_f32(Some(&metallic_loc), metallic);
            gl.uniform_1_f32(Some(&ao_loc), ao);
        }
    }
}

#[derive(Debug)]
pub(super) struct EdgeDrawItem {
    pub positions: [Position; 2],
}

impl EdgeDrawItem {
    pub(super) fn build(world: &WorldData, graph: &Graph) -> Vec<Self> {
        let mut ret = vec![];

        for i in graph.tracker.iter_alive() {
            for j in graph.tracker.iter_alive().exclude(i) {
                if graph.is_adjacent(i, j) {
                    ret.push(EdgeDrawItem {
                        positions: [world.positions[i], world.positions[j]],
                    });
                }
            }
        }
        ret
    }

    pub(super) fn set_per_pass_uniforms(gl: &glow::Context, shader: Shader, world: &WorldData) {
        let view_proj_loc = shader.uniform_location(gl, "ViewProj").unwrap();
        let view_proj = mat4_to_vec(world.projection * world.camera.view_mat());

        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&view_proj_loc), false, view_proj.as_slice());
        }
    }

    pub(super) fn set_uniforms(&self, gl: &glow::Context, shader: Shader) {
        let [a, b] = self.positions;

        let position0 = shader.uniform_location(gl, "Positions[0]").unwrap();
        let position1 = shader.uniform_location(gl, "Positions[1]").unwrap();

        unsafe {
            gl.uniform_3_f32(Some(&position0), a.x(), a.y(), a.z());
            gl.uniform_3_f32(Some(&position1), b.x(), b.y(), b.z());
        }
    }
}
