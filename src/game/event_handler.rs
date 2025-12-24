use super::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::managers::PersistenceManager;
use crate::render::game_renderer::GameRenderer;
use macroquad::prelude::BLUE;

impl Game {
    pub fn handle_event(&mut self, event: GameEvent, game_renderer: &GameRenderer) {
        match event {
            GameEvent::OpenSaveSelection => {
                self.save_files = PersistenceManager::list_save_files();
                self.on_title_screen = false;
                if self.save_files.is_empty() {
                    self.on_new_game_input_screen = true;
                    self.input_buffer.clear();
                } else {
                    self.on_save_select_screen = true;
                }
            }
            GameEvent::LoadSave(filename) => {
                self.current_save_name = filename.clone();
                self.persistence_manager.load_game(filename);
                self.on_save_select_screen = false;
            }
            GameEvent::StartNewGameSetup => {
                self.on_save_select_screen = false;
                self.on_new_game_input_screen = true;
                self.input_buffer.clear();
            }
            GameEvent::ConfirmNewGame(name) => {
                let mut filename = name.clone();
                if !filename.ends_with(".json") {
                    filename.push_str(".json");
                }
                self.current_save_name = filename;
                self.on_new_game_input_screen = false;
                self.input_buffer.clear();
                self.reset_player_state();
                self.world_manager.seed(::rand::random(), ::rand::random());
                self.world_manager.generate_visible_chunks(0.0, 0.0);
                
                // Add Initial Warp Gate Registry
                let player_start_x_block = (PLAYER_INITIAL_X / BLOCK_SIZE).floor() * BLOCK_SIZE;
                let player_start_y_block = (PLAYER_INITIAL_Y / BLOCK_SIZE).floor() * BLOCK_SIZE;
                self.player_manager.player.warp_gates.push(crate::components::WarpGate {
                    x: player_start_x_block,
                    y: player_start_y_block,
                    name: "Home".to_string(),
                });

                self.notification_manager.add_notification(
                    "New Game!".to_string(),
                    "success",
                    game_renderer.get_font(),
                );
            }
            GameEvent::SaveGame => {
                self.persistence_manager
                    .save_game(self.current_save_name.clone(), self.make_save_data());
            }
            GameEvent::QuitGame => {
                std::process::exit(0);
            }
            GameEvent::ReturnToTitle => {
                self.return_to_title_screen(game_renderer);
            }
            GameEvent::ReturnToTitleFromSaveSelect => {
                self.return_to_title_from_save_select();
            }
            GameEvent::UpgradeDrill => {
                let cost = self.player_manager.player.drill_level * 100;
                if self.player_manager.player.money >= cost {
                    self.player_manager.player.money -= cost;
                    self.player_manager.player.drill_level += 1;
                    self.notification_manager.add_notification(
                        "Drill Upgraded!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                } else {
                    self.notification_manager.add_notification(
                        "Not enough money!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::UpgradeTank => {
                let cost = self.player_manager.player.tank_level * 80;
                if self.player_manager.player.money >= cost {
                    self.player_manager.player.money -= cost;
                    self.player_manager.player.tank_level += 1;
                    self.player_manager.player.max_fuel += 50.0;
                    self.player_manager.player.fuel = self.player_manager.player.max_fuel;
                    self.notification_manager.add_notification(
                        "Tank Upgraded!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                } else {
                    self.notification_manager.add_notification(
                        "Not enough money!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::UpgradeEngine => {
                let cost = self.player_manager.player.engine_level * 120;
                if self.player_manager.player.money >= cost {
                    self.player_manager.player.money -= cost;
                    self.player_manager.player.engine_level += 1;
                    self.notification_manager.add_notification(
                        "Engine Upgraded!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                } else {
                    self.notification_manager.add_notification(
                        "Not enough money!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::UpgradeCargo => {
                let cost = self.player_manager.player.cargo_level * 150;
                if self.player_manager.player.money >= cost {
                    self.player_manager.player.money -= cost;
                    self.player_manager.player.cargo_level += 1;
                    self.player_manager.player.max_cargo += 250;
                    self.notification_manager.add_notification(
                        "Cargo Upgraded!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                } else {
                    self.notification_manager.add_notification(
                        "Not enough money!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::BuyWarpGate => {
                if self.player_manager.player.money >= 500 {
                    if self.player_manager.player.cargo.len() < self.player_manager.player.max_cargo as usize {
                        self.player_manager.player.money -= 500;
                         self.player_manager.player.cargo.push(crate::components::OwnedItem {
                            item_type: "WarpGate".to_string(),
                            is_natural: false,
                            is_auto_stored: false,
                        });
                        self.notification_manager.add_notification(
                            "Warp Gate Purchased!".to_string(),
                            "success",
                            game_renderer.get_font(),
                        );
                    } else {
                         self.notification_manager.add_notification(
                            "Cargo Full!".to_string(),
                            "error",
                            game_renderer.get_font(),
                        );
                    }
                } else {
                    self.notification_manager.add_notification(
                        "Not enough money!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::StartPlaceWarpGate => {
                self.on_warp_place_screen = true;
                self.input_buffer.clear();
                self.is_menu_visible = false;
            }
            GameEvent::ConfirmWarpGateName(name) => {
                // Item removal happens in handle_block_placement or we can do it here if we passed the index.
                // For now, let's assume the item was removed when we started placement or will be removed.
                // Actually, the previous logic removed from `inventory_warp_gates`. 
                // We should remove the "WarpGate" item from cargo here.
                
                if let Some(pos) = self.player_manager.player.cargo.iter().position(|it| it.item_type == "WarpGate") {
                     self.player_manager.player.cargo.remove(pos);
                }

                // Get placement coordinates
                let (wx, wy) = if let Some(target) = self.warp_placement_target {
                    target
                } else {
                     // Fallback to player pos aligned
                     ((self.player_manager.player.x / BLOCK_SIZE).round() * BLOCK_SIZE,
                      (self.player_manager.player.y / BLOCK_SIZE).round() * BLOCK_SIZE)
                };

                self.player_manager
                    .player
                    .warp_gates
                    .push(crate::components::WarpGate {
                        x: wx,
                        y: wy,
                        name: name.clone(),
                    });
                
                // We also need to set the block in the world!
                // Access world_manager.
                 if let Some((_, _, _, _, block)) = self.world_manager.get_block_at_world_coords(wx, wy) {
                     block.block_type = crate::components::BlockType::WarpGate;
                     block.sprite_rect = Some(crate::render::sprites::SPRITE_BLOCK_WARPGATE);
                     block.max_hp = 50;
                     block.current_hp = 50;
                     block.is_broken = false;
                     block.is_modified = true;
                     block.name = Some(name);
                 }
                 self.warp_placement_target = None;

                self.on_warp_place_screen = false;
                self.input_buffer.clear();
                self.notification_manager.add_notification(
                    "Warp Gate Placed!".to_string(),
                    "success",
                    game_renderer.get_font(),
                );
            }
            GameEvent::OpenWarpMenu => {
                self.on_warp_select_screen = true;
                self.is_shop_open = false;
            }
            GameEvent::TeleportToWarp(idx) => {
                if let Some(gate) = self.player_manager.player.warp_gates.get(idx) {
                    self.player_manager.player.x = gate.x;
                    self.player_manager.player.y = gate.y;
                    self.player_manager.player.vx = 0.0;
                    self.player_manager.player.vy = 0.0;
                    self.on_warp_select_screen = false;
                    self.notification_manager.add_notification(
                        format!("Warped to {}!", gate.name),
                        "success",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::OpenShop => {
                self.is_shop_open = true;
            }
            GameEvent::OpenWarehouse => {
                self.is_warehouse_open = true;
            }
            GameEvent::SellItem(item_type, quantity) => {
                let price = match item_type.as_str() {
                    "Coal" => 10,
                    "Stone" => 2,
                    "Dirt" => 1,
                    _ => 0,
                };
                let mut sold = 0;
                while sold < quantity {
                    if let Some(pos) = self
                        .player_manager
                        .player
                        .storage
                        .iter()
                        .position(|it| it.item_type == item_type)
                    {
                        self.player_manager.player.storage.remove(pos);
                        self.player_manager.player.money += price;
                        sold += 1;
                    } else if let Some(pos) = self
                        .player_manager
                        .player
                        .cargo
                        .iter()
                        .position(|it| it.item_type == item_type)
                    {
                        self.player_manager.player.cargo.remove(pos);
                        self.player_manager.player.money += price;
                        sold += 1;
                    } else {
                        break;
                    }
                }
            }

            GameEvent::DepositItem(item_type, quantity) => {
                let mut moved = 0;
                while moved < quantity
                    && self.player_manager.player.storage.len()
                        < self.player_manager.player.max_storage as usize
                {
                    if let Some(pos) = self
                        .player_manager
                        .player
                        .cargo
                        .iter()
                        .position(|it| it.item_type == item_type)
                    {
                        let item = self.player_manager.player.cargo.remove(pos);
                        self.player_manager.player.storage.push(item);
                        moved += 1;
                    } else {
                        break;
                    }
                }
                if moved < quantity
                    && self.player_manager.player.storage.len()
                        >= self.player_manager.player.max_storage as usize
                {
                    self.notification_manager.add_notification(
                        "Storage Full!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }
            GameEvent::WithdrawItem(item_type, quantity) => {
                let weight = crate::utils::get_item_weight(&item_type);
                let mut moved = 0;
                while moved < quantity
                    && self.player_manager.player.total_cargo_weight() + weight
                        <= self.player_manager.player.max_cargo
                {
                    if let Some(pos) = self
                        .player_manager
                        .player
                        .storage
                        .iter()
                        .position(|it| it.item_type == item_type)
                    {
                        let mut item = self.player_manager.player.storage.remove(pos);
                        item.is_auto_stored = false;
                        self.player_manager.player.cargo.push(item);
                        moved += 1;
                    } else {
                        break;
                    }
                }

                if moved < quantity
                    && self.player_manager.player.total_cargo_weight() + weight
                        > self.player_manager.player.max_cargo
                {
                    self.notification_manager.add_notification(
                        "Cargo Full!".to_string(),
                        "error",
                        game_renderer.get_font(),
                    );
                }
            }

            GameEvent::CloseMenu => {
                self.is_menu_visible = false;
                self.is_shop_open = false;
                self.is_inventory_open = false;
                self.is_warehouse_open = false;
                self.on_warp_select_screen = false;
                self.on_warp_place_screen = false;
            }

            GameEvent::Respawn => {
                // Clear cargo
                self.player_manager.player.cargo.clear();

                // Money penalty (10%)
                let penalty = (self.player_manager.player.money as f32 * 0.1) as i32;
                self.player_manager.player.money -= penalty;

                // Reset position and state
                self.player_manager.player.x = PLAYER_INITIAL_X;
                self.player_manager.player.y = PLAYER_INITIAL_Y;
                self.player_manager.player.vx = 0.0;
                self.player_manager.player.vy = 0.0;
                self.player_manager.player.fuel = self.player_manager.player.max_fuel;

                // Reset camera
                self.camera.x = PLAYER_INITIAL_X - SCREEN_WIDTH / 2.0;
                self.camera.y = PLAYER_INITIAL_Y - SCREEN_HEIGHT / 2.0;
                self.camera.old_x = self.camera.x;
                self.camera.old_y = self.camera.y;

                // Add some particles for visual effect
                for _ in 0..30 {
                    self.particle_manager
                        .add_particles(vec![crate::components::Particle::new(
                            self.player_manager.player.x,
                            self.player_manager.player.y,
                            BLUE,
                        )]);
                }

                self.is_menu_visible = false;

                self.notification_manager.add_notification(
                    self.lang_manager.get_string("notification.respawn.success"),
                    "success",
                    game_renderer.get_font(),
                );
            }
            GameEvent::SetSelectedItemIndex(idx) => {
                self.selected_item_index = idx;
                self.is_inventory_open = false;
            }
        }
    }
}
