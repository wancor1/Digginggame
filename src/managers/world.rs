use crate::components::{Block, BlockSaveData, Chunk};
use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, SCREEN_HEIGHT, SCREEN_WIDTH,
    SPRITE_BLOCK_COAL, SPRITE_BLOCK_DIRT, SPRITE_BLOCK_GRASS, SPRITE_BLOCK_STONE,
};
use crate::utils::{world_to_chunk_coords, world_to_relative_in_chunk_coords};
use ::rand::thread_rng;
use ::rand::Rng;
use macroquad::prelude::*;
use noise::{Perlin, Seedable};
use std::collections::{HashMap, HashSet};

pub struct WorldManager {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub generated_chunk_coords: HashSet<(i32, i32)>,
    pub world_seed_main: u32,
    pub world_seed_ore: u32,
    noise_main: Perlin,
    noise_ore: Perlin,
}

impl WorldManager {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        let seed_main = rng.gen::<u32>();
        let seed_ore = rng.gen::<u32>();

        let noise_main = Perlin::new().set_seed(seed_main);
        let noise_ore = Perlin::new().set_seed(seed_ore);

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
        self.noise_main = Perlin::new().set_seed(main);
        self.noise_ore = Perlin::new().set_seed(ore);
        self.chunks.clear();
        self.generated_chunk_coords.clear();
    }

    pub fn ensure_chunk_generated_and_get(&mut self, chunk_x: i32, chunk_y: i32) -> &mut Chunk {
        self.chunks
            .entry((chunk_x, chunk_y))
            .or_insert_with(|| Chunk::new(chunk_x, chunk_y));

        // We need to access it again mutably, HashMap borrow rules apply.
        // Rust's Entry API is nice but the generation needs &self props (noise).

        // Let's get the chunk.
        let chunk = self.chunks.get_mut(&(chunk_x, chunk_y)).unwrap();
        if !chunk.is_generated {
            chunk.generate(&self.noise_main, &self.noise_ore);
            self.generated_chunk_coords.insert((chunk_x, chunk_y));
        }
        chunk
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
                self.ensure_chunk_generated_and_get(cx, cy);
            }
        }
    }

    // Split retrieval to avoid multiple mutable borrows if helpful, or just return Option ref.
    pub fn get_block_at_world_coords(&mut self, world_x: f32, world_y: f32) -> Option<&mut Block> {
        let (cx, cy) = world_to_chunk_coords(world_x, world_y);
        let chunk = self.ensure_chunk_generated_and_get(cx, cy);
        let (rel_x, rel_y) = world_to_relative_in_chunk_coords(world_x, world_y);
        chunk.get_block(rel_x, rel_y)
    }

    // Heavy cloning or reference gathering.
    // For rendering, we want list of blocks to draw.
    // Instead of cloning blocks, maybe we just iterate chunks in view.
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

        for cx in start_cx..=end_cx {
            for cy in start_cy..=end_cy {
                if let Some(chunk) = self.chunks.get(&(cx, cy)) {
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
        gen_coords: HashSet<(i32, i32)>,
        mod_data: Vec<crate::components::Chunk>,
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

            let chunk = self.ensure_chunk_generated_and_get(cx, cy);

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

                    // Map string ID back to Rect constants
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
