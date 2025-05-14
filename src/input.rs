use cgmath::{InnerSpace, Point3, SquareMatrix, Vector4};
use winit::event::{ElementState, MouseScrollDelta};

use crate::{
    app::{WINDOW_HEIGHT, WINDOW_WIDTH},
    world::WorldData,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(usize)]
pub enum Key {
    Lmb = 0,
    Rmb,
    W,
    A,
    S,
    D,
    ArrowUp,
    ArrowDown,
    Lctrl,
    Lalt,
    Count,
}

pub struct Input {
    keys: [ElementState; Key::Count as usize],
    pub mouse_wheel: MouseScrollDelta,
    pub mouse_motion: (f64, f64),
    pub mouse_position: (f64, f64),
}

impl Input {
    pub fn new() -> Self {
        Input {
            keys: [ElementState::Released; Key::Count as usize],
            mouse_wheel: MouseScrollDelta::LineDelta(0.0, 0.0),
            mouse_motion: (0.0, 0.0),
            mouse_position: (0.0, 0.0),
        }
    }

    pub fn mouse_to_world(&self, world: &WorldData) -> Point3<f32> {
        let (x, y) = (
            self.mouse_position.0 as f32,
            WINDOW_HEIGHT as f32 - self.mouse_position.1 as f32,
        );

        let ray_start = Vector4::<f32>::new(
            (x / WINDOW_WIDTH as f32 - 0.5) * 2.0,
            (y / WINDOW_HEIGHT as f32 - 0.5) * 2.0,
            -1.0,
            1.0,
        );
        let ray_end = Vector4::<f32>::new(
            (x / WINDOW_WIDTH as f32 - 0.5) * 2.0,
            (y / WINDOW_HEIGHT as f32 - 0.5) * 2.0,
            0.0,
            1.0,
        );

        let inv_vp = (world.projection * world.camera.view_mat())
            .invert()
            .unwrap();

        let mut ray_start = inv_vp * ray_start;
        ray_start /= ray_start.w;

        let mut ray_end = inv_vp * ray_end;
        ray_end /= ray_end.w;

        let ray = (ray_end - ray_start).normalize();
        //world.camera.position() + Vector3::new(ray.x, ray.y, ray.z)
        Point3::new(ray.x, ray.y, ray.z)
    }

    pub fn update(&mut self) {
        self.mouse_wheel = MouseScrollDelta::LineDelta(0.0, 0.0);
        self.mouse_motion = (0.0, 0.0);
    }

    pub fn set(&mut self, key: Key, state: ElementState) {
        let key = key as usize;
        self.keys[key] = state;
    }

    pub fn get(&self, key: Key) -> ElementState {
        self.keys[key as usize]
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        self.get(key) == ElementState::Pressed
    }

    pub fn is_released(&self, key: Key) -> bool {
        self.get(key) == ElementState::Released
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}
