use crate::constants::{BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS};
use macroquad::prelude::*;

pub fn world_to_chunk_coords(world_x: f32, world_y: f32) -> (i32, i32) {
    let chunk_x = (world_x / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
    let chunk_y = (world_y / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
    (chunk_x, chunk_y)
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
