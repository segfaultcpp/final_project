use std::sync::Arc;

use binding::ScopedBind;
use draw::{EdgeDrawItem, NodeDrawItem};
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
    gl: Arc<glow::Context>,
    meshes: Meshes,
    shaders: Shaders,
    framebuffer: Framebuffer,
}

impl Renderer {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        unsafe {
            gl.enable(glow::MULTISAMPLE);
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
        }

        let meshes = Meshes::new(gl.as_ref());
        let shaders = Shaders::new(gl.as_ref());
        let framebuffer =
            FramebufferBuilder::new(crate::app::WINDOW_WIDTH, crate::app::WINDOW_HEIGHT)
                .with_depth(true)
                .build(gl.as_ref());

        Self {
            gl,
            meshes,
            shaders,
            framebuffer,
        }
    }

    pub fn render(&self, app_state: &AppState) {
        let tracker = &app_state.compute.state().get().graph.tracker;
        let node_items = NodeDrawItem::build(&app_state.world, tracker);
        let edge_items =
            EdgeDrawItem::build(&app_state.world, &app_state.compute.state().get().graph);

        let gl = self.gl.as_ref();

        unsafe {
            gl.viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
        }

        {
            self.framebuffer.bind(gl);
            self.scene_pass(
                gl,
                &app_state.world,
                node_items.as_slice(),
                edge_items.as_slice(),
            );
            Framebuffer::unbind(gl);
        }

        self.fullscreen_pass(gl);
    }

    pub fn idx_from_stencil(&self, (x, y): (f64, f64)) -> usize {
        self.framebuffer.read_pixel_from_stencil(
            self.gl.as_ref(),
            (x as i64 as i32, (WINDOW_HEIGHT as f64 - y) as i64 as i32),
        ) as usize
    }

    fn scene_pass(
        &self,
        gl: &glow::Context,
        world: &WorldData,
        node_items: &[NodeDrawItem],
        edge_items: &[EdgeDrawItem],
    ) {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);

            gl.enable(glow::STENCIL_TEST);
            gl.stencil_mask(0xff);
            gl.stencil_op(glow::KEEP, glow::KEEP, glow::REPLACE);

            gl.clear_color(1.0, 1.0, 1.0, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT | glow::STENCIL_BUFFER_BIT);

            gl.line_width(5.0);
        }

        self.meshes.node.bind(gl);

        self.shaders.edge.bind(gl);
        EdgeDrawItem::set_per_pass_uniforms(gl, self.shaders.edge, world);
        for item in edge_items.iter() {
            item.set_uniforms(gl, self.shaders.edge);
            unsafe {
                gl.draw_arrays(glow::LINES, 0, 2);
            }
        }

        self.shaders.node.bind(gl);
        NodeDrawItem::set_per_pass_uniforms(gl, self.shaders.node, world);
        for item in node_items.iter() {
            item.set_uniforms(gl, self.shaders.node);
            unsafe {
                gl.stencil_func(glow::ALWAYS, item.idx + 1, 0xff);

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
            gl.line_width(1.0);
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
}
