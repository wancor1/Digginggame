use super::WorldManager;
use crate::components::{BlockPos, BlockType};
use crate::constants::BLOCK_SIZE;
use num_traits::ToPrimitive;
use std::collections::HashSet;

impl WorldManager {
    pub fn update_liquids(&mut self, camera_x: f32, camera_y: f32) {
        if self.active_liquids.is_empty() {
            return;
        }

        self.liquid_tick_counter += 1;

        let cam_bx = (camera_x / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
        let cam_by = (camera_y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
        let radius = 40;
        let max_process_per_frame = 256;
        let mut processed_count = 0;

        // Pre-allocate search buffers to avoid allocations in the loop
        let mut visited_scratch = HashSet::with_capacity(512);

        // Use a temporary vector to avoid borrowing issues while iterating
        let mut active: Vec<BlockPos> = self.active_liquids.iter().copied().collect();
        
        // Sort by distance to player to prioritize nearby liquids
        active.sort_unstable_by_key(|p| (p.x - cam_bx).abs() + (p.y - cam_by).abs());

        let mut next_active = HashSet::with_capacity(self.active_liquids.len());

        for pos in active {
            let (bx, by) = (pos.x, pos.y);
            
            // Distance check
            let dist_x = (bx - cam_bx).abs();
            let dist_y = (by - cam_by).abs();
            if dist_x > radius || dist_y > radius {
                if dist_x < radius + 20 && dist_y < radius + 20 {
                    next_active.insert(pos);
                }
                continue;
            }

            // Processing limit to prevent frame spikes
            if processed_count >= max_process_per_frame {
                next_active.insert(pos);
                continue;
            }

            let (level, b_type, interval) = if let Some(b) = self.get_block_ref(bx, by) {
                if !b.block_type.is_liquid() || b.liquid_level == 0 {
                    continue;
                }
                let interval = b
                    .block_type
                    .get_data()
                    .map_or(1, |d| d.tick_interval)
                    .max(1);
                (b.liquid_level, b.block_type, interval)
            } else {
                continue;
            };

            if !self.liquid_tick_counter.is_multiple_of(u64::from(interval)) {
                next_active.insert(BlockPos::new(bx, by));
                continue;
            }

            processed_count += 1;
            let mut moved = false;

            // 1. Try Down
            if let Some(down) = self.get_block_ref(bx, by + 1) {
                let d_solid = down.block_type.is_solid();
                let d_level = down.liquid_level;
                if !d_solid && d_level < 8 {
                    let transfer = (8 - d_level).min(level);
                    if transfer > 0 {
                        self.set_liquid_block(bx, by, level - transfer, b_type);
                        self.set_liquid_block(bx, by + 1, d_level + transfer, b_type);

                        Self::activate_neighbors(bx, by, &mut next_active);
                        Self::activate_neighbors(bx, by + 1, &mut next_active);
                        moved = true;
                    }
                }
            }
            if moved {
                continue;
            }

            // 2. Try Horizontal (Equalize among self and neighbors)
            let mut horizontal_cells = vec![(bx, level)];
            let mut total_liquid = i32::from(level);

            for dx in [-1, 1] {
                let nx = bx + dx;
                if let Some(side) = self.get_block_ref(nx, by)
                    && !side.block_type.is_solid()
                {
                    if side.liquid_level > 0 || level > 1 {
                        total_liquid += i32::from(side.liquid_level);
                        horizontal_cells.push((nx, side.liquid_level));
                    }
                }
            }

            if horizontal_cells.len() > 1 {
                if (bx + by + i32::try_from(self.liquid_tick_counter).unwrap_or(0)) % 2 == 0 {
                    horizontal_cells.sort_by_key(|c| c.0);
                } else {
                    horizontal_cells.sort_by_key(|c| -c.0);
                }

                let count = horizontal_cells.len().to_i32().unwrap_or(0);
                let avg = total_liquid / count;
                let mut rem = total_liquid % count;

                for (cx, old_level) in horizontal_cells {
                    let new_level = avg + (if rem > 0 { rem -= 1; 1 } else { 0 });
                    if new_level.to_u8().unwrap_or(0) != old_level {
                        self.set_liquid_block(cx, by, new_level.to_u8().unwrap_or(0), b_type);
                        Self::activate_neighbors(cx, by, &mut next_active);
                        moved = true;
                    }
                }
            }
            if moved {
                continue;
            }

            // 3. Pressure (Upward/U-Pipe) - Process less frequently
            if self.liquid_tick_counter % 10 == 0 {
                visited_scratch.clear();
                let (surface_y, surface_x, surface_level) =
                    self.find_highest_liquid_pos(bx, by, &mut visited_scratch);

                if surface_y < by || (surface_y == by && surface_level > level) {
                    next_active.insert(BlockPos::new(bx, by));
                    
                    let (target_x, target_y, target_level, can_fill) = if level < 8 {
                        let not_trapped = !self.check_trapped_air(bx, by);
                        (bx, by, level, not_trapped)
                    } else if let Some(up) = self.get_block_ref(bx, by - 1) {
                        let is_permeable = !up.block_type.is_solid() && up.liquid_level < 8;
                        let not_trapped = if is_permeable {
                            !self.check_trapped_air(bx, by - 1)
                        } else {
                            true
                        };
                        (bx, by - 1, up.liquid_level, is_permeable && not_trapped)
                    } else {
                        (bx, by - 1, 0, false)
                    };

                    if can_fill && (surface_x != target_x || surface_y != target_y) {
                        let (s_level, s_type) = self
                            .get_block_ref(surface_x, surface_y)
                            .map_or((0, BlockType::Air), |s| (s.liquid_level, s.block_type));

                        if s_level > 0 {
                            self.set_liquid_block(surface_x, surface_y, s_level - 1, s_type);
                            self.set_liquid_block(target_x, target_y, target_level + 1, b_type);
                            Self::activate_neighbors(surface_x, surface_y, &mut next_active);
                            Self::activate_neighbors(target_x, target_y, &mut next_active);
                            moved = true;
                        }
                    }
                }
            }

            if !moved {
                let is_settled = {
                    let d = self.get_block_ref(bx, by + 1).map_or(true, |b| b.block_type.is_solid() || b.liquid_level == 8);
                    let l = self.get_block_ref(bx - 1, by).map_or(true, |b| b.block_type.is_solid() || b.liquid_level >= level);
                    let r = self.get_block_ref(bx + 1, by).map_or(true, |b| b.block_type.is_solid() || b.liquid_level >= level);
                    if level == 8 {
                        let u = self.get_block_ref(bx, by - 1).map_or(true, |b| b.block_type.is_solid() || b.liquid_level == 8);
                        d && l && r && u
                    } else {
                        d && l && r
                    }
                };
                if !is_settled {
                    next_active.insert(BlockPos::new(bx, by));
                }
            }
        }
        self.active_liquids = next_active;
    }

            if !moved {
                let is_settled = {
                    let d = self.get_block_ref(bx, by + 1).map_or(true, |b| b.block_type.is_solid() || b.liquid_level == 8);
                    let l = self.get_block_ref(bx - 1, by).map_or(true, |b| b.block_type.is_solid() || b.liquid_level >= level);
                    let r = self.get_block_ref(bx + 1, by).map_or(true, |b| b.block_type.is_solid() || b.liquid_level >= level);
                    if level == 8 {
                        let u = self.get_block_ref(bx, by - 1).map_or(true, |b| b.block_type.is_solid() || b.liquid_level == 8);
                        d && l && r && u
                    } else {
                        d && l && r
                    }
                };
                if !is_settled {
                    next_active.insert(BlockPos::new(bx, by));
                }
            }
        }
        self.active_liquids = next_active;
    }

    pub(crate) fn activate_neighbors(bx: i32, by: i32, next_active: &mut HashSet<BlockPos>) {
        next_active.insert(BlockPos::new(bx, by));
        next_active.insert(BlockPos::new(bx, by - 1));
        next_active.insert(BlockPos::new(bx, by + 1));
        next_active.insert(BlockPos::new(bx - 1, by));
        next_active.insert(BlockPos::new(bx + 1, by));
    }

    pub(crate) fn set_liquid_block(&mut self, bx: i32, by: i32, level: u8, b_type: BlockType) {
        if let Some(b) = self.get_block_mut(bx, by) {
            b.liquid_level = level;
            if level == 0 {
                b.block_type = BlockType::Air;
                b.is_broken = true;
                b.sprite_rect = None;
            } else {
                b.block_type = b_type;
                b.is_broken = false;
                b.sprite_rect = b_type.get_sprite();
            }
            b.is_modified = true;
        }
    }

    fn find_highest_liquid_pos(
        &self,
        bx: i32,
        by: i32,
        visited: &mut HashSet<BlockPos>,
    ) -> (i32, i32, u8) {
        let self_level = self.get_block_ref(bx, by).map_or(0, |b| b.liquid_level);

        if !visited.insert(BlockPos::new(bx, by)) || visited.len() > 512 {
            return (by, bx, self_level);
        }

        let mut best_pos = (by, bx, self_level);

        // Vertical search (up) is efficient
        let mut curr_y = by - 1;
        while let Some(b) = self.get_block_ref(bx, curr_y) {
            if b.liquid_level > 0 && !b.block_type.is_solid() {
                if curr_y < best_pos.0 || (curr_y == best_pos.0 && b.liquid_level > best_pos.2) {
                    best_pos = (curr_y, bx, b.liquid_level);
                }
                if b.liquid_level < 8 { break; }
                curr_y -= 1;
            } else { break; }
        }

        // Horizontal/Down search
        for (dx, dy) in [(-1, 0), (1, 0), (0, 1)] {
            if let Some(nb) = self.get_block_ref(bx + dx, by + dy) {
                if nb.liquid_level == 8 && !nb.block_type.is_solid() && !visited.contains(&BlockPos::new(bx+dx, by+dy)) {
                    let nb_best = self.find_highest_liquid_pos(bx + dx, by + dy, visited);
                    if nb_best.0 < best_pos.0 || (nb_best.0 == best_pos.0 && nb_best.2 > best_pos.2) {
                        best_pos = nb_best;
                    }
                }
            }
        }
        best_pos
    }

    fn check_trapped_air(&self, bx: i32, by: i32) -> bool {
        let mut visited = HashSet::with_capacity(128);
        let mut queue = std::collections::VecDeque::with_capacity(128);
        queue.push_back((bx, by));
        visited.insert(BlockPos::new(bx, by));

        let limit = 128;

        while let Some((cx, cy)) = queue.pop_front() {
            if visited.len() >= limit || cy < 0 {
                return false; 
            }

            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nx = cx + dx;
                let ny = cy + dy;
                if !visited.contains(&BlockPos::new(nx, ny)) {
                    if let Some(nb) = self.get_block_ref(nx, ny) {
                        if !nb.block_type.is_solid() && nb.liquid_level < 8 {
                            visited.insert(BlockPos::new(nx, ny));
                            queue.push_back((nx, ny));
                        }
                    }
                }
            }
        }
        true
    }
}
