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

        let active: Vec<BlockPos> = self.active_liquids.iter().copied().collect();
        let mut next_active = HashSet::new();

        let cam_bx = (camera_x / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
        let cam_by = (camera_y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
        let radius = 60;

        for pos in active {
            let (bx, by) = (pos.x, pos.y);
            if (bx - cam_bx).abs() > radius || (by - cam_by).abs() > radius {
                next_active.insert(BlockPos::new(bx, by));
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
                    // Only spread to air if we have enough pressure (level > 1)
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
                let avg = total_liquid / count.to_i32().unwrap_or(0);
                let mut rem = total_liquid % count.to_i32().unwrap_or(0);

                for (cx, old_level) in horizontal_cells {
                    let new_level = avg
                        + (if rem > 0 {
                            rem -= 1;
                            1
                        } else {
                            0
                        });
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

            // 3. Pressure (Upward/U-Pipe)
            // Allow pressure calculation even if not full (to raise own water level)
            if true {
                let (surface_y, surface_x, surface_level) =
                    self.find_highest_liquid_pos(bx, by, &mut HashSet::new());

                // Pressure exists if source is higher, OR same height but more liquid
                if surface_y < by || (surface_y == by && surface_level > level) {
                    // Under pressure: Keep this block active.
                    next_active.insert(BlockPos::new(bx, by));
                    if !moved {
                        Self::activate_neighbors(bx, by, &mut next_active);
                    }

                    // Determine Target: Self (if not full) or Up (if full)
                    // Added: Trapped Air Check. If target is sealed air, don't fill.
                    let (target_x, target_y, target_level, can_fill) = if level < 8 {
                        let not_trapped = !self.check_trapped_air(bx, by);
                        (bx, by, level, not_trapped)
                    } else if let Some(up) = self.get_block_ref(bx, by - 1) {
                        let is_permeable = !up.block_type.is_solid() && up.liquid_level < 8;
                        let not_trapped = if is_permeable {
                            // Check if air is trapped (sealed U-tube)
                            !self.check_trapped_air(bx, by - 1)
                        } else {
                            true
                        };
                        (bx, by - 1, up.liquid_level, is_permeable && not_trapped)
                    } else {
                        (bx, by - 1, 0, false)
                    };

                    if can_fill {
                        // Prevent infinite duplication: Source cannot be the Target.
                        if surface_x == target_x && surface_y == target_y {
                            // Do nothing
                        } else {
                            // "Teleport" flow:
                            let (s_level, s_type) = self
                                .get_block_ref(surface_x, surface_y)
                                .map_or((0, BlockType::Air), |s| (s.liquid_level, s.block_type));

                            if s_level > 0 {
                                // Execute Transfer
                                self.set_liquid_block(surface_x, surface_y, s_level - 1, s_type);
                                self.set_liquid_block(target_x, target_y, target_level + 1, b_type);

                                Self::activate_neighbors(surface_x, surface_y, &mut next_active);
                                Self::activate_neighbors(target_x, target_y, &mut next_active);

                                moved = true;
                            }
                        }
                    }
                }
            }

            if !moved {
                // Check if still unstable
                let is_settled = {
                    let d = self
                        .get_block_ref(bx, by + 1)
                        .map_or(true, |b| b.block_type.is_solid() || b.liquid_level == 8);
                    let l = self
                        .get_block_ref(bx - 1, by)
                        .map_or(true, |b| b.block_type.is_solid() || b.liquid_level >= level);
                    let r = self
                        .get_block_ref(bx + 1, by)
                        .map_or(true, |b| b.block_type.is_solid() || b.liquid_level >= level);
                    if level == 8 {
                        let u = self
                            .get_block_ref(bx, by - 1)
                            .map_or(true, |b| b.block_type.is_solid() || b.liquid_level == 8);
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
        // Initial fallback level. If we can't find current block (shouldn't happen), assume 0.
        let self_level = self.get_block_ref(bx, by).map_or(0, |b| b.liquid_level);

        if !visited.insert(BlockPos::new(bx, by)) || visited.len() > 2048 {
            return (by, bx, self_level);
        }

        let mut best_pos = (by, bx, self_level); // (y, x, level)

        // Check above in the same column
        let mut curr_y = by - 1;
        while let Some(b) = self.get_block_ref(bx, curr_y) {
            visited.insert(BlockPos::new(bx, curr_y));
            if b.liquid_level > 0 && !b.block_type.is_solid() {
                // Better if Higher (smaller Y) OR Same Y but Higher Level
                if curr_y < best_pos.0 || (curr_y == best_pos.0 && b.liquid_level > best_pos.2) {
                    best_pos = (curr_y, bx, b.liquid_level);
                }

                if b.liquid_level < 8 {
                    // Reached surface in this column
                    break;
                }
                curr_y -= 1;
            } else {
                break;
            }
        }

        // Search neighbors (Left, Right, Down)
        let neighbors = [(-1, 0), (1, 0), (0, 1)];
        for (dx, dy) in neighbors {
            if let Some(nb) = self.get_block_ref(bx + dx, by + dy) {
                // Traverse full blocks
                if nb.liquid_level == 8 && !nb.block_type.is_solid() {
                    let nb_best = self.find_highest_liquid_pos(bx + dx, by + dy, visited);

                    // Comparison Logic
                    if nb_best.0 < best_pos.0 || (nb_best.0 == best_pos.0 && nb_best.2 > best_pos.2)
                    {
                        best_pos = nb_best;
                    }
                }
            }
        }
        best_pos
    }

    // Returns TRUE if the air at (bx, by) is trapped (small enclosed pocket).
    fn check_trapped_air(&self, bx: i32, by: i32) -> bool {
        let mut visited = HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((bx, by));
        visited.insert(BlockPos::new(bx, by));

        let limit = 1024; // Max air volume to consider "trapped"

        while let Some((cx, cy)) = queue.pop_front() {
            if visited.len() >= limit {
                return false; // Pocket is big enough, considered open
            }
            if cy < 0 {
                return false; // Open to sky
            }

            // Check neighbors
            let neighbors = [(-1, 0), (1, 0), (0, -1), (0, 1)];
            for (dx, dy) in neighbors {
                let nx = cx + dx;
                let ny = cy + dy;

                if visited.contains(&BlockPos::new(nx, ny)) {
                    continue;
                }

                if let Some(nb) = self.get_block_ref(nx, ny) {
                    if !nb.block_type.is_solid() && nb.liquid_level < 8 {
                        // Air or partial liquid passes
                        visited.insert(BlockPos::new(nx, ny));
                        queue.push_back((nx, ny));
                    }
                    // Solid or Full Liquid blocks recursion
                } else {
                    // Out of bounds (e.g. side of map). Assume sealed? Or open?
                    // Typically map sides are hard boundaries.
                    // But if Y < 0, we handled it.
                }
            }
        }

        true // Exhausted search within limit, so it IS trapped.
    }
}
