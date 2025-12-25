use crate::constants::*;
use macroquad::prelude::*;
use macroquad::texture::Texture2D;

pub struct SelectBlock {
    selection_effect_start_time: f64,
    is_effect_active: bool,
    block_coords: Option<(f32, f32)>,
    preview_sprite: Option<Rect>,
    is_valid: bool,
}

impl Default for SelectBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectBlock {
    pub fn new() -> Self {
        Self {
            selection_effect_start_time: 0.0,
            is_effect_active: false,
            block_coords: None,
            preview_sprite: None,
            is_valid: true,
        }
    }

    pub fn update(
        &mut self,
        hovered_block_coords: Option<(f32, f32)>,
        preview_sprite: Option<Rect>,
        is_valid: bool,
    ) {
        if let Some(coords) = hovered_block_coords {
            if !self.is_effect_active || self.block_coords != Some(coords) {
                self.selection_effect_start_time = get_time();
            }
            self.is_effect_active = true;
            self.block_coords = Some(coords);
        } else {
            self.is_effect_active = false;
            self.block_coords = None;
        }
        self.preview_sprite = preview_sprite;
        self.is_valid = is_valid;
    }

    pub fn get_block_coords(&self) -> Option<(f32, f32)> {
        self.block_coords
    }

    pub fn draw(&mut self, camera_x: f32, camera_y: f32, atlas: &Texture2D) {
        if !self.is_effect_active {
            return;
        }

        let (world_block_x, world_block_y) = self.block_coords.unwrap();

        let screen_x = (world_block_x - camera_x).round();
        let screen_y = (world_block_y - camera_y).round();
        let elapsed = get_time() - self.selection_effect_start_time;

        if let Some(sprite) = self.preview_sprite {
            let color = if self.is_valid {
                Color::new(1.0, 1.0, 1.0, 0.4)
            } else {
                Color::new(1.0, 0.3, 0.3, 0.5)
            };
            draw_texture_ex(
                atlas,
                screen_x,
                screen_y,
                color,
                DrawTextureParams {
                    source: Some(sprite),
                    ..Default::default()
                },
            );
        }

        if elapsed > SELECTION_PULSE_DURATION {
            self.selection_effect_start_time = get_time();
        }

        if elapsed <= (SELECTION_PULSE_DURATION / 2.0) {
            draw_texture_ex(
                atlas,
                screen_x,
                screen_y,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_SELECT_NORMAL),
                    ..Default::default()
                },
            );
        } else {
            draw_texture_ex(
                atlas,
                screen_x - SELECTION_ENLARGE_AMOUNT,
                screen_y - SELECTION_ENLARGE_AMOUNT,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_SELECT_LARGE),
                    dest_size: Some(vec2(
                        BLOCK_SIZE + SELECTION_ENLARGE_AMOUNT * 2.0,
                        BLOCK_SIZE + SELECTION_ENLARGE_AMOUNT * 2.0,
                    )),
                    ..Default::default()
                },
            );
        }
    }
}
