use super::Game;
use crate::managers::persistence::BlockSaveData;
use crate::render::sprites::*;

impl Game {
    pub fn make_save_data(&self) -> serde_json::Value {
        let mut modified_chunks_data: Vec<serde_json::Value> = Vec::new();

        for (&(cx, cy), chunk) in self.world_manager.chunks.iter() {
            if chunk.is_modified_in_session {
                let mut modified_blocks: Vec<BlockSaveData> = Vec::new();
                for row in &chunk.blocks {
                    for block in row {
                        if block.is_modified {
                            let id = if block.sprite_rect == Some(SPRITE_BLOCK_DIRT) {
                                "dirt"
                            } else if block.sprite_rect == Some(SPRITE_BLOCK_GRASS) {
                                "grass"
                            } else if block.sprite_rect == Some(SPRITE_BLOCK_STONE) {
                                "stone"
                            } else if block.sprite_rect == Some(SPRITE_BLOCK_COAL) {
                                "coal"
                            } else {
                                "unknown"
                            };
                            modified_blocks.push(BlockSaveData {
                                x: block.x,
                                y: block.y,
                                current_hp: block.current_hp,
                                sprite_id: id.to_string(),
                            });
                        }
                    }
                }
                if !modified_blocks.is_empty() {
                    modified_chunks_data.push(serde_json::json!({
                        "cx": cx,
                        "cy": cy,
                        "modified_blocks": modified_blocks,
                    }));
                }
            }
        }

        serde_json::json!({
            "camera_x": self.camera.x,
            "camera_y": self.camera.y,
            "player_x": self.player_manager.player.x,
            "player_y": self.player_manager.player.y,
            "player_money": self.player_manager.player.money,
            "player_fuel": self.player_manager.player.fuel,
            "player_max_fuel": self.player_manager.player.max_fuel,
            "player_cargo": self.player_manager.player.cargo,
            "player_max_cargo": self.player_manager.player.max_cargo,
            "player_storage": self.player_manager.player.storage,
            "player_max_storage": self.player_manager.player.max_storage,
            "player_drill_level": self.player_manager.player.drill_level,
            "player_tank_level": self.player_manager.player.tank_level,
            "player_engine_level": self.player_manager.player.engine_level,
            "player_cargo_level": self.player_manager.player.cargo_level,
            "world_seed_main": self.world_manager.world_seed_main,
            "world_seed_ore": self.world_manager.world_seed_ore,
            "modified_chunks": modified_chunks_data
        })
    }
}
