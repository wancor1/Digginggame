use super::WorldManager;
use super::generation;
use super::modifications;
use crate::components::{Block, Chunk};
use crate::constants::*;
use crate::utils::{
    chunk_to_macrogrid_coords, world_to_chunk_coords, world_to_relative_in_chunk_coords,
};

impl WorldManager {
    pub fn ensure_chunk_exists_and_generated(&mut self, chunk_x: i32, chunk_y: i32) {
        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        let macrogrid = self.macrogrids.entry(mg_coords).or_default();

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

            // Track liquids
            for (bx, row) in entry.blocks.iter().enumerate() {
                for (by, block) in row.iter().enumerate() {
                    if block.block_type.is_liquid() {
                        let world_bx = chunk_x * CHUNK_SIZE_X_BLOCKS as i32 + bx as i32;
                        let world_by = chunk_y * CHUNK_SIZE_Y_BLOCKS as i32 + by as i32;
                        self.active_liquids.insert((world_bx, world_by));
                    }
                }
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

    pub fn get_block_mut(&mut self, bx: i32, by: i32) -> Option<&mut Block> {
        let chunk_x = (bx as f32 / CHUNK_SIZE_X_BLOCKS as f32).floor() as i32;
        let chunk_y = (by as f32 / CHUNK_SIZE_Y_BLOCKS as f32).floor() as i32;

        let rel_x = ((bx % CHUNK_SIZE_X_BLOCKS as i32) + CHUNK_SIZE_X_BLOCKS as i32)
            % CHUNK_SIZE_X_BLOCKS as i32;
        let rel_y = ((by % CHUNK_SIZE_Y_BLOCKS as i32) + CHUNK_SIZE_Y_BLOCKS as i32)
            % CHUNK_SIZE_Y_BLOCKS as i32;

        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        let chunk = self
            .macrogrids
            .get_mut(&mg_coords)?
            .chunks
            .get_mut(&rel_coords)?;

        if chunk.is_generated {
            chunk.get_block(rel_x as usize, rel_y as usize)
        } else {
            None
        }
    }

    pub fn get_block_ref(&self, bx: i32, by: i32) -> Option<&Block> {
        let chunk_x = (bx as f32 / CHUNK_SIZE_X_BLOCKS as f32).floor() as i32;
        let chunk_y = (by as f32 / CHUNK_SIZE_Y_BLOCKS as f32).floor() as i32;

        let rel_x = ((bx % CHUNK_SIZE_X_BLOCKS as i32) + CHUNK_SIZE_X_BLOCKS as i32)
            % CHUNK_SIZE_X_BLOCKS as i32;
        let rel_y = ((by % CHUNK_SIZE_Y_BLOCKS as i32) + CHUNK_SIZE_Y_BLOCKS as i32)
            % CHUNK_SIZE_Y_BLOCKS as i32;

        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        let chunk = self.macrogrids.get(&mg_coords)?.chunks.get(&rel_coords)?;

        if chunk.is_generated {
            Some(&chunk.blocks[rel_x as usize][rel_y as usize])
        } else {
            None
        }
    }
}
