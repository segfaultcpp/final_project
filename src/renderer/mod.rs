use std::sync::{Arc, Once};

use binding::ScopedBind;
use draw::{EdgeDrawItem, NodeDrawItem};
use egui::Rect;
use framebuffer::Framebuffer;
use glow::HasContext;
use mesh::{Mesh, Meshes};
use shader::{Shader, Shaders};

use crate::{
    app::{WINDOW_HEIGHT, WINDOW_WIDTH},
    world::camera::Camera,
};

pub mod draw;

mod binding;
pub mod framebuffer;
mod mesh;
mod shader;

pub struct Renderer {
    gl: Arc<glow::Context>,
    meshes: Meshes,
    shaders: Shaders,
}

static mut RENDERER_RESOURCES: Option<Renderer> = None;
static INIT_RENDERER_ONCE: Once = Once::new();

impl Renderer {
    pub fn init(gl: Arc<glow::Context>) {
        INIT_RENDERER_ONCE.call_once(|| {
            let meshes = Meshes::new(gl.as_ref());
            let shaders = Shaders::new(gl.as_ref());

            unsafe {
                RENDERER_RESOURCES = Some(Self {
                    gl,
                    meshes,
                    shaders,
                });
            }
        });
    }

    fn get_renderer_resources<'a>() -> &'a Self {
        unsafe {
            let Some(ref res) = RENDERER_RESOURCES else {
                panic!("Initialize Renderer before use!");
            };

            res
        }
    }

    pub fn gl<'a>() -> &'a glow::Context {
        let res = Self::get_renderer_resources();
        res.gl.as_ref()
    }

    pub fn render(
        camera: &Camera,
        framebuffer: &Framebuffer,
        viewport: (i32, i32),
        node_items: Vec<NodeDrawItem>,
        edge_items: Vec<EdgeDrawItem>,
    ) {
        let res = Self::get_renderer_resources();
        let gl = res.gl.as_ref();

        unsafe {
            // gl.viewport(0, 0, viewport.0, viewport.1);
            gl.viewport(0, 0, WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32);
        }

        {
            framebuffer.bind(gl);
            res.scene_pass(gl, camera, node_items.as_slice(), edge_items.as_slice());
            Framebuffer::unbind(gl);
        }

        res.fullscreen_pass(gl, framebuffer);
    }

    pub fn idx_from_stencil(framebuffer: &Framebuffer, (x, y): (f32, f32)) -> usize {
        let res = Self::get_renderer_resources();
        framebuffer.read_pixel_from_stencil(res.gl.as_ref(), (x as i32, y as i32)) as usize
    }

    fn scene_pass(
        &self,
        gl: &glow::Context,
        camera: &Camera,
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
        EdgeDrawItem::set_per_pass_uniforms(gl, self.shaders.edge, camera);
        for item in edge_items.iter() {
            item.set_uniforms(gl, self.shaders.edge);
            unsafe {
                gl.draw_arrays(glow::LINES, 0, 2);
            }
        }

        self.shaders.node.bind(gl);
        NodeDrawItem::set_per_pass_uniforms(gl, self.shaders.node, camera);
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

    fn fullscreen_pass(&self, gl: &glow::Context, framebuffer: &Framebuffer) {
        self.meshes.fullscreen.bind(gl);
        self.shaders.fullscreen.bind(gl);

        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(framebuffer.color_buffer));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        Mesh::unbind(gl);
        Shader::unbind(gl);
    }
}
