use super::WorldManager;
use super::generation;
use super::modifications;
use crate::components::{Block, BlockPos, Chunk, ChunkRelPos, MacroCell};
use crate::constants::{CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS};
use crate::utils::{
    chunk_to_macrogrid_coords, world_to_chunk_coords, world_to_relative_in_chunk_coords,
};
use num_traits::ToPrimitive;

impl WorldManager {
    pub fn ensure_macrocell_exists(&mut self, mg_coords: BlockPos) {
        let macrogrid = self.macrogrids.entry(mg_coords).or_default();
        if macrogrid.cell.is_none() {
            macrogrid.cell = Some(generation::generate_macro_cell(
                mg_coords.x,
                mg_coords.y,
                self.world_seed_main,
            ));
        }
    }

    pub fn get_interpolated_macro_cell(&mut self, chunk_x: i32, chunk_y: i32) -> MacroCell {
        use crate::constants::MACROGRID_SIZE_CHUNKS;
        let m_size = MACROGRID_SIZE_CHUNKS as f32;

        // Calculate macro-grid space coordinates of the chunk center
        // Each macro-grid sample is at its center: (mg_x * 8 + 3.5, mg_y * 8 + 3.5)
        let fx = (chunk_x as f32 - (m_size - 1.0) / 2.0) / m_size;
        let fy = (chunk_y as f32 - (m_size - 1.0) / 2.0) / m_size;

        let x0 = fx.floor() as i32;
        let y0 = fy.floor() as i32;
        let x1 = x0 + 1;
        let y1 = y0 + 1;

        let tx = fx - fx.floor();
        let ty = fy - fy.floor();

        let mut get_cell = |x: i32, y: i32| {
            let pos = BlockPos::new(x, y);
            self.ensure_macrocell_exists(pos);
            self.macrogrids
                .get(&pos)
                .and_then(|m| m.cell.as_ref())
                .cloned()
                .unwrap_or_else(|| generation::generate_macro_cell(x, y, self.world_seed_main))
        };

        let c00 = get_cell(x0, y0);
        let c10 = get_cell(x1, y0);
        let c01 = get_cell(x0, y1);
        let c11 = get_cell(x1, y1);

        // Bilinear interpolation
        let lerp_f32 = |v0: f32, v1: f32, t: f32| v0 + (v1 - v0) * t;

        let interp_stress = lerp_f32(
            lerp_f32(c00.geological_stress, c10.geological_stress, tx),
            lerp_f32(c01.geological_stress, c11.geological_stress, tx),
            ty,
        );
        let interp_temp = lerp_f32(
            lerp_f32(c00.temperature_base, c10.temperature_base, tx),
            lerp_f32(c01.temperature_base, c11.temperature_base, tx),
            ty,
        );
        let interp_humidity = lerp_f32(
            lerp_f32(c00.humidity_base, c10.humidity_base, tx),
            lerp_f32(c01.humidity_base, c11.humidity_base, tx),
            ty,
        );
        let interp_sediment = lerp_f32(
            lerp_f32(c00.sediment_depth, c10.sediment_depth, tx),
            lerp_f32(c01.sediment_depth, c11.sediment_depth, tx),
            ty,
        );
        let interp_paleo = lerp_f32(
            lerp_f32(c00.paleo_env, c10.paleo_env, tx),
            lerp_f32(c01.paleo_env, c11.paleo_env, tx),
            ty,
        );
        let interp_crust = lerp_f32(
            lerp_f32(c00.crust_thickness, c10.crust_thickness, tx),
            lerp_f32(c01.crust_thickness, c11.crust_thickness, tx),
            ty,
        );

        // Plate ID and Geohistory are NOT interpolated, use the nearest
        let (plate_id, geohistory_seed) = if tx < 0.5 {
            if ty < 0.5 { (c00.plate_id, c00.geohistory_seed) } else { (c01.plate_id, c01.geohistory_seed) }
        } else if ty < 0.5 {
            (c10.plate_id, c10.geohistory_seed)
        } else {
            (c11.plate_id, c11.geohistory_seed)
        };

        MacroCell {
            plate_id,
            geological_stress: interp_stress,
            temperature_base: interp_temp,
            humidity_base: interp_humidity,
            sediment_depth: interp_sediment,
            paleo_env: interp_paleo,
            geohistory_seed,
            crust_thickness: interp_crust,
        }
    }

    pub fn ensure_chunk_exists_and_generated(&mut self, chunk_x: i32, chunk_y: i32) {
        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);

        // Get interpolated macrocell data for this chunk. This also ensures macrocells exist.
        let macro_cell = self.get_interpolated_macro_cell(chunk_x, chunk_y);

        let macrogrid = self.macrogrids.entry(mg_coords).or_default();

        let entry = macrogrid
            .chunks
            .entry(rel_coords)
            .or_insert_with(|| Chunk::new(chunk_x, chunk_y));

        if !entry.is_generated {
            entry.blocks = generation::generate_chunk_blocks(
                chunk_x,
                chunk_y,
                &self.noise_main,
                &self.noise_ore,
                &macro_cell,
            );
            entry.is_generated = true;
            self.generated_chunk_coords
                .insert(BlockPos::new(chunk_x, chunk_y));

            // Apply pending modifications if any (Lazy Loading)
            if let Some(mod_data) = self
                .pending_modifications
                .remove(&BlockPos::new(chunk_x, chunk_y))
            {
                modifications::apply_chunk_save_data(entry, &mod_data);
            }

            // Track liquids - Only activate if potentially unstable
            let chunk_x_blocks = chunk_x * CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0);
            let chunk_y_blocks = chunk_y * CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0);
            
            // To properly check neighbors, we'd need adjacent chunks. 
            // For simplicity and performance during generation, we'll activate 
            // liquid blocks that are at the boundaries of the chunk or have non-full/different neighbors within the chunk.
            for bx in 0..CHUNK_SIZE_X_BLOCKS {
                for by in 0..CHUNK_SIZE_Y_BLOCKS {
                    let block = &entry.blocks[bx][by];
                    if block.block_type.is_liquid() {
                        let mut unstable = false;
                        if block.liquid_level < 8 {
                            unstable = true;
                        } else {
                            // Check neighbors within this chunk
                            let neighbors = [
                                (bx as i32 - 1, by as i32),
                                (bx as i32 + 1, by as i32),
                                (bx as i32, by as i32 - 1),
                                (bx as i32, by as i32 + 1),
                            ];
                            for (nx, ny) in neighbors {
                                if nx < 0 || nx >= CHUNK_SIZE_X_BLOCKS as i32 || ny < 0 || ny >= CHUNK_SIZE_Y_BLOCKS as i32 {
                                    // Boundary blocks are considered potentially unstable to trigger cross-chunk flow
                                    unstable = true;
                                    break;
                                }
                                let nb = &entry.blocks[nx as usize][ny as usize];
                                if !nb.block_type.is_solid() && (nb.block_type != block.block_type || nb.liquid_level < 8) {
                                    unstable = true;
                                    break;
                                }
                            }
                        }

                        if unstable {
                            let world_bx = chunk_x_blocks + bx.to_i32().unwrap_or(0);
                            let world_by = chunk_y_blocks + by.to_i32().unwrap_or(0);
                            self.active_liquids.insert(BlockPos::new(world_bx, world_by));
                        }
                    }
                }
            }
        }

        self.visited_chunks.insert(BlockPos::new(chunk_x, chunk_y));
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
        let BlockPos { x: cx, y: cy } = world_to_chunk_coords(world_x, world_y);
        let ChunkRelPos { x: rel_x, y: rel_y } =
            world_to_relative_in_chunk_coords(world_x, world_y);

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
        let chunk_x = (bx.to_f32().unwrap_or(0.0) / CHUNK_SIZE_X_BLOCKS.to_f32().unwrap_or(0.0))
            .floor()
            .to_i32()
            .unwrap_or(0);
        let chunk_y = (by.to_f32().unwrap_or(0.0) / CHUNK_SIZE_Y_BLOCKS.to_f32().unwrap_or(0.0))
            .floor()
            .to_i32()
            .unwrap_or(0);

        let rel_x = ((bx % CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0))
            + CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0))
            % CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0);
        let rel_y = ((by % CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0))
            + CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0))
            % CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0);

        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        let chunk = self
            .macrogrids
            .get_mut(&mg_coords)?
            .chunks
            .get_mut(&rel_coords)?;

        if chunk.is_generated {
            chunk.get_block(rel_x.to_usize().unwrap_or(0), rel_y.to_usize().unwrap_or(0))
        } else {
            None
        }
    }

    pub fn get_block_ref(&self, bx: i32, by: i32) -> Option<&Block> {
        let chunk_x = (bx.to_f32().unwrap_or(0.0) / CHUNK_SIZE_X_BLOCKS.to_f32().unwrap_or(0.0))
            .floor()
            .to_i32()
            .unwrap_or(0);
        let chunk_y = (by.to_f32().unwrap_or(0.0) / CHUNK_SIZE_Y_BLOCKS.to_f32().unwrap_or(0.0))
            .floor()
            .to_i32()
            .unwrap_or(0);

        let rel_x = ((bx % CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0))
            + CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0))
            % CHUNK_SIZE_X_BLOCKS.to_i32().unwrap_or(0);
        let rel_y = ((by % CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0))
            + CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0))
            % CHUNK_SIZE_Y_BLOCKS.to_i32().unwrap_or(0);

        let (mg_coords, rel_coords) = chunk_to_macrogrid_coords(chunk_x, chunk_y);
        let chunk = self.macrogrids.get(&mg_coords)?.chunks.get(&rel_coords)?;

        if chunk.is_generated {
            Some(&chunk.blocks[rel_x.to_usize().unwrap_or(0)][rel_y.to_usize().unwrap_or(0)])
        } else {
            None
        }
    }
}
