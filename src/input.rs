use cgmath::Point3;
use winit::event::{ElementState, MouseScrollDelta};

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

    pub fn mouse_to_world(&self) -> Point3<f32> {
        todo!()
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
