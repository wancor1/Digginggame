use super::Game;
use crate::components::OwnedItem;
use crate::constants::*;
use crate::managers::persistence::{
    BlockSaveData, ChunkSaveData, ItemStack, SAVE_VERSION, SaveData,
};

impl Game {
    fn stack_items(items: &[OwnedItem]) -> Vec<ItemStack> {
        let mut stacks: Vec<ItemStack> = Vec::new();
        for item in items {
            if let Some(stack) = stacks.iter_mut().find(|s| {
                s.item_type == item.item_type
                    && s.is_natural == item.is_natural
                    && s.is_auto_stored == item.is_auto_stored
            }) {
                stack.count += 1;
            } else {
                stacks.push(ItemStack {
                    item_type: item.item_type.clone(),
                    count: 1,
                    is_natural: item.is_natural,
                    is_auto_stored: item.is_auto_stored,
                });
            }
        }
        stacks
    }

    pub fn make_save_data(&self) -> SaveData {
        let mut modified_macrogrids: Vec<crate::managers::persistence::MacroGridSaveData> =
            Vec::new();

        for (&(mgx, mgy), macrogrid) in self.world_manager.macrogrids.iter() {
            let mut chunks_in_mg: Vec<ChunkSaveData> = Vec::new();

            for (&(rel_cx, rel_cy), chunk) in macrogrid.chunks.iter() {
                if chunk.is_modified_in_session {
                    let cx = mgx * MACROGRID_SIZE_CHUNKS as i32 + rel_cx;

                    let cy = mgy * MACROGRID_SIZE_CHUNKS as i32 + rel_cy;

                    let mut rle_blocks: Vec<u32> = Vec::new();

                    let mut named_blocks: Vec<BlockSaveData> = Vec::new();

                    let mut last_type_id: Option<u32> = None;

                    let mut current_count: u32 = 0;

                    // Scan row-major (y then x) for better horizontal RLE runs

                    for by in 0..CHUNK_SIZE_Y_BLOCKS {
                        for bx in 0..CHUNK_SIZE_X_BLOCKS {
                            let block = &chunk.blocks[bx][by];

                            let type_id = block.block_type.to_id();

                            let index = (bx * CHUNK_SIZE_Y_BLOCKS + by) as u32;

                            if let Some(name) = &block.name {
                                named_blocks.push(BlockSaveData {
                                    i: index,

                                    t: block.block_type,

                                    n: Some(name.clone()),
                                });
                            }

                            if let Some(last_id) = last_type_id {
                                if last_id == type_id && current_count < u32::MAX {
                                    current_count += 1;
                                } else {
                                    rle_blocks.push(last_id);

                                    rle_blocks.push(current_count);

                                    last_type_id = Some(type_id);

                                    current_count = 1;
                                }
                            } else {
                                last_type_id = Some(type_id);

                                current_count = 1;
                            }
                        }
                    }

                    if let Some(last_id) = last_type_id {
                        rle_blocks.push(last_id);

                        rle_blocks.push(current_count);
                    }

                    chunks_in_mg.push(ChunkSaveData {
                        cx,

                        cy,

                        blocks: rle_blocks,

                        named_blocks,
                    });
                }
            }

            if !chunks_in_mg.is_empty() {
                modified_macrogrids.push(crate::managers::persistence::MacroGridSaveData {
                    mgx,

                    mgy,

                    chunks: chunks_in_mg,
                });
            }
        }

        // Add pending modifications (chunks that were not loaded in this session)

        for chunk_data in self.world_manager.pending_modifications.values() {
            let (mg_coords, _) =
                crate::utils::chunk_to_macrogrid_coords(chunk_data.cx, chunk_data.cy);

            if let Some(mg_data) = modified_macrogrids
                .iter_mut()
                .find(|mg| mg.mgx == mg_coords.0 && mg.mgy == mg_coords.1)
            {
                mg_data.chunks.push(chunk_data.clone());
            } else {
                modified_macrogrids.push(crate::managers::persistence::MacroGridSaveData {
                    mgx: mg_coords.0,

                    mgy: mg_coords.1,

                    chunks: vec![chunk_data.clone()],
                });
            }
        }

        SaveData {
            version: SAVE_VERSION,

            camera_x: self.camera.x,

            camera_y: self.camera.y,

            player_x: self.player_manager.player.x,

            player_y: self.player_manager.player.y,

            player_money: self.player_manager.player.money,

            player_fuel: self.player_manager.player.fuel,

            player_max_fuel: self.player_manager.player.max_fuel,

            player_cargo: Self::stack_items(&self.player_manager.player.cargo),

            player_max_cargo: self.player_manager.player.max_cargo,

            player_storage: Self::stack_items(&self.player_manager.player.storage),

            player_max_storage: self.player_manager.player.max_storage,

            player_drill_level: self.player_manager.player.drill_level,

            player_tank_level: self.player_manager.player.tank_level,

            player_engine_level: self.player_manager.player.engine_level,

            player_cargo_level: self.player_manager.player.cargo_level,

            player_warp_gates: self.player_manager.player.warp_gates.clone(),

            world_seed_main: self.world_manager.world_seed_main,

            world_seed_ore: self.world_manager.world_seed_ore,

            visited_chunks: self.world_manager.visited_chunks.clone(),

            modified_macrogrids,
        }
    }
}
