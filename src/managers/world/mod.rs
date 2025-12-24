use crate::components::{Block, BlockType, Chunk, MacroGrid};
use crate::constants::*;
use crate::utils::{
    chunk_to_macrogrid_coords, world_to_chunk_coords, world_to_relative_in_chunk_coords,
};
use macroquad::prelude::*;
use noise::{Perlin, Seedable};
use std::collections::{HashMap, HashSet};

pub mod generation;
pub mod modifications;

pub struct WorldManager {
    pub macrogrids: HashMap<(i32, i32), MacroGrid>,
    pub generated_chunk_coords: HashSet<(i32, i32)>,
    pub visited_chunks: HashSet<(i32, i32)>,
    pub pending_modifications: HashMap<(i32, i32), crate::managers::persistence::ChunkSaveData>,
    pub world_seed_main: u32,
    pub world_seed_ore: u32,
    noise_main: Perlin,
    noise_ore: Perlin,
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldManager {
    pub fn new() -> Self {
        let mut rng = ::rand::rng();
        let seed_main = ::rand::Rng::random::<u32>(&mut rng);
        let seed_ore = ::rand::Rng::random::<u32>(&mut rng);

        let noise_main = Perlin::new(seed_main).set_seed(seed_main);
        let noise_ore = Perlin::new(seed_ore).set_seed(seed_ore);

        Self {
            macrogrids: HashMap::new(),
            generated_chunk_coords: HashSet::new(),
            visited_chunks: HashSet::new(),
            pending_modifications: HashMap::new(),
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
        self.macrogrids.clear();
        self.generated_chunk_coords.clear();
        self.visited_chunks.clear();
        self.pending_modifications.clear();
    }

    pub fn reset(&mut self) {
        self.macrogrids.clear();
        self.generated_chunk_coords.clear();
        self.visited_chunks.clear();
        self.pending_modifications.clear();
    }

    pub fn ensure_chunk_exists_and_generated(&mut self, chunk_x: i32, chunk_y: i32) {
        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        let macrogrid = self
            .macrogrids
            .entry(mg_coords)
            .or_insert_with(MacroGrid::new);

        let entry = macrogrid
            .chunks
            .entry(rel_coords)
            .or_insert_with(|| Chunk::new(chunk_x, chunk_y));

        if !entry.is_generated {
            entry.blocks = generation::generate_chunk_blocks(chunk_x, chunk_y, &self.noise_ore);
            entry.is_generated = true;
            self.generated_chunk_coords.insert((chunk_x, chunk_y));

            // Apply pending modifications if any (Lazy Loading)
            if let Some(mod_data) = self.pending_modifications.remove(&(chunk_x, chunk_y)) {
                modifications::apply_chunk_save_data(entry, &mod_data);
            }
        }

        self.visited_chunks.insert((chunk_x, chunk_y));
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_y: i32) -> Option<&mut Chunk> {
        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        self.macrogrids
            .get_mut(&mg_coords)?
            .chunks
            .get_mut(&rel_coords)
    }

    pub fn get_chunk(&self, chunk_x: i32, chunk_y: i32) -> Option<&Chunk> {
        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        self.macrogrids.get(&mg_coords)?.chunks.get(&rel_coords)
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

        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(cx, cy);
        let chunk = self
            .macrogrids
            .get_mut(&mg_coords)?
            .chunks
            .get_mut(&rel_coords)?;

        if chunk.is_generated {
            Some((cx, cy, rel_x, rel_y, chunk.get_block(rel_x, rel_y)?))
        } else {
            None
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
                            // Include block if it's not broken OR if it has a background to show
                            let should_render = (!block.is_broken) || (block.back_type != BlockType::Air);
                            
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

    pub fn apply_modifications(
        &mut self,
        mod_macrogrids_data: Vec<crate::managers::persistence::MacroGridSaveData>,
    ) {
        // Clear old pending modifications to avoid stale data (though typically this is called on load)
        self.pending_modifications.clear();

        for mg_data in mod_macrogrids_data {
            for chunk_data in mg_data.chunks {
                let cx = chunk_data.cx;
                let cy = chunk_data.cy;

                self.visited_chunks.insert((cx, cy));

                // If the chunk is ALREADY generated in memory, apply immediately.
                // Otherwise, store it for later lazy loading.
                let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(cx, cy);
                if let Some(macrogrid) = self.macrogrids.get_mut(&mg_coords)
                    && let Some(chunk) = macrogrid.chunks.get_mut(&rel_coords)
                    && chunk.is_generated
                {
                    modifications::apply_chunk_save_data(chunk, &chunk_data);
                } else {
                    self.pending_modifications.insert((cx, cy), chunk_data);
                }
            }
        }
    }

    pub fn update(&mut self) {
        let current_time = get_time();
        for macrogrid in self.macrogrids.values_mut() {
            for chunk in macrogrid.chunks.values_mut() {
                if !chunk.is_generated {
                    continue;
                }
                for row in &mut chunk.blocks {
                    for block in row {
                        if !block.is_broken
                            && block.current_hp < block.max_hp
                            && let Some(last_time) = block.last_damage_time
                            && current_time - last_time >= 60.0
                        {
                            block.current_hp = block.max_hp;
                            block.last_damage_time = None;
                        }
                    }
                }
            }
        }
    }
}
