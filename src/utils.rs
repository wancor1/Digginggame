use crate::constants::{BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, FONT_SIZE};
use macroquad::prelude::*;

pub fn estimate_text_width(text: &str, font: Option<&Font>) -> f32 {
    let dimensions = measure_text(text, font.copied(), FONT_SIZE as u16, 1.0);
    dimensions.width
}

pub fn calculate_text_center_position(
    box_width: f32,
    box_height: f32,
    text_content: &str,
    font: Option<&Font>,
) -> (f32, f32) {
    if text_content.is_empty() {
        return (box_width / 2.0, (box_height - FONT_SIZE) / 2.0);
    }
    let text_dims = measure_text(text_content, font.copied(), FONT_SIZE as u16, 1.0);
    let text_x = (box_width - text_dims.width) / 2.0;
    // Macroquad text drawing is baseline-based usually, but here we approximate center
    let text_y = (box_height + text_dims.height) / 2.0;
    // Fix: measure_text returns height as ascent+descent.
    // If we draw at y, that's baseline.
    // For centering, we often want y = box_y + (box_h - text_h) / 2.0 + ascent.
    // Simplifying for now to match python logic which was top-left based.
    // Python: text_y = (box_height - FONT_SIZE) / 2
    // Rust Macroquad: draw_text(..., y) where y is baseline.
    // Let's return the Y for the baseline.

    (text_x, (box_height / 2.0) + (text_dims.height / 3.0))
}

pub fn world_to_chunk_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let chunk_x = (world_x / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
    let chunk_y = (world_y / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
    (chunk_x, chunk_y)
}

pub fn world_to_relative_in_chunk_coords(world_x: f32, world_y: f32) -> (usize, usize) {
    let (cx, cy) = world_to_chunk_coords(world_x, world_y);
    // Python math.floor handles negatives correctly, Rust cast to i32 does trunkation.
    // But above we used floor().

    // We need standard modulo behavior for negative numbers
    let mut block_x = (world_x / BLOCK_SIZE).floor() as i32;
    let mut block_y = (world_y / BLOCK_SIZE).floor() as i32;

    let chunk_size_x = CHUNK_SIZE_X_BLOCKS as i32;
    let chunk_size_y = CHUNK_SIZE_Y_BLOCKS as i32;

    let rel_x = ((block_x % chunk_size_x) + chunk_size_x) % chunk_size_x;
    let rel_y = ((block_y % chunk_size_y) + chunk_size_y) % chunk_size_y;

    (rel_x as usize, rel_y as usize)
}

pub fn chunk_coords_to_world_origin(chunk_x: i32, chunk_y: i32) -> (f32, f32) {
    let world_x = chunk_x as f32 * CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE;
    let world_y = chunk_y as f32 * CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE;
    (world_x, world_y)
}
