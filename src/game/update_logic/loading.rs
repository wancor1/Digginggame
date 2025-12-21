use crate::Game;
use crate::constants::*;
use crate::render::game_renderer::GameRenderer;

pub fn handle_loading(game: &mut Game, game_renderer: &GameRenderer) {
    if let Some(res) = game.persistence_manager.check_load_status() {
        match res {
            Ok(data) => {
                game.camera.x = data.camera_x;
                game.camera.y = data.camera_y;
                game.player_manager.player.x = data.player_x;
                game.player_manager.player.y = data.player_y;
                game.player_manager.player.money = data.player_money;
                game.player_manager.player.fuel = data.player_fuel;
                game.player_manager.player.max_fuel = data.player_max_fuel;
                game.player_manager.player.max_cargo = data.player_max_cargo;
                game.player_manager.player.max_storage = data.player_max_storage;
                game.player_manager.player.drill_level = data.player_drill_level;
                game.player_manager.player.tank_level = data.player_tank_level;
                game.player_manager.player.engine_level = data.player_engine_level;
                game.player_manager.player.cargo_level = data.player_cargo_level;
                game.player_manager.player.warp_gates = data.player_warp_gates;

                // Expand stacked items
                let mut cargo = Vec::new();
                for stack in data.player_cargo {
                    for _ in 0..stack.count {
                        cargo.push(crate::components::OwnedItem {
                            item_type: stack.item_type.clone(),
                            is_natural: stack.is_natural,
                            is_auto_stored: stack.is_auto_stored,
                        });
                    }
                }
                game.player_manager.player.cargo = cargo;

                let mut storage = Vec::new();
                for stack in data.player_storage {
                    for _ in 0..stack.count {
                        storage.push(crate::components::OwnedItem {
                            item_type: stack.item_type.clone(),
                            is_natural: stack.is_natural,
                            is_auto_stored: stack.is_auto_stored,
                        });
                    }
                }
                game.player_manager.player.storage = storage;

                // Update camera to match player position immediately
                game.camera.x = game.player_manager.player.x - SCREEN_WIDTH / 2.0
                    + game.player_manager.player.width / 2.0;
                game.camera.y = game.player_manager.player.y - SCREEN_HEIGHT / 2.0
                    + game.player_manager.player.height / 2.0;

                game.world_manager
                    .seed(data.world_seed_main, data.world_seed_ore);

                game.world_manager
                    .generate_visible_chunks(game.camera.x, game.camera.y);

                game.world_manager.apply_modifications(data.modified_chunks);

                game.notification_manager.add_notification(
                    "Loaded!".to_string(),
                    "success",
                    game_renderer.get_font(),
                );
                game.on_title_screen = false;
                game.is_menu_visible = false;
            }
            Err(msg) => {
                game.notification_manager
                    .add_notification(msg, "error", game_renderer.get_font());
            }
        }
    }
}
