use crate::components::{Block, BlockType};
use crate::constants::*;
use crate::render::sprites::*;
use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};

pub fn generate_chunk_blocks(chunk_x: i32, chunk_y: i32, noise_ore: &Perlin) -> Vec<Vec<Block>> {
    let (origin_x, origin_y) = chunk_coords_to_world_origin(chunk_x, chunk_y);
    let mut blocks = Vec::new();

    for bx in 0..CHUNK_SIZE_X_BLOCKS {
        let mut row = Vec::new();
        for by in 0..CHUNK_SIZE_Y_BLOCKS {
            let wx = origin_x + bx as f32 * BLOCK_SIZE;
            let wy = origin_y + by as f32 * BLOCK_SIZE;
            let y_block = (wy / BLOCK_SIZE).floor() as i32;
            let x_block = (wx / BLOCK_SIZE).floor() as i32;

            // Initial Spawn Point - Warp Gate
            let player_start_x_block = (PLAYER_INITIAL_X / BLOCK_SIZE).floor() as i32;
            let player_start_y_block = (PLAYER_INITIAL_Y / BLOCK_SIZE).floor() as i32;

            let (max_hp, sprite_rect, block_type) =
                if x_block == player_start_x_block && y_block == player_start_y_block {
                    (50, Some(SPRITE_BLOCK_WARPGATE), BlockType::WarpGate)
                } else if x_block == (PLAYER_INITIAL_X / BLOCK_SIZE) as i32
                    && y_block == ((PLAYER_INITIAL_Y + BLOCK_SIZE) / BLOCK_SIZE) as i32
                {
                    (
                        HARDNESS_INDESTRUCTIBLE,
                        Some(SPRITE_BLOCK_INDESTRUCTIBLE),
                        BlockType::Indestructible,
                    )
                } else if y_block < SURFACE_Y_LEVEL {
                    (0, None, BlockType::Air)
                } else if y_block == SURFACE_Y_LEVEL {
                    (HARDNESS_GRASS, Some(SPRITE_BLOCK_GRASS), BlockType::Grass)
                } else {
                    let mut b_type = BlockType::Dirt;
                    let mut s_rect = Some(SPRITE_BLOCK_DIRT);
                    let mut base_hardness = HARDNESS_DIRT;

                    if y_block > SURFACE_Y_LEVEL + 5 {
                        let ore_val = noise_ore.get([
                            wx as f64 * NOISE_SCALE_ORE,
                            wy as f64 * NOISE_SCALE_ORE,
                            256.0,
                        ]);
                        if ore_val >= ORE_THRESHOLD {
                            s_rect = Some(SPRITE_BLOCK_COAL);
                            base_hardness = HARDNESS_COAL;
                            b_type = BlockType::Coal;
                        } else if y_block > SURFACE_Y_LEVEL + 10 {
                            s_rect = Some(SPRITE_BLOCK_STONE);
                            base_hardness = HARDNESS_STONE;
                            b_type = BlockType::Stone;
                        }
                    }

                    if y_block > 1000 {
                        s_rect = Some(SPRITE_BLOCK_INDESTRUCTIBLE);
                        base_hardness = HARDNESS_INDESTRUCTIBLE;
                        b_type = BlockType::Indestructible;
                    }

                    let hp = if base_hardness == HARDNESS_INDESTRUCTIBLE {
                        -1
                    } else {
                        let depth = (y_block - SURFACE_Y_LEVEL) as f64;
                        let multiplier = 1.0 + depth * HARDNESS_DEPTH_MULTIPLIER;
                        (base_hardness as f64 * multiplier).floor() as i32
                    };

                    (hp, s_rect, b_type)
                };

            if block_type == BlockType::WarpGate {
                let mut b = Block::new(wx, wy, max_hp, sprite_rect, block_type);
                b.name = Some("Home".to_string());
                row.push(b);
            } else {
                row.push(Block::new(wx, wy, max_hp, sprite_rect, block_type));
            }
        }
        blocks.push(row);
    }
    blocks
}

fn chunk_coords_to_world_origin(chunk_x: i32, chunk_y: i32) -> (f32, f32) {
    let world_x = chunk_x as f32 * CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE;
    let world_y = chunk_y as f32 * CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE;
    (world_x, world_y)
}
