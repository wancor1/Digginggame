use crate::components::{BlockPos, ChunkRelPos};
use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, MACROGRID_SIZE_CHUNKS, SCREEN_HEIGHT,
    SCREEN_WIDTH, SURFACE_TEMPERATURE, SURFACE_Y_LEVEL, TEMPERATURE_GRADIENT,
};
use macroquad::prelude::*;
use num_traits::ToPrimitive;

pub mod icon;

#[must_use]
pub fn get_temperature(y: f32) -> f32 {
    let depth = (y / BLOCK_SIZE).floor() - SURFACE_Y_LEVEL.to_f32().unwrap_or(0.0);
    depth
        .max(0.0)
        .mul_add(TEMPERATURE_GRADIENT, SURFACE_TEMPERATURE)
}

#[must_use]
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

#[must_use]
pub fn get_game_mouse_position() -> (f32, f32) {
    let (render_width, _, offset_x, offset_y) = get_render_dimensions();
    let (mx, my) = mouse_position();
    let rw = render_width.floor();
    let ox = offset_x.floor();
    let oy = offset_y.floor();
    let scale = rw / SCREEN_WIDTH;
    ((mx - ox) / scale, (my - oy) / scale)
}

#[must_use]
pub fn get_game_mouse_position_if_inside_render() -> Option<(f32, f32)> {
    let (render_width, render_height, offset_x, offset_y) = get_render_dimensions();
    let (mx, my) = mouse_position();
    let rw = render_width.floor();
    let rh = render_height.floor();
    let ox = offset_x.floor();
    let oy = offset_y.floor();

    let inside_x = mx >= ox && mx < ox + rw;
    let inside_y = my >= oy && my < oy + rh;
    if !(inside_x && inside_y) {
        return None;
    }

    let scale = rw / SCREEN_WIDTH;
    Some(((mx - ox) / scale, (my - oy) / scale))
}

#[must_use]
pub fn world_to_chunk_coords(world_x: f32, world_y: f32) -> BlockPos {
    let chunk_x = (world_x / (CHUNK_SIZE_X_BLOCKS.to_f32().unwrap_or(0.0) * BLOCK_SIZE))
        .floor()
        .to_i32()
        .unwrap_or(0);
    let chunk_y = (world_y / (CHUNK_SIZE_Y_BLOCKS.to_f32().unwrap_or(0.0) * BLOCK_SIZE))
        .floor()
        .to_i32()
        .unwrap_or(0);
    BlockPos::new(chunk_x, chunk_y)
}

#[must_use]
pub fn chunk_to_macrogrid_coords(chunk_x: i32, chunk_y: i32) -> (BlockPos, BlockPos) {
    let mg_x = (chunk_x.to_f32().unwrap_or(0.0) / MACROGRID_SIZE_CHUNKS.to_f32().unwrap_or(0.0))
        .floor()
        .to_i32()
        .unwrap_or(0);
    let mg_y = (chunk_y.to_f32().unwrap_or(0.0) / MACROGRID_SIZE_CHUNKS.to_f32().unwrap_or(0.0))
        .floor()
        .to_i32()
        .unwrap_or(0);
    let rel_cx = ((chunk_x % MACROGRID_SIZE_CHUNKS.to_i32().unwrap_or(0))
        + MACROGRID_SIZE_CHUNKS.to_i32().unwrap_or(0))
        % MACROGRID_SIZE_CHUNKS.to_i32().unwrap_or(0);
    let rel_cy = ((chunk_y % MACROGRID_SIZE_CHUNKS.to_i32().unwrap_or(0))
        + MACROGRID_SIZE_CHUNKS.to_i32().unwrap_or(0))
        % MACROGRID_SIZE_CHUNKS.to_i32().unwrap_or(0);
    (BlockPos::new(mg_x, mg_y), BlockPos::new(rel_cx, rel_cy))
}

#[must_use]
pub fn world_to_relative_in_chunk_coords(world_x: f32, world_y: f32) -> ChunkRelPos {
    // We need standard modulo behavior for negative numbers
    let block_x = (world_x / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
    let block_y = (world_y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);

    let chunk_size_x = CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0);
    let chunk_size_y = CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0);

    let rel_x = ((block_x % chunk_size_x) + chunk_size_x) % chunk_size_x;
    let rel_y = ((block_y % chunk_size_y) + chunk_size_y) % chunk_size_y;

    ChunkRelPos::new(rel_x.to_usize().unwrap_or(0), rel_y.to_usize().unwrap_or(0))
}

#[must_use]
pub fn get_item_weight(item_type: &str) -> i32 {
    crate::managers::block::BLOCK_MANAGER
        .get_by_item_type(item_type)
        .map_or(0, |bt| {
            crate::managers::block::BLOCK_MANAGER.get_weight(&bt)
        })
}

#[must_use]
pub fn get_item_sprite(item_type: &str) -> Rect {
    crate::managers::block::BLOCK_MANAGER
        .get_by_item_type(item_type)
        .and_then(|bt| crate::managers::block::BLOCK_MANAGER.get_sprite(&bt))
        .unwrap_or(Rect::new(0.0, 0.0, 0.0, 0.0))
}
