use super::WorldManager;
use crate::components::{BlockType, Chunk};
use crate::constants::*;
use crate::utils::chunk_to_macrogrid_coords;

impl WorldManager {
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
                    apply_chunk_save_data(chunk, &chunk_data);
                } else {
                    self.pending_modifications.insert((cx, cy), chunk_data);
                }
            }
        }
    }
}

pub fn apply_chunk_save_data(
    chunk: &mut Chunk,
    chunk_data: &crate::managers::persistence::ChunkSaveData,
) {
    // 1. Decode flat RLE blocks
    let mut current_idx = 0;
    for chunk_triplet in chunk_data.blocks.chunks(3) {
        if chunk_triplet.len() < 3 {
            break;
        }
        let type_id = chunk_triplet[0];
        let level = chunk_triplet[1] as u8;
        let count = chunk_triplet[2];
        let block_type = BlockType::from_id(type_id);

        for _ in 0..count {
            let bx = current_idx % CHUNK_SIZE_X_BLOCKS;
            let by = current_idx / CHUNK_SIZE_X_BLOCKS;

            if let Some(block) = chunk.get_block(bx, by) {
                block.block_type = block_type;
                block.is_modified = true;

                let new_sprite = block.block_type.get_sprite();

                block.sprite_rect = new_sprite;
                block.is_broken = block.block_type == BlockType::Air;
                block.current_hp = if block.is_broken { 0 } else { block.max_hp };
                block.liquid_level = level;
            }
            current_idx += 1;
        }
    }

    // 2. Apply named blocks
    for named_block in &chunk_data.named_blocks {
        let bx = named_block.i as usize / CHUNK_SIZE_Y_BLOCKS;
        let by = named_block.i as usize % CHUNK_SIZE_Y_BLOCKS;
        if let Some(block) = chunk.get_block(bx, by) {
            block.name = named_block.n.clone();
        }
    }

    chunk.is_modified_in_session = true;
}
