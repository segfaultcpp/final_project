use cgmath::Matrix4;
use glow::HasContext;

use crate::{
    graph::node::NodeStatusTracker,
    world::{Material, Position, WorldData, mat4_to_vec},
};

use super::shader::Shader;

pub(super) struct NodeDrawItem {
    pub position: Position,
    pub material: Material,
}

impl NodeDrawItem {
    pub(super) fn build(world: &WorldData, tracker: &NodeStatusTracker) -> Vec<Self> {
        tracker
            .iter_alive()
            .map(|i| Self {
                position: world.positions[i],
                material: world.materials[i],
            })
            .collect::<Vec<_>>()
    }

    pub(super) fn set_uniforms(&self, gl: &glow::Context, shader: Shader) {
        let NodeDrawItem {
            position: pos,
            material: _mat,
        } = self;

        let model_loc = shader.uniform_location(gl, "Model").unwrap();
        let model = mat4_to_vec(Matrix4::<f32>::from_translation(pos.0));

        let albedo_loc = shader.uniform_location(gl, "Albedo").unwrap();
        let albedo = self.material.albedo;

        let roughness_loc = shader.uniform_location(gl, "Roughness").unwrap();
        let roughness = self.material.roughness;

        let metallic_loc = shader.uniform_location(gl, "Metallic").unwrap();
        let metallic = self.material.metallic;

        let ao_loc = shader.uniform_location(gl, "Ao").unwrap();
        let ao = self.material.ao;

        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&model_loc), false, model.as_slice());
            gl.uniform_3_f32(Some(&albedo_loc), albedo.x, albedo.y, albedo.z);
            gl.uniform_1_f32(Some(&roughness_loc), roughness);
            gl.uniform_1_f32(Some(&metallic_loc), metallic);
            gl.uniform_1_f32(Some(&ao_loc), ao);
        }
    }
}
