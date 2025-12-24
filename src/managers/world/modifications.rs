use crate::components::{BlockType, Chunk};
use crate::constants::*;

pub fn apply_chunk_save_data(
    chunk: &mut Chunk,
    chunk_data: &crate::managers::persistence::ChunkSaveData,
) {
    // 1. Decode flat RLE blocks
    let mut current_idx = 0;
    for chunk_pair in chunk_data.blocks.chunks(2) {
        if chunk_pair.len() < 2 {
            break;
        }
        let type_id = chunk_pair[0];
        let count = chunk_pair[1];
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
