use macroquad::prelude::*;

pub struct InputHandler {
    pub camera_x: f32,
    pub camera_y: f32,
    key_pressed_start: [f64; 512], // Simple array map for key timings
}

impl InputHandler {
    const CAMERA_SPEED_NORMAL: f32 = 8.0;
    const CAMERA_SPEED_FAST: f32 = 16.0;

    pub fn new() -> Self {
        Self {
            camera_x: 0.0,
            camera_y: 0.0,
            key_pressed_start: [0.0; 512],
        }
    }

    pub fn handle_camera_movement(&mut self) -> bool {
        let mut moved = false;
        let speed = if is_key_down(KeyCode::LeftShift) {
            Self::CAMERA_SPEED_FAST
        } else {
            Self::CAMERA_SPEED_NORMAL
        };

        if is_key_down(KeyCode::W) {
            self.camera_y -= speed;
            moved = true;
        }
        if is_key_down(KeyCode::S) {
            self.camera_y += speed;
            moved = true;
        }
        if is_key_down(KeyCode::A) {
            self.camera_x -= speed;
            moved = true;
        }
        if is_key_down(KeyCode::D) {
            self.camera_x += speed;
            moved = true;
        }

        moved
    }
}
