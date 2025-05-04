use draw::NodeDrawItem;
use framebuffer::{Framebuffer, FramebufferBuilder};
use glow::HasContext;
use mesh::Meshes;
use shader::Shaders;

use crate::{
    AppState,
    world::{WorldData, mat4_to_vec},
};

pub mod draw;

mod framebuffer;
mod mesh;
mod shader;

pub struct Renderer {
    meshes: Meshes,
    shaders: Shaders,
    fbo: Framebuffer,
}

impl Renderer {
    pub fn new(gl: &glow::Context) -> Self {
        unsafe {
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
        }
        Self {
            meshes: Meshes::new(gl),
            shaders: Shaders::new(gl),
            fbo: FramebufferBuilder::new(crate::app::WINDOW_WIDTH, crate::app::WINDOW_HEIGHT)
                .with_depth(true)
                .build(gl),
        }
    }

    pub fn render(&self, gl: &glow::Context, app_state: &AppState) {
        let tracker = &app_state.compute.state().get().graph.tracker;
        let items = NodeDrawItem::build(&app_state.world, tracker);

        self.scene_pass(gl, &app_state.world, items.as_slice());
    }

    fn scene_pass(&self, gl: &glow::Context, world: &WorldData, items: &[NodeDrawItem]) {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
        }

        self.meshes.node.bind(gl);
        self.shaders.node.bind(gl);

        self.set_uniforms(gl, &world);

        for item in items.iter() {
            item.set_uniforms(gl, self.shaders.node);
            unsafe {
                gl.draw_elements(
                    glow::TRIANGLES,
                    self.meshes.node.index_count.unwrap() as i32,
                    glow::UNSIGNED_INT,
                    0,
                );
            }
        }

        self.shaders.node.unbind(gl);
        self.meshes.node.unbind(gl);

        unsafe {
            gl.disable(glow::DEPTH_TEST);
        }
    }

    fn fullscreen_pass(&self, gl: &glow::Context) {}

    fn set_uniforms(&self, gl: &glow::Context, world: &WorldData) {
        let view_proj_loc = self.shaders.node.uniform_location(gl, "ViewProj").unwrap();
        let view_proj = mat4_to_vec(world.projection * world.camera.view_mat());

        let camera_pos_loc = self.shaders.node.uniform_location(gl, "CameraPos").unwrap();
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
}
