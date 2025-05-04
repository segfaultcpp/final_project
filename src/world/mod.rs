use camera::Camera;
use cgmath::{Array, Matrix4, Point3, Vector3};

use crate::{
    app::{WINDOW_HEIGHT, WINDOW_WIDTH},
    compute::state::Iteration,
    graph::{
        GraphInfo,
        node::{NADVec, NodeStatusTracker},
    },
};

pub mod camera;

#[derive(Clone, Copy)]
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
    pub fn update_albedo(&mut self, b: f64, min_b: f64, max_b: f64) {
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

pub struct WorldData {
    pub positions: NADVec<Position>,
    pub materials: NADVec<Material>,
    pub camera: Camera,
    pub projection: Matrix4<f32>,
}

impl WorldData {
    pub fn new(tracker: &NodeStatusTracker) -> Self {
        Self {
            positions: Self::init_positions(tracker),
            materials: Self::init_materials(tracker),
            camera: Camera::new(),
            projection: cgmath::perspective(cgmath::Deg(50.0), 16.0 / 9.0, 0.001, 100.0),
        }
    }

    fn init_positions(tracker: &NodeStatusTracker) -> NADVec<Position> {
        let width = (tracker.node_count() as f64).sqrt().ceil() as usize;
        let mut positions = NADVec::<Position>::new(tracker);

        let mut row = 0;
        let mut column = 0;
        let step = 6.0;
        for (i, node) in tracker.iter_alive().enumerate() {
            if i != 0 && i % width == 0 {
                row += 1;
                column = 0;
            }

            let x = column as f32 * step;
            let y = row as f32 * step;

            positions[node].0 = (x, y, 0.0).into();
            column += 1;
        }

        positions
    }

    fn init_materials(tracker: &NodeStatusTracker) -> NADVec<Material> {
        NADVec::<Material>::new(tracker)
    }

    pub fn update_materials(&mut self, iter: &Iteration) {
        let Iteration { graph, info } = iter;
        for i in graph.tracker.iter_alive() {
            self.materials[i].update_albedo(
                info.betweenness[i],
                info.min_betweenness,
                info.max_betweenness,
            );
        }
    }
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
