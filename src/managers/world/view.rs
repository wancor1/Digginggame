use super::WorldManager;
use crate::components::{Block, BlockType};
use crate::constants::*;
use crate::utils::chunk_to_macrogrid_coords;
use macroquad::prelude::*;

impl WorldManager {
    pub fn generate_visible_chunks(&mut self, camera_x: f32, camera_y: f32) {
        let start_cx = (camera_x / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let start_cy = (camera_y / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let end_cx =
            ((camera_x + SCREEN_WIDTH) / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let end_cy =
            ((camera_y + SCREEN_HEIGHT) / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;

        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                self.ensure_chunk_exists_and_generated(cx, cy);
            }
        }
    }

    pub fn get_active_blocks_in_view(&mut self, camera_x: f32, camera_y: f32) -> Vec<&Block> {
        let mut blocks = Vec::new();
        let view_rect = Rect::new(
            camera_x - BLOCK_SIZE,
            camera_y - BLOCK_SIZE,
            SCREEN_WIDTH + BLOCK_SIZE * 2.0,
            SCREEN_HEIGHT + BLOCK_SIZE * 2.0,
        );

        let start_cx = (view_rect.x / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let start_cy = (view_rect.y / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let end_cx = ((view_rect.x + view_rect.w) / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE))
            .floor() as i32;
        let end_cy = ((view_rect.y + view_rect.h) / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE))
            .floor() as i32;

        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                self.ensure_chunk_exists_and_generated(cx, cy);
            }
        }

        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(cx, cy);
                if let Some(macrogrid) = self.macrogrids.get(&(mg_coords))
                    && let Some(chunk) = macrogrid.chunks.get(&rel_coords)
                    && chunk.is_generated
                {
                    for row in &chunk.blocks {
                        for block in row {
                            // Include block if:
                            // 1. It's not broken
                            // 2. It has a background to show (back_type != Air)
                            // 3. It's underground (so we can draw black background)
                            let is_underground = block.y >= SURFACE_Y_LEVEL as f32 * BLOCK_SIZE;
                            let should_render = (!block.is_broken)
                                || (block.back_type != BlockType::Air)
                                || is_underground;

                            if should_render
                                && block.x + BLOCK_SIZE > view_rect.x
                                && block.x < view_rect.x + view_rect.w
                                && block.y + BLOCK_SIZE > view_rect.y
                                && block.y < view_rect.y + view_rect.h
                            {
                                blocks.push(block);
                            }
                        }
                    }
                }
            }
        }
        blocks
    }

    pub fn get_active_blocks_in_view_immutable(&self, camera_x: f32, camera_y: f32) -> Vec<&Block> {
        let mut blocks = Vec::new();
        let view_rect = Rect::new(
            camera_x - BLOCK_SIZE,
            camera_y - BLOCK_SIZE,
            SCREEN_WIDTH + BLOCK_SIZE * 2.0,
            SCREEN_HEIGHT + BLOCK_SIZE * 2.0,
        );

        let start_cx = (view_rect.x / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let start_cy = (view_rect.y / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE)).floor() as i32;
        let end_cx = ((view_rect.x + view_rect.w) / (CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE))
            .floor() as i32;
        let end_cy = ((view_rect.y + view_rect.h) / (CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE))
            .floor() as i32;

        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(cx, cy);
                if let Some(macrogrid) = self.macrogrids.get(&(mg_coords))
                    && let Some(chunk) = macrogrid.chunks.get(&rel_coords)
                    && chunk.is_generated
                {
                    for row in &chunk.blocks {
                        for block in row {
                            // Include block if:
                            // 1. It's not broken
                            // 2. It has a background to show (back_type != Air)
                            // 3. It's underground (so we can draw black background)
                            let is_underground = block.y >= SURFACE_Y_LEVEL as f32 * BLOCK_SIZE;
                            let should_render = (!block.is_broken)
                                || (block.back_type != BlockType::Air)
                                || is_underground;

                            if should_render
                                && block.x + BLOCK_SIZE > view_rect.x
                                && block.x < view_rect.x + view_rect.w
                                && block.y + BLOCK_SIZE > view_rect.y
                                && block.y < view_rect.y + view_rect.h
                            {
                                blocks.push(block);
                            }
                        }
                    }
                }
            }
        }
        blocks
    }
}
