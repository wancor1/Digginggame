use crate::components::{Block, BlockType};
use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, HARDNESS_DEPTH_MULTIPLIER,
    NOISE_SCALE_ORE, ORE_THRESHOLD, PLAYER_INITIAL_X, PLAYER_INITIAL_Y, SURFACE_Y_LEVEL,
};
use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};
use num_traits::ToPrimitive;

#[must_use]
pub fn generate_chunk_blocks(chunk_x: i32, chunk_y: i32, noise_ore: &Perlin) -> Vec<Vec<Block>> {
    let (origin_x, origin_y) = chunk_coords_to_world_origin(chunk_x, chunk_y);
    let mut blocks = Vec::new();

    for bx in 0..CHUNK_SIZE_X_BLOCKS {
        let mut row = Vec::new();
        for by in 0..CHUNK_SIZE_Y_BLOCKS {
            let wx = origin_x + bx.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
            let wy = origin_y + by.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
            let y_block = (wy / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let x_block = (wx / BLOCK_SIZE).floor().to_i32().unwrap_or(0);

            // Initial Spawn Point - Warp Gate
            let player_start_x_block = (PLAYER_INITIAL_X / BLOCK_SIZE)
                .floor()
                .to_i32()
                .unwrap_or(0);
            let player_start_y_block = (PLAYER_INITIAL_Y / BLOCK_SIZE)
                .floor()
                .to_i32()
                .unwrap_or(0);

            let (max_hp, sprite_rect, block_type) =
                if x_block == player_start_x_block && y_block == player_start_y_block {
                    let bt = BlockType::WarpGate;
                    (50, bt.get_sprite(), bt)
                } else if x_block == (PLAYER_INITIAL_X / BLOCK_SIZE).to_i32().unwrap_or(0)
                    && y_block
                        == ((PLAYER_INITIAL_Y + BLOCK_SIZE) / BLOCK_SIZE)
                            .to_i32()
                            .unwrap_or(0)
                {
                    let bt = BlockType::Indestructible;
                    (bt.get_base_hardness(), bt.get_sprite(), bt)
                } else if y_block < SURFACE_Y_LEVEL {
                    (0, None, BlockType::Air)
                } else if y_block == SURFACE_Y_LEVEL {
                    let bt = BlockType::Grass;
                    (bt.get_base_hardness(), bt.get_sprite(), bt)
                } else {
                    let mut b_type = BlockType::Dirt;

                    if y_block > SURFACE_Y_LEVEL + 5 {
                        let ore_val = noise_ore.get([
                            wx.to_f64().unwrap_or(0.0) * NOISE_SCALE_ORE,
                            wy.to_f64().unwrap_or(0.0) * NOISE_SCALE_ORE,
                            256.0,
                        ]);
                        if ore_val >= ORE_THRESHOLD {
                            b_type = BlockType::Coal;
                        } else if y_block > SURFACE_Y_LEVEL + 10 {
                            let water_val = noise_ore.get([
                                wx.to_f64().unwrap_or(0.0) * 0.1,
                                wy.to_f64().unwrap_or(0.0) * 0.1,
                                512.0,
                            ]);
                            if water_val > 0.65 {
                                b_type = BlockType::Water;
                            } else {
                                b_type = BlockType::Stone;
                            }
                        }
                    }

                    if y_block > 1000 {
                        b_type = BlockType::Indestructible;
                    }

                    let base_hardness = b_type.get_base_hardness();
                    let s_rect = b_type.get_sprite();

                    let hp = if base_hardness == -1 {
                        -1
                    } else {
                        let depth = (y_block - SURFACE_Y_LEVEL).to_f64().unwrap_or(0.0);
                        let multiplier = 1.0 + depth * HARDNESS_DEPTH_MULTIPLIER;
                        (base_hardness.to_f64().unwrap_or(0.0) * multiplier)
                            .floor()
                            .to_i32()
                            .unwrap_or(0)
                    };

                    (hp, s_rect, b_type)
                };

            if block_type == BlockType::WarpGate {
                let mut b = Block::new(wx, wy, max_hp, sprite_rect, block_type);
                b.name = Some("Home".to_string());
                b.back_type = BlockType::Air; // Don't show a wall behind the home gate
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
    let world_x =
        chunk_x.to_f32().unwrap_or(0.0) * CHUNK_SIZE_X_BLOCKS.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
    let world_y =
        chunk_y.to_f32().unwrap_or(0.0) * CHUNK_SIZE_Y_BLOCKS.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
    (world_x, world_y)
}
