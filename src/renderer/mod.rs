use binding::ScopedBind;
use draw::NodeDrawItem;
use framebuffer::{Framebuffer, FramebufferBuilder};
use glow::HasContext;
use log::info;
use mesh::{Mesh, Meshes};
use shader::{Shader, Shaders};

use crate::{
    AppState,
    app::{WINDOW_HEIGHT, WINDOW_WIDTH},
    world::{WorldData, mat4_to_vec},
};

pub mod draw;

mod binding;
mod framebuffer;
mod mesh;
mod shader;

pub struct Renderer {
    meshes: Meshes,
    shaders: Shaders,
    framebuffer: Framebuffer,
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
            framebuffer: FramebufferBuilder::new(
                crate::app::WINDOW_WIDTH,
                crate::app::WINDOW_HEIGHT,
                // 1920, 1080,
            )
            .with_depth(true)
            .build(gl),
        }
    }

    pub fn render(&self, gl: &glow::Context, app_state: &AppState) {
        let tracker = &app_state.compute.state().get().graph.tracker;
        let items = NodeDrawItem::build(&app_state.world, tracker);

        unsafe {
            gl.viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
        }

        {
            self.framebuffer.bind(gl);
            self.scene_pass(gl, &app_state.world, items.as_slice());
            Framebuffer::unbind(gl);
        }

        self.fullscreen_pass(gl);
    }

    fn scene_pass(&self, gl: &glow::Context, world: &WorldData, items: &[NodeDrawItem]) {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);

            gl.enable(glow::STENCIL_TEST);
            gl.stencil_mask(0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);

            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);
        }

        self.meshes.node.bind(gl);
        self.shaders.node.bind(gl);

        self.set_uniforms(gl, world);

        for item in items.iter() {
            item.set_uniforms(gl, self.shaders.node);
            unsafe {
                gl.stencil_func(glow::ALWAYS, item.idx, 0xff);

                gl.draw_elements(
                    glow::TRIANGLES,
                    self.meshes.node.index_count.unwrap() as i32,
                    glow::UNSIGNED_INT,
                    0,
                );
            }
        }

        Mesh::unbind(gl);
        Shader::unbind(gl);

        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.disable(glow::STENCIL_TEST);
            gl.stencil_mask(0);
        }
    }

    fn fullscreen_pass(&self, gl: &glow::Context) {
        self.meshes.fullscreen.bind(gl);
        self.shaders.fullscreen.bind(gl);

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(self.framebuffer.color_buffer));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Mesh::unbind(gl);
        Shader::unbind(gl);
    }

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
