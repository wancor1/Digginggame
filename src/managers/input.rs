use crate::events::CameraMoveIntent;
use macroquad::prelude::*;
use crate::constants::{INITIAL_CAMERA_DELAY_SECONDS, CAMERA_MOVE_INTERVAL_SECONDS};

pub struct InputHandler {
    key_pressed_start: [f64; 512], // Simple array map for key timings
    last_move_time: [f64; 512], // Time when the last move intent was sent for a key
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            key_pressed_start: [0.0; 512],
            last_move_time: [0.0; 512],
        }
    }

    pub fn handle_camera_movement(&mut self) -> Vec<CameraMoveIntent> {
        let mut intents = Vec::new();
        let current_time = get_time();

        self.process_key_input(
            KeyCode::W,
            CameraMoveIntent::Up,
            &mut intents,
            current_time,
        );
        self.process_key_input(
            KeyCode::S,
            CameraMoveIntent::Down,
            &mut intents,
            current_time,
        );
        self.process_key_input(
            KeyCode::A,
            CameraMoveIntent::Left,
            &mut intents,
            current_time,
        );
        self.process_key_input(
            KeyCode::D,
            CameraMoveIntent::Right,
            &mut intents,
            current_time,
        );

        intents
    }

    fn process_key_input(
        &mut self,
        key_code: KeyCode,
        intent_type: CameraMoveIntent,
        intents: &mut Vec<CameraMoveIntent>,
        current_time: f64,
    ) {
        let key_index = key_code as usize;

        if is_key_down(key_code) {
            if self.key_pressed_start[key_index] == 0.0 {
                // Key was just pressed
                self.key_pressed_start[key_index] = current_time;
                self.last_move_time[key_index] = current_time;
                intents.push(intent_type);
            } else {
                // Key is being held down
                let time_since_pressed = current_time - self.key_pressed_start[key_index];
                let time_since_last_move = current_time - self.last_move_time[key_index];

                if time_since_pressed >= INITIAL_CAMERA_DELAY_SECONDS
                    && time_since_last_move >= CAMERA_MOVE_INTERVAL_SECONDS
                {
                    intents.push(intent_type);
                    self.last_move_time[key_index] = current_time;
                }
            }
        } else {
            // Key is not pressed, reset timers
            self.key_pressed_start[key_index] = 0.0;
            self.last_move_time[key_index] = 0.0;
        }
    }
}
