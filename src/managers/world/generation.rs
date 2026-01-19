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

    // Scaling for macrogrid - even larger scale for continents to avoid "too many islands"
    let x = mg_x as f64 * 0.025;
    let y = mg_y as f64 * 0.025;

    let plate_val = worley.get([x, y]);
    let plate_id = (plate_val.abs() * 1000.0) as u32;

    // Estimate geological stress by checking neighbors
    let eps = 0.01;
    let v0 = worley.get([x, y]);
    let v1 = worley.get([x + eps, y]);
    let v2 = worley.get([x, y + eps]);

    let d_stress = ((v0 - v1).abs() + (v0 - v2).abs()) as f32 * 80.0;
    let stress = d_stress.min(1.0);

    // Climate and Geology parameters
    let temperature_base = perlin.get([x * 1.2, y * 1.2, 0.0]) as f32;
    let humidity_base = perlin.get([x * 1.2, y * 1.2, 10.0]) as f32;
    let sediment_depth = perlin.get([x * 2.0, y * 2.0, 20.0]) as f32;
    
    // paleo_env determines Continent vs Ocean. 
    let paleo_env = perlin.get([x * 0.8, y * 0.8, 30.0]) as f32;
    
    let crust_thickness = perlin.get([x * 0.5, y * 0.5, 40.0]) as f32;
    
    let geohistory_seed = seed.wrapping_add((mg_x.wrapping_mul(34123) ^ mg_y.wrapping_mul(12347)) as u32);

    MacroCell {
        plate_id,
        geological_stress: stress,
        temperature_base,
        humidity_base,
        sediment_depth,
        paleo_env,
        geohistory_seed,
        crust_thickness,
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
    let mut blocks = Vec::with_capacity(CHUNK_SIZE_X_BLOCKS);

    // Seed-based noise generators for local features
    let local_seed = macro_cell.geohistory_seed;
    let cave_ridged = RidgedMulti::<Perlin>::new(local_seed);
    let cave_fbm = Fbm::<Perlin>::new(local_seed.wrapping_add(1));
    let strata_fbm = Fbm::<Perlin>::new(local_seed.wrapping_add(2));
    let detail_noise = Fbm::<Perlin>::new(local_seed.wrapping_add(3));
    let vein_noise = RidgedMulti::<Perlin>::new(local_seed.wrapping_add(4));

    let sea_level = SURFACE_Y_LEVEL as f32;

    for bx in 0..CHUNK_SIZE_X_BLOCKS {
        let mut row = Vec::with_capacity(CHUNK_SIZE_Y_BLOCKS);
        for by in 0..CHUNK_SIZE_Y_BLOCKS {
            let wx = origin_x + bx.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
            let wy = origin_y + by.to_f32().unwrap_or(0.0) * BLOCK_SIZE;

            let wx_f64 = wx.to_f64().unwrap_or(0.0);
            let wy_f64 = wy.to_f64().unwrap_or(0.0);

            let x_block = (wx / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let y_block = (wy / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let x_m = x_block as f64;
            let y_m = y_block as f64;

            // --- Phase 1: Surface Elevation ---
            let is_ocean = macro_cell.paleo_env < -0.1;
            
            let get_surface_y = |x_val: f64| {
                let landscape_noise = noise_main.get([x_val * 0.003, 100.0]) as f32;
                
                let jag_noise = noise_main.get([x_val * 0.04, 200.0]).abs() as f32;
                let mountain_val = macro_cell.geological_stress.powf(1.2) * 100.0;
                let mountain_height = mountain_val * (0.4 + jag_noise * 0.6);
                
                let plat_noise = noise_main.get([x_val * 0.006, 300.0]) as f32;
                let plat_effect = if !is_ocean && macro_cell.geological_stress > 0.3 {
                    (plat_noise * 8.0).tanh() * 20.0
                } else { 0.0 };

                let env_base = if is_ocean {
                    sea_level + 18.0 + (macro_cell.paleo_env.abs() * 45.0)
                } else {
                    sea_level - 6.0 - (macro_cell.paleo_env * 20.0)
                };

                (env_base - landscape_noise * 15.0 - mountain_height - plat_effect).floor() as i32
            };

            let surface_y_raw = get_surface_y(x_m);
            
            // River system
            let river_noise = noise_main.get([x_m * 0.012, 800.0]);
            let is_river = !is_ocean && macro_cell.humidity_base > 0.2 && river_noise > 0.82;
            let river_depth = if is_river { (river_noise - 0.82) * 60.0 } else { 0.0 };
            let surface_y = surface_y_raw + river_depth.floor() as i32;

            // --- Phase 2: Biome & Climate ---
            let temp = macro_cell.temperature_base;
            let humid = macro_cell.humidity_base;

            let (mut surface_block, soil_block, subsoil_block) = if temp < -0.4 {
                (BlockType::Permafrost, BlockType::Permafrost, BlockType::Gravel)
            } else if temp < -0.1 {
                (BlockType::Grass, BlockType::Dirt, BlockType::Gravel)
            } else if temp > 0.4 {
                if humid < -0.2 {
                    (BlockType::Sand, BlockType::Sand, BlockType::Sand)
                } else if humid > 0.3 {
                    (BlockType::Grass, BlockType::Dirt, BlockType::Dirt)
                } else {
                    (BlockType::Sand, BlockType::Dirt, BlockType::Dirt)
                }
            } else if humid > 0.4 {
                (BlockType::Grass, BlockType::Dirt, BlockType::Dirt)
            } else {
                (BlockType::Grass, BlockType::Dirt, BlockType::Stone)
            };
            
            if is_ocean {
                surface_block = if temp > 0.2 { BlockType::Sand } else { BlockType::Gravel };
            }

            // --- Phase 3: Strata Layers & Geological Structures ---
            let distort_x = x_m + strata_fbm.get([x_m * 0.002, y_m * 0.002]) * 50.0;
            let distort_y = y_m + strata_fbm.get([x_m * 0.002 + 100.0, y_m * 0.002]) * 50.0;

            let fold_v = strata_fbm.get([distort_x * 0.005, distort_y * 0.001]);
            let fold_offset = (fold_v as f32 * 50.0).floor() as i32;
            
            let fault_v = noise_main.get([x_m * 0.04, 500.0]);
            let is_fault_line = macro_cell.geological_stress > 0.5 && fault_v.abs() > 0.8;
            let fault_offset = if is_fault_line {
                (fault_v.signum() * (30.0 + f64::from(macro_cell.geological_stress) * 70.0)) as i32
            } else { 0 };

            let crust_mod = (macro_cell.crust_thickness * 150.0) as i32;
            let sedimentary_depth = 70 + (macro_cell.sediment_depth * 120.0) as i32 + fold_offset + fault_offset + crust_mod;
            let metamorphic_depth = sedimentary_depth + 160 + fold_offset + (crust_mod / 2);

            let intrusion_v = detail_noise.get([wx_f64 * 0.007, wy_f64 * 0.007, 777.0]);
            let is_intrusion = intrusion_v > 0.74 && y_block > surface_y + 30;
            let is_halo = !is_intrusion && intrusion_v > 0.62 && y_block > surface_y + 20;

            let pipe_grid = 250.0;
            let pg_x = (wx_f64 / pipe_grid).floor();
            let pipe_noise = detail_noise.get([pg_x * 17.0, 999.0]);
            let pipe_center = (pg_x * pipe_grid) + (pipe_noise.abs() * 0.7 + 0.15) * pipe_grid;
            let is_in_pipe = pipe_noise > 0.82 && (wx_f64 - pipe_center).abs() < (3.0 + pipe_noise * 4.0) && y_block > surface_y + 50;

            let vein_v = vein_noise.get([wx_f64 * 0.12, wy_f64 * 0.12, 888.0]);
            let is_vein = vein_v > 0.86 && y_block > surface_y + 20;

            let worm_v = cave_ridged.get([x_m * 0.07, y_m * 0.07, 1500.0]);
            let chamber_v = cave_fbm.get([x_m * 0.025, y_m * 0.025, 2500.0]);
            let is_cave = (worm_v > 0.87 || chamber_v > 0.82) && y_block > surface_y + 15;

            let magma_v = detail_noise.get([x_m * 0.04, y_m * 0.04, 4000.0]);
            let is_magma = magma_v > 0.85 && y_block > metamorphic_depth + 120;

            // --- Phase 4: Block Selection ---
            let player_start_x_block = (PLAYER_INITIAL_X / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
            let player_start_y_block = (PLAYER_INITIAL_Y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);

            let mut b_type;

            if x_block == player_start_x_block && y_block == player_start_y_block {
                b_type = BlockType::WarpGate;
            } else if x_block == player_start_x_block && y_block == player_start_y_block + 1 {
                b_type = BlockType::Indestructible;
            } else if y_block < surface_y_raw {
                b_type = if y_block >= sea_level.floor() as i32 { BlockType::Seawater } else { BlockType::Air };
            } else if is_river && y_block < surface_y {
                b_type = BlockType::Water;
            } else if is_cave && !is_magma && !is_in_pipe && !is_vein {
                b_type = BlockType::Air;
            } else if is_magma {
                b_type = BlockType::Magma;
            } else if y_block == surface_y {
                let is_cliff = (get_surface_y(x_m + 1.0) - get_surface_y(x_m - 1.0)).abs() > 3;
                b_type = if is_cliff && !is_ocean { BlockType::Stone } else { surface_block };
            } else {
                let d = y_block - surface_y;
                
                if d < 7 {
                    b_type = soil_block;
                } else if d < 15 {
                    let is_low = surface_y_raw > sea_level as i32 + 5;
                    b_type = if is_low && humid > 0.2 { BlockType::Gravel } else { subsoil_block };
                } else if is_in_pipe {
                    b_type = BlockType::Kimberlite;
                } else if is_intrusion {
                    let type_noise = detail_noise.get([x_m * 0.05, y_m * 0.05, 7000.0]);
                    b_type = if type_noise > 0.2 { BlockType::Basalt } else { BlockType::Granite };
                } else if d < sedimentary_depth {
                    let env = macro_cell.paleo_env + (strata_fbm.get([x_m * 0.002, y_m * 0.01]) as f32 * 0.5);
                    b_type = if env > 0.3 { BlockType::Stone }
                             else if env > 0.0 { BlockType::Limestone }
                             else if env > -0.3 { BlockType::OilShale }
                             else { BlockType::Chert };
                    
                    if is_halo && b_type == BlockType::Limestone {
                        b_type = BlockType::Marble;
                    }
                    
                    if (b_type == BlockType::Limestone || b_type == BlockType::Gravel) && chamber_v > 0.75 {
                        b_type = BlockType::Water;
                    }
                } else if d < metamorphic_depth {
                    let m_noise = detail_noise.get([x_m * 0.06, y_m * 0.06, 6000.0]);
                    b_type = if is_halo || m_noise > 0.25 { BlockType::Marble } else { BlockType::Schist };
                } else {
                    let d_noise = detail_noise.get([x_m * 0.04, y_m * 0.04, 7000.0]);
                    b_type = if d_noise > 0.3 { BlockType::Basalt } else { BlockType::Granite };
                    
                    if d > metamorphic_depth + 1200 {
                        b_type = BlockType::Peridotite;
                    }
                }

                // --- Mineralization Logic ---
                let ore_rand = noise_ore.get([wx_f64 * 0.2, wy_f64 * 0.2]);
                let cluster_rand = noise_ore.get([wx_f64 * 0.05, wy_f64 * 0.05, 888.0]);

                if is_in_pipe {
                    if ore_rand > 0.65 { b_type = BlockType::Diamond; }
                } else if is_vein {
                    b_type = if ore_rand > 0.6 { BlockType::NativeGold }
                             else if ore_rand > 0.2 { BlockType::Quartz }
                             else if ore_rand < -0.7 { BlockType::Galena }
                             else if ore_rand < -0.4 { BlockType::NativePlatinum }
                             else { b_type };
                } else if is_halo && b_type == BlockType::Marble {
                    if ore_rand > 0.5 { b_type = BlockType::Hematite; }
                    else if ore_rand < -0.5 { b_type = BlockType::Chalcopyrite; }
                } else if cluster_rand > 0.68 {
                    if d < sedimentary_depth {
                        if ore_rand > 0.55 { b_type = BlockType::Coal; }
                        else if ore_rand < -0.7 && temp > 0.3 { b_type = BlockType::Halite; }
                        else if ore_rand < -0.6 { b_type = BlockType::Gypsum; }
                    } else if d < metamorphic_depth {
                        if ore_rand > 0.7 { b_type = BlockType::Graphite; }
                        else if ore_rand < -0.8 { b_type = BlockType::Ruby; }
                        else if ore_rand > 0.75 { b_type = BlockType::Spodumene; }
                    } else if ore_rand > 0.65 {
                        b_type = BlockType::Hematite;
                    } else if ore_rand < -0.65 {
                        b_type = BlockType::Chalcopyrite;
                    } else if ore_rand > 0.8 {
                        b_type = BlockType::Cassiterite;
                    }
                }
                
                if is_cave && (worm_v > 0.85 || chamber_v > 0.8) {
                    let geode = detail_noise.get([x_m * 0.4, y_m * 0.4, 999.0]);
                    if geode > 0.82 { b_type = BlockType::Quartz; }
                    else if geode < -0.9 { b_type = BlockType::Ruby; }
                }
            }

            if y_block > 3000 { b_type = BlockType::Indestructible; }

            let base_hardness = b_type.get_base_hardness();
            let s_rect = b_type.get_sprite();
            let hp = if base_hardness == -1 { -1 } else {
                let depth = (y_block - SURFACE_Y_LEVEL).max(0).to_f64().unwrap_or(0.0);
                let multiplier = 1.0 + depth * HARDNESS_DEPTH_MULTIPLIER;
                (base_hardness.to_f64().unwrap_or(0.0) * multiplier).floor() as i32
            };

            let mut b = Block::new(wx, wy, hp, s_rect, b_type);
            if b_type == BlockType::WarpGate {
                b.name = Some("Home".to_string());
                b.back_type = BlockType::Air;
            }
            row.push(b);
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
