use cgmath::{InnerSpace, Matrix4, Point3, Vector2, Vector3};

#[derive(Clone, Copy)]
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy)]
pub struct Camera {
    position: Point3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,
    right: Vector3<f32>,
    world_up: Vector3<f32>,

    yaw: f32,
    pitch: f32,

    movement_speed: f32,
    mouse_sensitivity: f32,

    mouse_captured: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    const DEFAULT_YAW: f32 = -90.0;
    const DEFAULT_PITCH: f32 = 0.0;
    const DEFAULT_SPEED: f32 = 10.0;
    const DEFAULT_SENSITIVITY: f32 = 0.1;

    pub fn new() -> Self {
        let mut camera = Self {
            position: (0.0, 0.0, 35.0).into(),
            front: (0.0, 0.0, -1.0).into(),
            up: (0.0, 1.0, 0.0).into(),
            right: (0.0, 0.0, 0.0).into(),
            world_up: (0.0, 1.0, 0.0).into(),
            yaw: Self::DEFAULT_YAW,
            pitch: Self::DEFAULT_PITCH,
            movement_speed: Self::DEFAULT_SPEED,
            mouse_sensitivity: Self::DEFAULT_SENSITIVITY,
            mouse_captured: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
        };

        camera.update_camera_vectors();
        camera
    }

    pub fn view_mat(&self) -> Matrix4<f32> {
        Matrix4::<f32>::look_at_rh(self.position, self.position + self.front, self.up)
    }

    pub fn view_mat_f64(&self) -> Matrix4<f64> {
        let position = Point3::<f64>::new(
            self.position.x as f64,
            self.position.y as f64,
            self.position.z as f64,
        );

        let front = Vector3::<f64>::new(
            self.front.x as f64,
            self.front.y as f64,
            self.front.z as f64,
        );

        let up = Vector3::<f64>::new(self.up.x as f64, self.up.y as f64, self.up.z as f64);

        Matrix4::<f64>::look_at_rh(position, position + front, up)
    }

    pub fn position(&self) -> Point3<f32> {
        self.position
    }

    pub fn position_as_vec(&self) -> Vector3<f32> {
        let p = self.position;
        (p.x, p.y, p.z).into()
    }

    pub fn process_keyboard(&mut self, movement: CameraMovement, delta: f32) {
        let velocity = self.movement_speed * delta;

        use CameraMovement::*;
        match movement {
            Forward => self.position += self.front * velocity,
            Backward => self.position -= self.front * velocity,
            Left => self.position -= self.right * velocity,
            Right => self.position += self.right * velocity,
            Up => self.position += self.world_up * velocity,
            Down => self.position -= self.world_up * velocity,
        }
    }

    pub fn process_mouse_motion(&mut self, (x, y): (f32, f32), delta: f32) {
        const SPEED: f32 = 4.0;
        self.position -= self.right * x * delta * SPEED;
        self.position += self.world_up * y * delta * SPEED;
    }

    pub fn process_mouse_zoom(&mut self, y: f32, delta: f32) {
        const SPEED: f32 = 4.0;
        self.position -= self.front * y * delta * SPEED;
    }

    pub fn process_mouse_movement(&mut self, mut offset: Vector2<f32>) {
        offset *= self.mouse_sensitivity;

        self.yaw += offset.x;
        self.pitch += offset.y;

        self.pitch = self.pitch.min(89.0);
        self.pitch = self.pitch.max(-89.0);

        self.update_camera_vectors();
    }

    fn update_camera_vectors(&mut self) {
        self.front = {
            use cgmath::{Angle, Deg};

            let mut front = Vector3::<f32>::new(0.0, 0.0, 0.0);
            front.x = Deg::cos(Deg(self.yaw)) * Deg::cos(Deg(self.pitch));
            front.y = Deg::sin(Deg(self.pitch));
            front.z = Deg::sin(Deg(self.yaw)) * Deg::cos(Deg(self.pitch));
            front.normalize()
        };

        self.right = self.front.cross(self.world_up).normalize();
        self.up = self.right.cross(self.front).normalize();
    }
}
