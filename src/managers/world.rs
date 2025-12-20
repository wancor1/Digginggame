use crate::components::{Block, Chunk};
use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, HARDNESS_INCREASE_PER_BLOCK,
    HARDNESS_MIN, NOISE_HARDNESS_RANGE, NOISE_SCALE_HARDNESS, NOISE_SCALE_ORE, ORE_THRESHOLD,
    SCREEN_HEIGHT, SCREEN_WIDTH, SPRITE_BLOCK_COAL, SPRITE_BLOCK_DIRT, SPRITE_BLOCK_GRASS,
    SPRITE_BLOCK_STONE, SURFACE_Y_LEVEL,
};
use crate::utils::{world_to_chunk_coords, world_to_relative_in_chunk_coords};

use ::rand::Rng;
use macroquad::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use std::collections::{HashMap, HashSet}; // Needed for apply_modifications

pub struct WorldManager {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub generated_chunk_coords: HashSet<(i32, i32)>,
    pub world_seed_main: u32,
    pub world_seed_ore: u32,
    noise_main: Perlin,
    noise_ore: Perlin,
}

impl WorldManager {
    fn chunk_coords_to_world_origin(chunk_x: i32, chunk_y: i32) -> (f32, f32) {
        let world_x = chunk_x as f32 * CHUNK_SIZE_X_BLOCKS as f32 * BLOCK_SIZE;
        let world_y = chunk_y as f32 * CHUNK_SIZE_Y_BLOCKS as f32 * BLOCK_SIZE;
        (world_x, world_y)
    }

    pub fn new() -> Self {
        let mut rng = ::rand::thread_rng();
        let seed_main = rng.random::<u32>();
        let seed_ore = rng.random::<u32>();

        let noise_main = Perlin::new(seed_main).set_seed(seed_main);
        let noise_ore = Perlin::new(seed_ore).set_seed(seed_ore);

        Self {
            chunks: HashMap::new(),
            generated_chunk_coords: HashSet::new(),
            world_seed_main: seed_main,
            world_seed_ore: seed_ore,
            noise_main,
            noise_ore,
        }
    }

    pub fn seed(&mut self, main: u32, ore: u32) {
        self.world_seed_main = main;
        self.world_seed_ore = ore;
        self.noise_main = Perlin::new(main).set_seed(main);
        self.noise_ore = Perlin::new(ore).set_seed(ore);
        self.chunks.clear();
        self.generated_chunk_coords.clear();
    }

    pub fn reset(&mut self) {
        self.chunks.clear();
        self.generated_chunk_coords.clear();
    }

    pub fn ensure_chunk_exists_and_generated(&mut self, chunk_x: i32, chunk_y: i32) {
        let noise_main_ref = &self.noise_main;
        let noise_ore_ref = &self.noise_ore;

        let entry = self
            .chunks
            .entry((chunk_x, chunk_y))
            .or_insert_with(|| Chunk::new(chunk_x, chunk_y));

        if !entry.is_generated {
            let (origin_x, origin_y) =
                Self::chunk_coords_to_world_origin(entry.chunk_x, entry.chunk_y);

            for bx in 0..CHUNK_SIZE_X_BLOCKS {
                let mut row = Vec::new();
                for by in 0..CHUNK_SIZE_Y_BLOCKS {
                    let wx = origin_x + bx as f32 * BLOCK_SIZE;
                    let wy = origin_y + by as f32 * BLOCK_SIZE;
                    let mut max_hp = 0;
                    let mut sprite_rect = None;

                    let y_block = (wy / BLOCK_SIZE).floor() as i32;

                    if y_block < SURFACE_Y_LEVEL {
                        // Air blocks, max_hp will be 0
                    } else {
                        if y_block == SURFACE_Y_LEVEL {
                            sprite_rect = Some(SPRITE_BLOCK_GRASS);
                            max_hp = HARDNESS_MIN;
                        } else {
                            let depth = (y_block - (SURFACE_Y_LEVEL + 1)) as f64;
                            let base_hardness =
                                HARDNESS_MIN as f64 + depth * HARDNESS_INCREASE_PER_BLOCK;
                            let noise_val = noise_main_ref.get([
                                wx as f64 * NOISE_SCALE_HARDNESS,
                                wy as f64 * NOISE_SCALE_HARDNESS,
                                0.0,
                            ]);

                            let noise_contribution = noise_val
                                * NOISE_HARDNESS_RANGE
                                * (if noise_val >= 0.0 { 1.0 } else { 0.25 });
                            max_hp = (base_hardness + noise_contribution)
                                .floor()
                                .max(HARDNESS_MIN as f64)
                                as i32;

                            if max_hp <= 10 {
                                sprite_rect = Some(SPRITE_BLOCK_DIRT);
                            } else {
                                let ore_val = noise_ore_ref.get([
                                    wx as f64 * NOISE_SCALE_ORE,
                                    wy as f64 * NOISE_SCALE_ORE,
                                    256.0,
                                ]);
                                sprite_rect = Some(if ore_val >= ORE_THRESHOLD {
                                    SPRITE_BLOCK_COAL
                                } else {
                                    SPRITE_BLOCK_STONE
                                });
                            }
                        }
                    }
                    row.push(Block::new(wx, wy, max_hp, sprite_rect));
                }
                entry.blocks.push(row);
            }
            entry.is_generated = true;
            self.generated_chunk_coords.insert((chunk_x, chunk_y));
        }
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_y: i32) -> Option<&mut Chunk> {
        self.chunks.get_mut(&(chunk_x, chunk_y))
    }

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

    pub fn get_block_at_world_coords(
        &mut self,
        world_x: f32,
        world_y: f32,
    ) -> Option<(i32, i32, usize, usize, &mut Block)> {
        let (cx, cy) = world_to_chunk_coords(world_x, world_y);
        let (rel_x, rel_y) = world_to_relative_in_chunk_coords(world_x, world_y);

        // Ensure the chunk exists and is generated, then get mutable access
        // We ensure it here, but can't borrow mutably again inside.
        // So we just rely on `ensure_chunk_exists_and_generated` being called beforehand in main.rs loop
        // to populate the chunk.
        // self.ensure_chunk_exists_and_generated(cx, cy);

        let chunk = self.chunks.get_mut(&(cx, cy))?;
        if chunk.is_generated {
            Some((cx, cy, rel_x, rel_y, chunk.get_block(rel_x, rel_y)?))
        } else {
            None
        }
    }

    pub fn get_active_blocks_in_view<'a>(
        &'a mut self,
        camera_x: f32,
        camera_y: f32,
    ) -> Vec<&'a Block> {
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

        // Phase 1: Ensure all visible chunks are generated (mutation phase)
        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                self.ensure_chunk_exists_and_generated(cx, cy);
            }
        }

        // Phase 2: Collect immutable references to blocks (immutable access phase)
        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                if let Some(chunk) = self.chunks.get(&(cx, cy)) {
                    // At this point, `self` is not mutably borrowed by any generation method
                    // (because ensure_chunk_exists_and_generated has completed and dropped its mutable borrow)
                    if chunk.is_generated {
                        for row in &chunk.blocks {
                            for block in row {
                                if !block.is_broken
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
        }
        blocks
    }

    pub fn regenerate_world_from_save(
        &mut self,
        _gen_coords: HashSet<(i32, i32)>,
        _mod_data: Vec<crate::components::Chunk>,
    ) {
        // Since Chunk in save data format is complex to map directly back to struct with current logic,
        // we assumed PersistentManager passes parsed data.

        // Actually, PersistenceManager logic in Python:
        // 1. Set seeds.
        // 2. Add generated coords to set.
        // 3. Ensure chunks exist.
        // 4. Apply modifications.

        // We need a method to apply modifications to a chunk.
    }

    pub fn apply_modifications(&mut self, mod_chunks_data: Vec<serde_json::Value>) {
        // passing intermediate json value or struct
        // Let's assume passed struct is simple.
        for chunk_data in mod_chunks_data {
            let cx = chunk_data["cx"].as_i64().unwrap() as i32;
            let cy = chunk_data["cy"].as_i64().unwrap() as i32;
            let mods = chunk_data["modified_blocks"].as_array().unwrap();

            self.ensure_chunk_exists_and_generated(cx, cy); // Ensure chunk is generated before applying mods

            if let Some(chunk) = self.get_chunk_mut(cx, cy) {
                for mod_block in mods {
                    let bx = mod_block["x"].as_f64().unwrap() as f32;
                    let by = mod_block["y"].as_f64().unwrap() as f32;
                    let hp = mod_block["current_hp"].as_i64().unwrap() as i32;
                    let sprite_id = mod_block["sprite_id"].as_str().unwrap();

                    let (rel_x, rel_y) = world_to_relative_in_chunk_coords(bx, by);
                    if let Some(block) = chunk.get_block(rel_x, rel_y) {
                        block.current_hp = hp;
                        block.is_modified = true;
                        block.is_broken = block.current_hp <= 0;

                        block.sprite_rect = match sprite_id {
                            "dirt" => Some(SPRITE_BLOCK_DIRT),
                            "grass" => Some(SPRITE_BLOCK_GRASS),
                            "stone" => Some(SPRITE_BLOCK_STONE),
                            "coal" => Some(SPRITE_BLOCK_COAL),
                            _ => None, // "air" or "unknown"
                        };
                    }
                }
                chunk.is_modified_in_session = true;
            }
        }
    }
}
