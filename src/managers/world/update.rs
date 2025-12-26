use super::WorldManager;
use macroquad::prelude::*;

impl WorldManager {
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
