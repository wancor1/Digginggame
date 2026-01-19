use crate::components::{Block, BlockType, MacroCell};
use crate::constants::{
    BLOCK_SIZE, CHUNK_SIZE_X_BLOCKS, CHUNK_SIZE_Y_BLOCKS, HARDNESS_DEPTH_MULTIPLIER,
    PLAYER_INITIAL_X, PLAYER_INITIAL_Y, SURFACE_Y_LEVEL,
};
use macroquad::prelude::*;
use noise::{Fbm, NoiseFn, Perlin, RidgedMulti, Worley};
use num_traits::ToPrimitive;

#[must_use]
pub fn generate_macro_cell(mg_x: i32, mg_y: i32, seed: u32) -> MacroCell {
    let worley = Worley::new(seed);
    let perlin = Perlin::new(seed.wrapping_add(1));

    // Scaling for macrogrid - larger scale for plates
    let x = mg_x as f64 * 0.05;
    let y = mg_y as f64 * 0.05;

    let plate_val = worley.get([x, y]);
    let plate_id = (plate_val.abs() * 1000.0) as u32;

    // Estimate geological stress by checking neighbors for ID changes
    // Improved to a continuous value
    let eps = 0.02;
    let v0 = worley.get([x, y]);
    let v1 = worley.get([x + eps, y]);
    let v2 = worley.get([x, y + eps]);

    let d_stress = ((v0 - v1).abs() + (v0 - v2).abs()) as f32 * 50.0;
    let stress = d_stress.min(1.0);

    // Climate parameters
    let temperature_base = perlin.get([x * 2.0, y * 2.0, 0.0]) as f32;
    let humidity_base = perlin.get([x * 2.0, y * 2.0, 10.0]) as f32;
    let sediment_depth = perlin.get([x * 2.0, y * 2.0, 20.0]) as f32;
    let paleo_env = perlin.get([x * 1.5, y * 1.5, 30.0]) as f32;
    let geohistory_seed = seed.wrapping_add((mg_x ^ mg_y) as u32);

    MacroCell {
        plate_id,
        geological_stress: stress,
        temperature_base,
        humidity_base,
        sediment_depth,
        paleo_env,
        geohistory_seed,
    }
}

#[must_use]
pub fn generate_chunk_blocks(
    chunk_x: i32,
    chunk_y: i32,
    noise_main: &Perlin,
    noise_ore: &Perlin,
    macro_cell: &MacroCell,
) -> Vec<Vec<Block>> {
    let (origin_x, origin_y) = chunk_coords_to_world_origin(chunk_x, chunk_y);
    let mut blocks = Vec::new();

    // Advanced noise setups
    let cave_ridged = RidgedMulti::<Perlin>::new(macro_cell.geohistory_seed);
    let strata_fbm = Fbm::<Perlin>::new(macro_cell.geohistory_seed.wrapping_add(5));

    // Step 3: Baseline Strata and Elevation
    // Surface elevation influenced by geological stress
    let surface_base = SURFACE_Y_LEVEL as f32;

    for bx in 0..CHUNK_SIZE_X_BLOCKS {
        let mut row = Vec::new();
        for by in 0..CHUNK_SIZE_Y_BLOCKS {
            let wx = origin_x + bx.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
            let wy = origin_y + by.to_f32().unwrap_or(0.0) * BLOCK_SIZE;

            let wx_f64 = wx.to_f64().unwrap_or(0.0);
            let wy_f64 = wy.to_f64().unwrap_or(0.0);

            let y_block = (wy / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let x_block = (wx / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let x_m = x_block as f64; // 1 block = 1 meter

            // Phase 9: Basin and Plateau dynamics
            let get_surface_y = |x_val: f64| {
                let s_noise = noise_main.get([x_val * 0.005, 0.0]) as f32 * 15.0;
                
                let s_stress = macro_cell.geological_stress * 60.0;
                
                let basin_effect = if macro_cell.sediment_depth > 0.3 {
                    (macro_cell.sediment_depth - 0.3) * 40.0
                } else { 0.0 };
                
                let plateau_noise = noise_main.get([x_val * 0.002, 500.0]).abs() as f32;
                let plateau_effect = if macro_cell.geological_stress > 0.6 && plateau_noise < 0.2 {
                    25.0
                } else { 0.0 };

                (surface_base - s_noise - s_stress + basin_effect - plateau_effect).floor() as i32
            };

            let surface_y_raw = get_surface_y(x_m);
            
            // Phase 9: River system (Fluviology)
            let river_noise = noise_main.get([x_m * 0.015, 800.0]);
            let is_river = macro_cell.humidity_base > 0.1 && river_noise > 0.82;
            let river_depth = if is_river { (river_noise - 0.82) * 50.0 } else { 0.0 };
            let surface_y = surface_y_raw + river_depth.floor() as i32;

            // Step 4: Erosion & Outcrops
            let y_left = get_surface_y(x_m - 1.0);
            let y_right = get_surface_y(x_m + 1.0);
            let slope = (y_right - y_left).abs();
            let is_eroded = slope > 2;

            // Step 5 & 10: Detailed Biomes
            let temp = macro_cell.temperature_base;
            let humid = macro_cell.humidity_base;

            let (surface_block, soil_block, subsoil_block) = if temp < -0.3 {
                (BlockType::Permafrost, BlockType::Permafrost, BlockType::Gravel)
            } else if temp > 0.4 {
                if humid < -0.2 {
                    (BlockType::Sand, BlockType::Sand, BlockType::Sand)
                } else {
                    (BlockType::Grass, BlockType::Dirt, BlockType::Dirt)
                }
            } else {
                if humid > 0.5 {
                    (BlockType::Grass, BlockType::Dirt, BlockType::Gravel)
                } else {
                    (BlockType::Grass, BlockType::Dirt, BlockType::Stone)
                }
            };

            // Phase 9: Intrusive Structures (Dykes and Sills)
            let dyke_noise = noise_main.get([x_m * 0.012, 0.0, 900.0]);
            let is_dyke = dyke_noise > 0.95;
            let sill_noise = noise_main.get([0.0, y_block as f64 * 0.008, 1100.0]);
            let is_sill = sill_noise > 0.96 && y_block > surface_y + 50;

            // Folding (褶曲) using FBM for better complexity
            let fold_noise = (strata_fbm.get([x_m * 0.002, 100.0]) as f32 * 50.0).floor() as i32;
            
            // Faulting (断層)
            let fault_offset = if macro_cell.geological_stress > 0.85 {
                let f_noise = noise_main.get([x_m * 0.04, 200.0]);
                if f_noise > 0.7 { 30 } else if f_noise < -0.7 { -30 } else { 0 }
            } else {
                0
            };

            // Phase 15: Unconformity (不整合)
            let unconformity_y = 400; 
            let is_below_unconformity = y_block > unconformity_y;
            let unconformity_tilt = if is_below_unconformity {
                (noise_main.get([x_m * 0.01, 1200.0]) * 50.0) as i32
            } else { 0 };

            let sedimentary_depth =
                60 + (macro_cell.sediment_depth * 80.0) as i32 + fold_noise + fault_offset + unconformity_tilt;

            let metamorphic_depth = sedimentary_depth + 120 + fold_noise;

            // Phase 11: Natural Caves (Worm Caves using RidgedMulti)
            let cave_val = cave_ridged.get([x_m * 0.05, y_block as f64 * 0.05, 1500.0]);
            let is_cave = cave_val > 0.85 && y_block > surface_y + 15;

            // Phase 14: Karst Dynamics (鍾乳洞)
            let karst_noise = noise_main.get([x_m * 0.02, wy_f64 * 0.02, 2000.0]);

            // Phase 11: Geodes
            let geode_noise = noise_main.get([x_m * 0.08, y_block as f64 * 0.08, 3000.0]);
            let is_geode_center = geode_noise > 0.985 && y_block > metamorphic_depth;
            
            // Initial Spawn Point - Warp Gate
            let player_start_x_block = (PLAYER_INITIAL_X / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let player_start_y_block = (PLAYER_INITIAL_Y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);

            let (max_hp, sprite_rect, block_type) =
                if x_block == player_start_x_block && y_block == player_start_y_block {
                    let bt = BlockType::WarpGate;
                    (50, bt.get_sprite(), bt)
                } else if x_block == player_start_x_block && y_block == player_start_y_block + 1 {
                    let bt = BlockType::Indestructible;
                    (bt.get_base_hardness(), bt.get_sprite(), bt)
                } else if y_block < surface_y_raw && !is_river {
                    (0, None, BlockType::Air)
                } else if is_river && y_block < surface_y_raw {
                    (0, None, BlockType::Air)
                } else if is_river && y_block < surface_y {
                    let bt = BlockType::Water;
                    (bt.get_base_hardness(), bt.get_sprite(), bt)
                } else if is_cave && !is_dyke && !is_sill {
                    // Standard cave carving
                    (0, None, BlockType::Air)
                } else if is_geode_center {
                    (0, None, BlockType::Air)
                } else if y_block == surface_y {
                    // Surface determination with erosion
                    let bt = if is_eroded { BlockType::Stone } else { surface_block };
                    (bt.get_base_hardness(), bt.get_sprite(), bt)
                } else {
                    let mut b_type = soil_block;

                    if y_block > surface_y {
                        let relative_depth = y_block - surface_y;
                        
                        // Phase 9: Soil Horizons and Intrusions
                        if relative_depth < 4 && !is_dyke && !is_sill {
                            b_type = soil_block;
                        } else if relative_depth < 12 && !is_dyke && !is_sill {
                            b_type = subsoil_block;
                        } else if is_dyke || is_sill {
                            // Dyke/Sill intrusion
                            b_type = if relative_depth > metamorphic_depth { BlockType::Basalt } else { BlockType::Granite };
                        } else if relative_depth < sedimentary_depth {
                            // Phase 13: Bio-sedimentary Layers
                            let sedimentary_base = if macro_cell.paleo_env > 0.2 {
                                BlockType::Dirt // Former Land
                            } else if macro_cell.paleo_env > -0.2 {
                                BlockType::Limestone // Former Shallow Sea
                            } else {
                                BlockType::Chert // Former Deep Sea
                            };

                            if is_eroded && relative_depth < 6 {
                                let outcrop_noise = noise_ore.get([x_m * 0.1, y_block as f64 * 0.1]);
                                b_type = if outcrop_noise > 0.7 { BlockType::Coal } else { BlockType::Stone };
                            } else {
                                let stone_patch = noise_main.get([x_m * 0.06, wy_f64 * 0.06]);
                                b_type = if stone_patch > 0.3 { BlockType::Stone } else { sedimentary_base };
                                
                                // Phase 14: Limestone caves
                                if b_type == BlockType::Limestone && karst_noise > 0.65 {
                                    b_type = BlockType::Air;
                                }

                                if relative_depth > 20 && b_type != BlockType::Air {
                                    let rock_noise = noise_main.get([x_m * 0.1, wy_f64 * 0.1, 10.0]);
                                    if rock_noise > 0.7 { b_type = BlockType::OilShale; }
                                }
                            }
                        } else if relative_depth < metamorphic_depth {
                            b_type = BlockType::Schist;
                            let meta_noise = noise_main.get([x_m * 0.1, wy_f64 * 0.1]);
                            if meta_noise > 0.4 { b_type = BlockType::Marble; }
                        } else {
                            b_type = BlockType::Granite;
                            let igneous_noise = noise_main.get([x_m * 0.07, wy_f64 * 0.07]);
                            if igneous_noise > 0.5 { b_type = BlockType::Basalt; }
                            
                            if relative_depth > metamorphic_depth + 1000 { b_type = BlockType::Indestructible; }
                        }

                        // Phase 16: Hydrothermal Alteration
                        if !is_dyke && !is_sill {
                            let dist_to_dyke = (dyke_noise - 0.95).abs();
                            if dist_to_dyke < 0.02 {
                                // Altered Zone
                                if b_type == BlockType::Stone || b_type == BlockType::Limestone {
                                    b_type = BlockType::Quartz; // Silicification
                                }
                            }
                        }

                        // Geode Surroundings
                        if geode_noise > 0.96 && b_type != BlockType::Air {
                            b_type = if geode_noise > 0.98 { BlockType::Ruby } else { BlockType::Quartz };
                        }

                        // --- Phase 8: Mineralization Process ---
                        let ore_noise_val = noise_ore.get([wx_f64 * 0.12, wy_f64 * 0.12]);
                        let vein_noise = noise_ore.get([wx_f64 * 0.04, wy_f64 * 0.2, 100.0]); 
                        
                        // 1. Sedimentary & Weathering Deposits
                        if relative_depth < sedimentary_depth {
                            if temp > 0.5 && humid > 0.5 && relative_depth < 18 {
                                if ore_noise_val > 0.75 { b_type = BlockType::Bauxite; }
                            }
                            
                            if temp > 0.3 && humid < -0.4 && relative_depth > 12 {
                                if ore_noise_val > 0.65 { b_type = BlockType::Halite; }
                                else if ore_noise_val < -0.75 { b_type = BlockType::Gypsum; }
                            }

                            if ore_noise_val > 0.78 {
                                b_type = BlockType::Coal;
                            }
                        }
                        
                        // 2. Metamorphic Deposits
                        else if relative_depth < metamorphic_depth {
                            if ore_noise_val > 0.82 {
                                b_type = BlockType::Graphite;
                            } else if ore_noise_val < -0.88 {
                                b_type = BlockType::Ruby;
                            }
                        }
                        
                        // 3. Igneous & Magmatic Deposits
                        else {
                            if ore_noise_val > 0.75 {
                                b_type = BlockType::Hematite; 
                            } else if ore_noise_val < -0.75 {
                                b_type = BlockType::Chalcopyrite; 
                            }
                            
                            if relative_depth > metamorphic_depth + 400 && ore_noise_val > 0.92 {
                                b_type = BlockType::NativePlatinum;
                            }

                            let pipe_noise = noise_ore.get([wx_f64 * 0.03, 0.0, 500.0]);
                            if pipe_noise > 0.93 {
                                b_type = BlockType::Kimberlite;
                                if noise_ore.get([wx_f64 * 0.5, wy_f64 * 0.5]) > 0.6 {
                                    b_type = BlockType::Diamond;
                                }
                            }
                        }

                        // 4. Hydrothermal Veins
                        if macro_cell.geological_stress > 0.6 && vein_noise.abs() > 0.88 {
                            if relative_depth > 40 {
                                let vein_type_roll = noise_ore.get([wx_f64 * 0.02, wy_f64 * 0.02, 300.0]);
                                b_type = if vein_type_roll > 0.6 {
                                    BlockType::NativeGold
                                } else if vein_type_roll > 0.2 {
                                    BlockType::Galena 
                                } else if vein_type_roll > -0.3 {
                                    BlockType::Sphalerite 
                                } else {
                                    BlockType::Quartz
                                };
                            }
                        }

                        let boundary_dist = (relative_depth - metamorphic_depth).abs();
                        if boundary_dist < 12 && ore_noise_val > 0.85 {
                            b_type = BlockType::Spodumene;
                        }
                    }

                    if y_block > 2000 {
                        b_type = BlockType::Indestructible;
                    }

                let base_hardness = b_type.get_base_hardness();
                let s_rect = b_type.get_sprite();

                let hp = if base_hardness == -1 {
                    -1
                } else {
                    let depth = (y_block - surface_y).to_f64().unwrap_or(0.0);
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
                b.back_type = BlockType::Air; 
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