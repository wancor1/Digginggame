use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, MACROGRID_SIZE_CHUNKS, SCREEN_HEIGHT,
    SCREEN_WIDTH, SURFACE_TEMPERATURE, SURFACE_Y_LEVEL, TEMPERATURE_GRADIENT,
};
use macroquad::prelude::*;

pub fn get_temperature(y: f32) -> f32 {
    let depth = (y / BLOCK_SIZE).floor() - SURFACE_Y_LEVEL as f32;
    SURFACE_TEMPERATURE + (depth.max(0.0) * TEMPERATURE_GRADIENT)
}

pub fn get_render_dimensions() -> (f32, f32, f32, f32) {
    let target_aspect = SCREEN_WIDTH / SCREEN_HEIGHT;
    let screen_aspect = screen_width() / screen_height();

    let (render_width, render_height, offset_x, offset_y);
    if screen_aspect > target_aspect {
        render_height = screen_height();
        render_width = SCREEN_WIDTH * (render_height / SCREEN_HEIGHT);
        offset_x = (screen_width() - render_width) / 2.0;
        offset_y = 0.0;
    } else {
        render_width = screen_width();
        render_height = SCREEN_HEIGHT * (render_width / SCREEN_WIDTH);
        offset_x = 0.0;
        offset_y = (screen_height() - render_height) / 2.0;
    }
    (render_width, render_height, offset_x, offset_y)
}

pub fn get_game_mouse_position() -> (f32, f32) {
    let (render_width, _, offset_x, offset_y) = get_render_dimensions();
    let (mx, my) = mouse_position();
    let rw = render_width.floor();
    let ox = offset_x.floor();
    let oy = offset_y.floor();
    let scale = rw / SCREEN_WIDTH;
    ((mx - ox) / scale, (my - oy) / scale)
}

pub fn world_to_chunk_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let chunk_x = (world_x / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
    let chunk_y = (world_y / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
    (chunk_x, chunk_y)
}

pub fn chunk_to_macrogrid_coords(chunk_x: i32, chunk_y: i32) -> ((i32, i32), (i32, i32)) {
    let mg_x = (chunk_x as f32 / MACROGRID_SIZE_CHUNKS as f32).floor() as i32;
    let mg_y = (chunk_y as f32 / MACROGRID_SIZE_CHUNKS as f32).floor() as i32;
    let rel_cx = ((chunk_x % MACROGRID_SIZE_CHUNKS as i32) + MACROGRID_SIZE_CHUNKS as i32)
        % MACROGRID_SIZE_CHUNKS as i32;
    let rel_cy = ((chunk_y % MACROGRID_SIZE_CHUNKS as i32) + MACROGRID_SIZE_CHUNKS as i32)
        % MACROGRID_SIZE_CHUNKS as i32;
    ((mg_x, mg_y), (rel_cx, rel_cy))
}

pub fn world_to_relative_in_chunk_coords(world_x: f32, world_y: f32) -> (usize, usize) {
    let (_cx, _cy) = world_to_chunk_coords(world_x, world_y);
    // Python math.floor handles negatives correctly, Rust cast to i32 does trunkation.
    // But above we used floor().

    // We need standard modulo behavior for negative numbers
    let block_x = (world_x / BLOCK_SIZE).floor() as i32;
    let block_y = (world_y / BLOCK_SIZE).floor() as i32;

    let chunk_size_x = CHUNK_SIZE_X_BLOCKS as i32;
    let chunk_size_y = CHUNK_SIZE_Y_BLOCKS as i32;

    let rel_x = ((block_x % chunk_size_x) + chunk_size_x) % chunk_size_x;
    let rel_y = ((block_y % chunk_size_y) + chunk_size_y) % chunk_size_y;

    (rel_x as usize, rel_y as usize)
}

pub fn get_item_weight(item_type: &str) -> i32 {
    crate::managers::block::BLOCK_MANAGER
        .get_by_item_type(item_type)
        .map(|bt| crate::managers::block::BLOCK_MANAGER.get_weight(&bt))
        .unwrap_or(0)
}

pub fn get_item_sprite(item_type: &str) -> Rect {
    crate::managers::block::BLOCK_MANAGER
        .get_by_item_type(item_type)
        .and_then(|bt| crate::managers::block::BLOCK_MANAGER.get_sprite(&bt))
        .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
}
