use crate::Game;
use crate::components::{BlockPos, ChunkRelPos};
use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use crate::render::ui::common::MenuRenderContext;
use macroquad::prelude::*;
use num_traits::ToPrimitive;

pub fn draw_terrain(
    game: &mut Game,
    ctx: &MenuRenderContext,
    view_x: f32,
    view_y: f32,
    blocks_per_pixel: f32,
) {
    // 1. Pre-load/ensure visited chunks in view
    let half_w_blocks = (SCREEN_WIDTH / 2.0) * blocks_per_pixel;
    let half_h_blocks = (SCREEN_HEIGHT / 2.0) * blocks_per_pixel;

    let start_x_blocks = (view_x / BLOCK_SIZE - half_w_blocks)
        .floor()
        .to_i32()
        .unwrap_or(0);
    let end_x_blocks = (view_x / BLOCK_SIZE + half_w_blocks)
        .ceil()
        .to_i32()
        .unwrap_or(0);
    let start_y_blocks = (view_y / BLOCK_SIZE - half_h_blocks)
        .floor()
        .to_i32()
        .unwrap_or(0);
    let end_y_blocks = (view_y / BLOCK_SIZE + half_h_blocks)
        .ceil()
        .to_i32()
        .unwrap_or(0);

    let start_cx = (start_x_blocks.to_f32().unwrap_or(0.0)
        / CHUNK_SIZE_X_BLOCKS.to_f32().unwrap_or(0.0))
    .floor()
    .to_i32()
    .unwrap_or(0);
    let end_cx = (end_x_blocks.to_f32().unwrap_or(0.0)
        / CHUNK_SIZE_X_BLOCKS.to_f32().unwrap_or(0.0))
    .ceil()
    .to_i32()
    .unwrap_or(0);
    let start_cy = (start_y_blocks.to_f32().unwrap_or(0.0)
        / CHUNK_SIZE_Y_BLOCKS.to_f32().unwrap_or(0.0))
    .floor()
    .to_i32()
    .unwrap_or(0);
    let end_cy = (end_y_blocks.to_f32().unwrap_or(0.0)
        / CHUNK_SIZE_Y_BLOCKS.to_f32().unwrap_or(0.0))
    .ceil()
    .to_i32()
    .unwrap_or(0);

    for cx in start_cx..=end_cx {
        for cy in start_cy..=end_cy {
            if game
                .world_manager
                .visited_chunks
                .contains(&BlockPos::new(cx, cy))
            {
                game.world_manager.ensure_chunk_exists_and_generated(cx, cy);
            }
        }
    }

    // 2. Render pixels
    let mut last_cx = i32::MAX;
    let mut last_cy = i32::MAX;
    let mut last_visited = false;
    let mut last_chunk: Option<&crate::components::Chunk> = None;

    for py_idx in 0..SCREEN_HEIGHT.to_i32().unwrap_or(0) {
        let py = py_idx.to_f32().unwrap_or(0.0);
        for px_idx in 0..SCREEN_WIDTH.to_i32().unwrap_or(0) {
            let px = px_idx.to_f32().unwrap_or(0.0);

            // Calculate world coordinates - Match marker centering
            let off_x = px - SCREEN_WIDTH / 2.0;
            let off_y = py - SCREEN_HEIGHT / 2.0;

            // world_x should be view_x at off_x = 0
            let world_x = view_x + off_x * blocks_per_pixel * BLOCK_SIZE;
            let world_y = view_y + off_y * blocks_per_pixel * BLOCK_SIZE;

            let BlockPos { x: cx, y: cy } = crate::utils::world_to_chunk_coords(world_x, world_y);

            if cx != last_cx || cy != last_cy {
                last_cx = cx;
                last_cy = cy;
                last_visited = game
                    .world_manager
                    .visited_chunks
                    .contains(&BlockPos::new(cx, cy));
                last_chunk = if last_visited {
                    game.world_manager.get_chunk(cx, cy)
                } else {
                    None
                };
            }

            // Only draw if visited
            let color = if last_visited {
                last_chunk.map_or_else(
                    || Color::from_rgba(10, 10, 10, 255),
                    |chunk| {
                        let ChunkRelPos { x: rel_x, y: rel_y } =
                            crate::utils::world_to_relative_in_chunk_coords(world_x, world_y);
                        if rel_x < CHUNK_SIZE_X_BLOCKS && rel_y < CHUNK_SIZE_Y_BLOCKS {
                            let block = &chunk.blocks[rel_x][rel_y];
                            if block.is_broken {
                                Color::from_rgba(20, 20, 40, 255)
                            } else {
                                block.block_type.get_map_color()
                            }
                        } else {
                            BLACK
                        }
                    },
                )
            } else {
                BLACK // Fog of war
            };

            draw_rectangle(
                ctx.offset_x + px * ctx.scale,
                ctx.offset_y + py * ctx.scale,
                ctx.scale,
                ctx.scale,
                color,
            );
        }
    }
}
