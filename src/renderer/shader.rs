use glow::HasContext;
use log::info;

use crate::unbind_on_drop;

use super::{ScopedBind, binding::UnbindOnDrop};

#[derive(Copy, Clone)]
pub(super) struct Shader(pub glow::Program);

impl Shader {
    fn new(gl: &glow::Context, name: &str) -> Self {
        let shaders_folder = "data/shaders/";
        let shader_path = String::from(shaders_folder) + name;
        let vs_path = shader_path.clone() + "_vs.glsl";
        let fs_path = shader_path + "_fs.glsl";

        info!("Loading vertex shader in path {vs_path}");
        info!("Loading fragment shader in path {fs_path}");

        let vs_src = std::fs::read_to_string(vs_path).unwrap();
        let fs_src = std::fs::read_to_string(fs_path).unwrap();

        unsafe {
            let vs = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vs, &vs_src);
            gl.compile_shader(vs);

            if !gl.get_shader_compile_status(vs) {
                panic!(
                    "Failed to compile vertex shader!\n{}",
                    gl.get_shader_info_log(vs)
                );
            }

            let fs = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(fs, &fs_src);
            gl.compile_shader(fs);

            if !gl.get_shader_compile_status(fs) {
                panic!(
                    "Failed to compile fragment shader!\n{}",
                    gl.get_shader_info_log(fs)
                );
            }

            let program = gl.create_program().unwrap();
            gl.attach_shader(program, vs);
            gl.attach_shader(program, fs);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            gl.delete_shader(vs);
            gl.delete_shader(fs);

            Self(program)
        }
    }

    pub(super) fn uniform_location(
        self,
        gl: &glow::Context,
        name: &str,
    ) -> Option<glow::UniformLocation> {
        unsafe { gl.get_uniform_location(self.0, name) }
    }

    pub(super) fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.0));
        }
    }
}

impl ScopedBind for Shader {
    fn scoped_bind<'a>(&self, gl: &'a glow::Context) -> UnbindOnDrop<'a, Self> {
        unsafe {
            gl.use_program(Some(self.0));
        }

        unbind_on_drop!(gl)
    }

    fn unbind(gl: &glow::Context) {
        unsafe {
            gl.use_program(None);
        }
    }
}

pub(super) struct Shaders {
    pub node: Shader,
    pub fullscreen: Shader,
}

impl Shaders {
    pub(super) fn new(gl: &glow::Context) -> Self {
        Self {
            node: Shader::new(gl, "node"),
            fullscreen: Shader::new(gl, "fullscreen"),
        }
    }
}
