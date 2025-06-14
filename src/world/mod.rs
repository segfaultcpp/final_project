use camera::Camera;
use cgmath::{Matrix4, Point3, Vector3};
use lazy_static::lazy_static;

use crate::renderer::draw::{EdgeDrawItem, NodeDrawItem};

pub mod camera;
pub mod desc;
pub mod editor;
pub mod node;
// pub mod summary;

#[derive(Clone, Copy, Debug)]
pub struct Position(pub Vector3<f32>);

impl Position {
    pub fn x(self) -> f32 {
        self.0.x
    }

    pub fn y(self) -> f32 {
        self.0.y
    }

    pub fn z(self) -> f32 {
        self.0.z
    }
}

#[derive(Clone, Copy)]
pub struct Material {
    pub albedo: Point3<f32>,
    pub roughness: f32,
    pub metallic: f32,
    pub ao: f32,
}

impl Material {
    pub fn with_albedo(mut self, albedo: [f32; 3]) -> Self {
        self.albedo = albedo.into();
        self
    }

    pub fn with_roughness(mut self, roughness: f32) -> Self {
        self.roughness = roughness;
        self
    }

    pub fn with_metallic(mut self, metallic: f32) -> Self {
        self.metallic = metallic;
        self
    }

    pub fn with_ao(mut self, ao: f32) -> Self {
        self.ao = ao;
        self
    }

    pub fn update_albedo(&mut self, b: f64, min_b: f64, max_b: f64) {
        assert!(b <= max_b && b >= min_b);

        let k = (b - min_b) / (max_b - min_b);
        self.albedo = Self::mix_color((0.0, 1.0, 0.0).into(), (1.0, 0.0, 0.0).into(), k as f32);
    }

    fn mix_color(mut a: Point3<f32>, mut b: Point3<f32>, k: f32) -> Point3<f32> {
        a *= 1.0 - k;
        b *= k;
        a + Vector3::<f32>::new(b.x, b.y, b.z)
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            albedo: (1.0, 0.0, 0.0).into(),
            roughness: 0.25,
            metallic: 0.0,
            ao: 1.0,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(Vector3::new(0.0, 0.0, 0.0))
    }
}

lazy_static! {
    pub static ref PROJECTION: Matrix4<f32> =
        cgmath::perspective(cgmath::Deg(50.0), 16.0 / 9.0, 0.001, 100000.0);
}

pub trait World {
    fn camera(&self) -> &Camera;
    fn build_node_draw_items(&self) -> Vec<NodeDrawItem>;
    fn build_edge_draw_items(&self) -> Vec<EdgeDrawItem>;
}

// TODO: kostil
pub fn mat4_to_vec(mat: Matrix4<f32>) -> Vec<f32> {
    let mat: [[f32; 4]; 4] = mat.into();
    let mut vec = Vec::with_capacity(16);

    for i in 0..4 {
        for j in 0..4 {
            vec.push(mat[i][j]);
        }
    }

    vec
}
