use crate::Game;
use crate::constants::*;
use crate::render::ui::common::MenuRenderContext;
use macroquad::prelude::*;

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

    let start_x_blocks = (view_x / BLOCK_SIZE - half_w_blocks).floor() as i32;
    let end_x_blocks = (view_x / BLOCK_SIZE + half_w_blocks).ceil() as i32;
    let start_y_blocks = (view_y / BLOCK_SIZE - half_h_blocks).floor() as i32;
    let end_y_blocks = (view_y / BLOCK_SIZE + half_h_blocks).ceil() as i32;

    let start_cx = (start_x_blocks as f32 / CHUNK_SIZE_X_BLOCKS as f32).floor() as i32;
    let end_cx = (end_x_blocks as f32 / CHUNK_SIZE_X_BLOCKS as f32).ceil() as i32;
    let start_cy = (start_y_blocks as f32 / CHUNK_SIZE_Y_BLOCKS as f32).floor() as i32;
    let end_cy = (end_y_blocks as f32 / CHUNK_SIZE_Y_BLOCKS as f32).ceil() as i32;

    for cx in start_cx..=end_cx {
        for cy in start_cy..=end_cy {
            if game.world_manager.visited_chunks.contains(&(cx, cy)) {
                game.world_manager.ensure_chunk_exists_and_generated(cx, cy);
            }
        }
    }

    // 2. Render pixels
    let mut last_cx = i32::MAX;
    let mut last_cy = i32::MAX;
    let mut last_visited = false;
    let mut last_chunk: Option<&crate::components::Chunk> = None;

    for py_idx in 0..SCREEN_HEIGHT as i32 {
        let py = py_idx as f32;
        for px_idx in 0..SCREEN_WIDTH as i32 {
            let px = px_idx as f32;

            // Calculate world coordinates - Match marker centering
            let off_x = px - SCREEN_WIDTH / 2.0;
            let off_y = py - SCREEN_HEIGHT / 2.0;

            // world_x should be view_x at off_x = 0
            let world_x = view_x + off_x * blocks_per_pixel * BLOCK_SIZE;
            let world_y = view_y + off_y * blocks_per_pixel * BLOCK_SIZE;

            let (cx, cy) = crate::utils::world_to_chunk_coords(world_x, world_y);

            if cx != last_cx || cy != last_cy {
                last_cx = cx;
                last_cy = cy;
                last_visited = game.world_manager.visited_chunks.contains(&(cx, cy));
                last_chunk = if last_visited {
                    game.world_manager.get_chunk(cx, cy)
                } else {
                    None
                };
            }

            // Only draw if visited
            let color = if last_visited {
                if let Some(chunk) = last_chunk {
                    let (rel_x, rel_y) =
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
                } else {
                    Color::from_rgba(10, 10, 10, 255)
                }
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
