use super::Game;
use crate::components::{BlockType, Particle};
use crate::constants::*;
use crate::events::GameEvent; // Added import
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

                    if let Some(serde_json::Value::Array(arr)) = map.get("player_warp_gates") {
                        self.player_manager.player.warp_gates = arr
                            .iter()
                            .filter_map(|v| serde_json::from_value(v.clone()).ok())
                            .collect();
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

                    // Rebuild warp_gates registry from chunks?
                    // Ideally we should save `warp_gates` in the save file explicitly.
                    // For now, let's just clear it and hope it's consistent or not needed to rebuild here
                    // because we are loading player state? No, warp_gates is in player struct, but if it's not in JSON...
                    // The original code didn't load warp_gates from JSON. It needs to.
                    // But I didn't touch save/load logic. Let's assume the user doesn't need persistence for warp gates RIGHT NOW or the existing system handles it if it was using Serde on Player.
                    // Yes, Player derives Serialize/Deserialize, so `warp_gates` vec should be loaded automatically!
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

        self.world_manager.update();

        let mx = (mouse_position().0 / screen_width()) * SCREEN_WIDTH;
        let my = (mouse_position().1 / screen_height()) * SCREEN_HEIGHT;
        let world_mx = (mx + self.camera.x).round();
        let world_my = (my + self.camera.y).round();

        let hovered_block_coords = self
            .world_manager
            .get_block_at_world_coords(world_mx, world_my)
            .map(|(_, _, _, _, block)| (block.x, block.y));

        let mut preview_sprite = None;
        let mut is_valid = true;

        let mut current_item_type = None;
        if self.selected_item_index < self.player_manager.player.cargo.len() {
            current_item_type = Some(
                self.player_manager.player.cargo[self.selected_item_index]
                    .item_type
                    .clone(),
            );
        }

        if let Some(item_type) = &current_item_type {
            let bt = BlockType::from_item_type(item_type);
            let potential_sprite = match bt {
                Some(BlockType::Dirt) => Some(SPRITE_BLOCK_DIRT),
                Some(BlockType::Stone) => Some(SPRITE_BLOCK_STONE),
                Some(BlockType::Coal) => Some(SPRITE_BLOCK_COAL),
                Some(BlockType::Grass) => Some(SPRITE_BLOCK_GRASS),
                Some(BlockType::WarpGate) => Some(SPRITE_BLOCK_WARPGATE),
                _ => None,
            };

            if let Some(sprite) = potential_sprite {
                if let Some((_, _, _, _, block)) = self
                    .world_manager
                    .get_block_at_world_coords(world_mx, world_my)
                {
                    if let Some(BlockType::WarpGate) = bt {
                        // Warp Gates can be placed in Air (non-solid)
                        // But we also don't want to place it if it's already a Warp Gate or Solid?
                        // Prompt: "Initial spawn point... Non-solid block concept"
                        // If I place it, it replaces the block or fills air.
                        if block.block_type != BlockType::Air {
                            is_valid = false;
                            preview_sprite = None;
                        } else {
                            preview_sprite = Some(sprite);
                            // WarpGate is non-solid, so we don't check overlap with player?
                            // Or maybe we do to prevent getting stuck if it becomes solid later?
                            // Prompt implies "Non-solid", so overlap is allowed.
                        }
                    } else {
                        // Standard blocks
                        if !block.is_broken {
                            is_valid = false;
                            preview_sprite = None;
                        } else {
                            preview_sprite = Some(sprite);
                            let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                            let player_rect = self.player_manager.player.rect();
                            if block_rect.overlaps(&player_rect) {
                                is_valid = false;
                            }
                        }
                    }
                } else {
                    is_valid = false;
                    preview_sprite = None;
                }
            }
        }

        self.select_block
            .update(hovered_block_coords, preview_sprite, is_valid);

        if self.is_mouse_button_pressed_buffered(MouseButton::Left) {
            self.handle_block_interaction(world_mx, world_my, game_renderer);
        }
        if self.is_mouse_button_pressed_buffered(MouseButton::Right) {
            self.handle_right_click(world_mx, world_my, game_renderer);
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
            if block.max_hp == -1 {
                return;
            }
            if block.current_hp == block.max_hp {
                block.is_modified = true;
            }
            block.current_hp -= self.player_manager.player.drill_level;
            block.last_damage_time = Some(get_time());

            if block.current_hp <= 0 {
                // Special handling for WarpGate destruction
                if block.block_type == BlockType::WarpGate {
                    // Remove from registry
                    if let Some(pos) = self
                        .player_manager
                        .player
                        .warp_gates
                        .iter()
                        .position(|w| w.x == block.x && w.y == block.y)
                    {
                        self.player_manager.player.warp_gates.remove(pos);
                        self.notification_manager.add_notification(
                            "Warp Gate Destroyed!".to_string(),
                            "info",
                            game_renderer.get_font(),
                        );
                    }
                }

                block.current_hp = 0;
                block.is_broken = true;
                block.is_modified = true;
                block.block_type = BlockType::Air; // Reset type to Air
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

    pub fn handle_right_click(
        &mut self,
        world_mx: f32,
        world_my: f32,
        game_renderer: &GameRenderer,
    ) {
        let (cx, cy) = world_to_chunk_coords(world_mx, world_my);
        self.world_manager.ensure_chunk_exists_and_generated(cx, cy);

        // 1. Check interaction with existing functional blocks
        if let Some((_, _, _, _, block)) = self
            .world_manager
            .get_block_at_world_coords(world_mx, world_my)
        {
            if !block.is_broken && block.block_type == BlockType::WarpGate {
                self.handle_event(GameEvent::OpenWarpMenu, game_renderer);
                return; // Interaction consumes the click
            }
        }

        // 2. Check Item Placement
        if self.selected_item_index >= self.player_manager.player.cargo.len() {
            return;
        }

        let it_type = self.player_manager.player.cargo[self.selected_item_index]
            .item_type
            .clone();

        let block_type_to_place = BlockType::from_item_type(&it_type);
        let is_placeable = block_type_to_place
            .as_ref()
            .map_or(false, |bt| bt.is_placeable());

        if !is_placeable {
            return;
        }

        if let Some((cx, cy, _, _, block)) = self
            .world_manager
            .get_block_at_world_coords(world_mx, world_my)
        {
            // For standard blocks, target must be broken.
            // For WarpGate, target must be Air (which is broken/empty).
            if block.block_type == BlockType::Air || block.is_broken {
                // Check collision with player if block is solid
                // WarpGate is non-solid, others are solid.
                let will_be_solid = block_type_to_place
                    .as_ref()
                    .map_or(false, |bt| bt.is_solid());

                let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                let player_rect = self.player_manager.player.rect();

                if !will_be_solid || !block_rect.overlaps(&player_rect) {
                    if let Some(BlockType::WarpGate) = block_type_to_place {
                        // Trigger Warp Gate Name Input
                        self.warp_placement_target = Some((block.x, block.y));
                        self.handle_event(GameEvent::StartPlaceWarpGate, game_renderer);
                        // Actual placement happens in ConfirmWarpGateName
                        return;
                    }

                    // Place standard block
                    block.is_broken = false;
                    block.is_modified = true;

                    let (hp, sprite, b_type) = match block_type_to_place {
                        Some(BlockType::Dirt) => {
                            (HARDNESS_DIRT, SPRITE_BLOCK_DIRT, BlockType::Dirt)
                        }
                        Some(BlockType::Stone) => {
                            (HARDNESS_STONE, SPRITE_BLOCK_STONE, BlockType::Stone)
                        }
                        Some(BlockType::Coal) => {
                            (HARDNESS_COAL, SPRITE_BLOCK_COAL, BlockType::Coal)
                        }
                        Some(BlockType::Grass) => {
                            (HARDNESS_GRASS, SPRITE_BLOCK_GRASS, BlockType::Grass)
                        }
                        _ => (HARDNESS_DIRT, SPRITE_BLOCK_DIRT, BlockType::Dirt),
                    };

                    // Apply depth multiplier for HP
                    let y_block = (block.y / BLOCK_SIZE).floor() as i32;
                    let depth = (y_block - SURFACE_Y_LEVEL).max(0) as f64;
                    let multiplier = 1.0 + depth * HARDNESS_DEPTH_MULTIPLIER;
                    block.max_hp = (hp as f64 * multiplier).floor() as i32;
                    block.current_hp = block.max_hp;
                    block.sprite_rect = Some(sprite);
                    block.block_type = b_type;

                    // Visual feedback: particles
                    let count = 5;
                    let particles: Vec<Particle> = (0..count)
                        .map(|_| {
                            let particle_color = block
                                .sprite_rect
                                .map_or(WHITE, |rect| game_renderer.get_random_pixel_color(rect));
                            Particle::new(block.x, block.y, particle_color)
                        })
                        .collect();
                    self.particle_manager.add_particles(particles);

                    if let Some(chunk) = self.world_manager.get_chunk_mut(cx, cy) {
                        chunk.is_modified_in_session = true;
                    }

                    // Remove item from cargo
                    self.player_manager
                        .player
                        .cargo
                        .remove(self.selected_item_index);
                }
            }
        }
    }
}
