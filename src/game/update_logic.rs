use super::Game;
use crate::components::Particle;
use crate::constants::*;
use crate::render::game_renderer::GameRenderer;
use crate::render::sprites::*;
use crate::utils::world_to_chunk_coords;
use ::rand::Rng;
use macroquad::prelude::*;

impl Game {
    pub fn handle_loading(&mut self, game_renderer: &GameRenderer) {
        if let Some((success, data)) = self.persistence_manager.check_load_status() {
            if success {
                if let serde_json::Value::Object(map) = data {
                    if let Some(cx) = map.get("camera_x").and_then(|v| v.as_f64()) {
                        self.camera.x = cx as f32;
                    }
                    if let Some(cy) = map.get("camera_y").and_then(|v| v.as_f64()) {
                        self.camera.y = cy as f32;
                    }
                    if let Some(px) = map.get("player_x").and_then(|v| v.as_f64()) {
                        self.player_manager.player.x = px as f32;
                    }
                    if let Some(py) = map.get("player_y").and_then(|v| v.as_f64()) {
                        self.player_manager.player.y = py as f32;
                    }

                    if let Some(v) = map.get("player_money").and_then(|v| v.as_i64()) {
                        self.player_manager.player.money = v as i32;
                    }
                    if let Some(v) = map.get("player_fuel").and_then(|v| v.as_f64()) {
                        self.player_manager.player.fuel = v as f32;
                    }
                    if let Some(v) = map.get("player_max_fuel").and_then(|v| v.as_f64()) {
                        self.player_manager.player.max_fuel = v as f32;
                    }
                    if let Some(v) = map.get("player_max_cargo").and_then(|v| v.as_i64()) {
                        self.player_manager.player.max_cargo = v as i32;
                    }
                    if let Some(v) = map.get("player_max_storage").and_then(|v| v.as_i64()) {
                        self.player_manager.player.max_storage = v as i32;
                    }
                    if let Some(v) = map.get("player_drill_level").and_then(|v| v.as_i64()) {
                        self.player_manager.player.drill_level = v as i32;
                    }
                    if let Some(v) = map.get("player_tank_level").and_then(|v| v.as_i64()) {
                        self.player_manager.player.tank_level = v as i32;
                    }
                    if let Some(v) = map.get("player_engine_level").and_then(|v| v.as_i64()) {
                        self.player_manager.player.engine_level = v as i32;
                    }
                    if let Some(v) = map.get("player_cargo_level").and_then(|v| v.as_i64()) {
                        self.player_manager.player.cargo_level = v as i32;
                    }

                    if let Some(serde_json::Value::Array(arr)) = map.get("player_cargo") {
                        self.player_manager.player.cargo = arr
                            .iter()
                            .filter_map(|v| {
                                if let Some(s) = v.as_str() {
                                    Some(crate::components::OwnedItem {
                                        item_type: s.to_string(),
                                        is_natural: true,
                                        is_auto_stored: true,
                                    })
                                } else if let Some(obj) = v.as_object() {
                                    let it = obj.get("item_type")?.as_str()?.to_string();
                                    let nat = obj.get("is_natural")?.as_bool()?;
                                    let auto = obj
                                        .get("is_auto_stored")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(nat);
                                    Some(crate::components::OwnedItem {
                                        item_type: it,
                                        is_natural: nat,
                                        is_auto_stored: auto,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                    if let Some(serde_json::Value::Array(arr)) = map.get("player_storage") {
                        self.player_manager.player.storage = arr
                            .iter()
                            .filter_map(|v| {
                                if let Some(s) = v.as_str() {
                                    Some(crate::components::OwnedItem {
                                        item_type: s.to_string(),
                                        is_natural: true,
                                        is_auto_stored: true,
                                    })
                                } else if let Some(obj) = v.as_object() {
                                    let it = obj.get("item_type")?.as_str()?.to_string();
                                    let nat = obj.get("is_natural")?.as_bool()?;
                                    let auto = obj
                                        .get("is_auto_stored")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(nat);
                                    Some(crate::components::OwnedItem {
                                        item_type: it,
                                        is_natural: nat,
                                        is_auto_stored: auto,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }

                    // Update camera to match player position immediately
                    self.camera.x = self.player_manager.player.x - SCREEN_WIDTH / 2.0
                        + self.player_manager.player.width / 2.0;
                    self.camera.y = self.player_manager.player.y - SCREEN_HEIGHT / 2.0
                        + self.player_manager.player.height / 2.0;

                    let wm_main = map
                        .get("world_seed_main")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    let wm_ore = map
                        .get("world_seed_ore")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32);
                    if let (Some(m), Some(o)) = (wm_main, wm_ore) {
                        self.world_manager.seed(m, o);
                    }

                    self.world_manager
                        .generate_visible_chunks(self.camera.x, self.camera.y);

                    if let Some(serde_json::Value::Array(mods)) = map.get("modified_chunks") {
                        self.world_manager.apply_modifications(mods.clone());
                    }
                }

                self.notification_manager.add_notification(
                    "Loaded!".to_string(),
                    "success",
                    game_renderer.get_font(),
                );
                self.on_title_screen = false;
                self.is_menu_visible = false;
            } else if let serde_json::Value::String(msg) = data {
                self.notification_manager
                    .add_notification(msg, "error", game_renderer.get_font());
            }
        }
    }

    pub fn handle_gameplay_update(&mut self, game_renderer: &GameRenderer) {
        if self.is_key_pressed_buffered(KeyCode::Escape) {
            if self.is_shop_open {
                self.is_shop_open = false;
            } else if self.is_inventory_open {
                self.is_inventory_open = false;
            } else if self.is_warehouse_open {
                self.is_warehouse_open = false;
            } else if self.on_warp_select_screen {
                self.on_warp_select_screen = false;
            } else if self.on_warp_place_screen {
                self.on_warp_place_screen = false;
            } else {
                self.is_menu_visible = true;
            }
        }

        if self.is_key_pressed_buffered(KeyCode::I) || self.is_key_pressed_buffered(KeyCode::Tab) {
            if !self.is_menu_visible
                && !self.is_shop_open
                && !self.on_warp_place_screen
                && !self.on_warp_select_screen
            {
                self.is_inventory_open = !self.is_inventory_open;
            }
        }

        if !self.is_menu_visible
            && !self.is_shop_open
            && !self.is_inventory_open
            && !self.is_warehouse_open
            && !self.on_warp_place_screen
            && !self.on_warp_select_screen
        {
            self.player_manager.update(&mut self.world_manager);
        }

        self.camera.x = self.player_manager.player.x - SCREEN_WIDTH / 2.0
            + self.player_manager.player.width / 2.0;
        self.camera.y = self.player_manager.player.y - SCREEN_HEIGHT / 2.0
            + self.player_manager.player.height / 2.0;

        self.on_surface =
            self.player_manager.player.y < (SURFACE_Y_LEVEL as f32 * BLOCK_SIZE) + 8.0;

        self.world_manager
            .generate_visible_chunks(self.camera.x, self.camera.y);

        let mx = (mouse_position().0 / screen_width()) * SCREEN_WIDTH;
        let my = (mouse_position().1 / screen_height()) * SCREEN_HEIGHT;
        let world_mx = (mx + self.camera.x).round();
        let world_my = (my + self.camera.y).round();

        let hovered_block_coords = self
            .world_manager
            .get_block_at_world_coords(world_mx, world_my)
            .and_then(|(_, _, _, _, block)| {
                if !block.is_broken {
                    Some((block.x, block.y))
                } else {
                    None
                }
            });
        self.select_block.update(hovered_block_coords);

        if self.is_mouse_button_pressed_buffered(MouseButton::Left) {
            self.handle_block_interaction(world_mx, world_my, game_renderer);
        }

        let blocks = self
            .world_manager
            .get_active_blocks_in_view(self.camera.x, self.camera.y);
        self.particle_manager.update(&blocks, &self.camera);
        self.item_manager
            .update(&mut self.player_manager.player, &blocks);
    }

    pub fn handle_block_interaction(
        &mut self,
        world_mx: f32,
        world_my: f32,
        game_renderer: &GameRenderer,
    ) {
        let (cx, cy) = world_to_chunk_coords(world_mx, world_my);
        self.world_manager.ensure_chunk_exists_and_generated(cx, cy);

        if let Some((cx, cy, _, _, block)) = self
            .world_manager
            .get_block_at_world_coords(world_mx, world_my)
            .filter(|(_, _, _, _, b)| !b.is_broken)
        {
            if block.current_hp == block.max_hp {
                block.is_modified = true;
            }
            block.current_hp -= self.player_manager.player.drill_level;

            if block.current_hp <= 0 {
                block.current_hp = 0;
                block.is_broken = true;
                block.is_modified = true;
                let count = ::rand::rng().random_range(5..15);
                let particles: Vec<Particle> = (0..count)
                    .map(|_| {
                        let particle_color = block
                            .sprite_rect
                            .map_or(WHITE, |rect| game_renderer.get_random_pixel_color(rect));
                        Particle::new(block.x, block.y, particle_color)
                    })
                    .collect();
                self.particle_manager.add_particles(particles);

                if let Some(rect) = block.sprite_rect {
                    let item_type = if rect == SPRITE_BLOCK_COAL {
                        Some("Coal".to_string())
                    } else if rect == SPRITE_BLOCK_STONE {
                        Some("Stone".to_string())
                    } else if rect == SPRITE_BLOCK_DIRT {
                        Some("Dirt".to_string())
                    } else {
                        None
                    };
                    if let Some(it) = item_type {
                        // Item is 4x4, Block is 8x8.
                        // Center is (block.x + 4, block.y + 4), so top-left of item should be (block.x + 2, block.y + 2)
                        self.item_manager
                            .spawn_item(block.x + 2.0, block.y + 2.0, it, rect, true);
                    }
                }
            }
            if let Some(chunk) = self.world_manager.get_chunk_mut(cx, cy) {
                chunk.is_modified_in_session = true;
            }
        }
    }
}
