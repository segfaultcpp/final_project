use bytemuck::{NoUninit, cast_slice};
use glow::HasContext;
use log::info;

trait Vertex: Clone + Copy + NoUninit {}

#[repr(C)]
#[derive(Clone, Copy, NoUninit)]
struct NodeVertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl Vertex for NodeVertex {}

impl NodeVertex {
    fn load_sphere() -> (Vec<Self>, Option<Vec<u32>>) {
        let (models, _) =
            tobj::load_obj("data/meshes/sphere.obj", &tobj::LoadOptions::default()).unwrap();

        let tobj::Model { mesh, name } = models.into_iter().next().unwrap();
        info!("Loading model {name}");
        info!("Face count = {}", mesh.face_arities.len());
        info!("Index count = {}", mesh.indices.len());
        info!("Position count = {}", mesh.positions.len());
        info!("Normal count = {}", mesh.normals.len());

        let vtx_count = mesh.positions.len() / 3;
        let mut vertices = Vec::with_capacity(vtx_count);

        for i in 0..vtx_count {
            let j = i * 3;
            vertices.push(Self {
                position: (
                    mesh.positions[j],
                    mesh.positions[j + 1],
                    mesh.positions[j + 2],
                )
                    .into(),
                normal: (mesh.normals[j], mesh.normals[j + 1], mesh.normals[j + 2]).into(),
            });
        }

        (vertices, Some(mesh.indices))
    }
}

#[repr(C)]
#[derive(Copy, Clone, NoUninit)]
struct FullscreenTriangleVertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
}

impl Vertex for FullscreenTriangleVertex {}

impl FullscreenTriangleVertex {
    fn new() -> Vec<Self> {
        vec![
            Self {
                position: [-1.0, 3.0],
                tex_coord: [0.0, -1.0],
            },
            Self {
                position: [-1.0, -1.0],
                tex_coord: [0.0, 1.0],
            },
            Self {
                position: [3.0, -1.0],
                tex_coord: [2.0, 1.0],
            },
        ]
    }
}

#[derive(Copy, Clone)]
pub(super) struct Mesh {
    pub vao: glow::VertexArray,
    pub vbo: glow::Buffer,
    pub ibo: Option<glow::Buffer>,
    pub vertex_count: usize,
    pub index_count: Option<usize>,
}

impl Mesh {
    fn new<T: Vertex>(gl: &glow::Context, (vertices, indices): (Vec<T>, Option<Vec<u32>>)) -> Self {
        unsafe {
            let vao = gl.create_vertex_array().unwrap();
            let vbo = gl.create_buffer().unwrap();

            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                cast_slice(&vertices[0..]),
                glow::STATIC_DRAW,
            );

            let ibo = if let Some(ref indices) = indices {
                let ibo = gl.create_buffer().unwrap();
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ibo));
                gl.buffer_data_u8_slice(
                    glow::ELEMENT_ARRAY_BUFFER,
                    cast_slice(&indices[0..]),
                    glow::STATIC_DRAW,
                );
                Some(ibo)
            } else {
                None
            };

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, size_of::<T>() as i32, 0);
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                size_of::<T>() as i32,
                size_of::<[f32; 3]>() as i32,
            );
            gl.enable_vertex_attrib_array(1);

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Mesh {
                vao,
                vbo,
                ibo,
                vertex_count: vertices.len(),
                index_count: indices.map(|indices| indices.len()),
            }
        }
    }

    pub(super) fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
        }
    }

    pub(super) fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(None);
        }
    }
}

#[derive(Copy, Clone)]
pub(super) struct Meshes {
    pub node: Mesh,
    pub fullscreen: Mesh,
}

impl Meshes {
    pub(super) fn new(gl: &glow::Context) -> Self {
        Self {
            node: Mesh::new(gl, NodeVertex::load_sphere()),
            fullscreen: Mesh::new(gl, (FullscreenTriangleVertex::new(), None)),
        }
    }
}
