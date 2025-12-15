use crate::events::CameraMoveIntent;
use macroquad::prelude::*;

pub struct InputHandler {
    key_pressed_start: [f64; 512], // Simple array map for key timings
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            key_pressed_start: [0.0; 512],
        }
    }

    pub fn handle_camera_movement(&mut self) -> Vec<CameraMoveIntent> {
        let mut intents = Vec::new();

        if is_key_down(KeyCode::W) {
            intents.push(CameraMoveIntent::Up);
        }
        if is_key_down(KeyCode::S) {
            intents.push(CameraMoveIntent::Down);
        }
        if is_key_down(KeyCode::A) {
            intents.push(CameraMoveIntent::Left);
        }
        if is_key_down(KeyCode::D) {
            intents.push(CameraMoveIntent::Right);
        }

        intents
    }
}
