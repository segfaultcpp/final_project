use cgmath::Matrix4;
use glow::HasContext;

use super::shader::Shader;
use crate::world::{Material, PROJECTION, Position, camera::Camera, mat4_to_vec};

pub struct NodeDrawItem {
    pub position: Position,
    pub material: Material,
    pub idx: i32,
}

impl NodeDrawItem {
    pub fn build<'a>(
        positions: impl Iterator<Item = &'a Position>,
        materials: impl Iterator<Item = &'a Material>,
    ) -> Vec<Self> {
        positions
            .zip(materials)
            .enumerate()
            .map(|(i, (p, m))| Self {
                position: *p,
                material: *m,
                idx: i as i32,
            })
            .collect()
    }

    pub(super) fn set_per_pass_uniforms(gl: &glow::Context, shader: Shader, camera: &Camera) {
        let view_proj_loc = shader.uniform_location(gl, "ViewProj").unwrap();
        let view_proj = mat4_to_vec(*PROJECTION * camera.view_mat());

        let camera_pos_loc = shader.uniform_location(gl, "CameraPos").unwrap();
        let camera_pos = camera.position();

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
pub struct EdgeDrawItem {
    pub positions: [Position; 2],
}

impl EdgeDrawItem {
    pub(super) fn set_per_pass_uniforms(gl: &glow::Context, shader: Shader, camera: &Camera) {
        let view_proj_loc = shader.uniform_location(gl, "ViewProj").unwrap();
        let view_proj = mat4_to_vec(*PROJECTION * camera.view_mat());

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
