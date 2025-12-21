use super::Game;
use crate::components::{BlockType, Particle};
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::game_renderer::GameRenderer;
use crate::render::sprites::*;
use crate::utils::world_to_chunk_coords;
use ::rand::Rng;
use macroquad::prelude::*;

impl Game {
    pub fn handle_loading(&mut self, game_renderer: &GameRenderer) {
        if let Some(res) = self.persistence_manager.check_load_status() {
            match res {
                Ok(data) => {
                    self.camera.x = data.camera_x;
                    self.camera.y = data.camera_y;
                    self.player_manager.player.x = data.player_x;
                    self.player_manager.player.y = data.player_y;
                    self.player_manager.player.money = data.player_money;
                    self.player_manager.player.fuel = data.player_fuel;
                    self.player_manager.player.max_fuel = data.player_max_fuel;
                    self.player_manager.player.max_cargo = data.player_max_cargo;
                    self.player_manager.player.max_storage = data.player_max_storage;
                    self.player_manager.player.drill_level = data.player_drill_level;
                    self.player_manager.player.tank_level = data.player_tank_level;
                    self.player_manager.player.engine_level = data.player_engine_level;
                    self.player_manager.player.cargo_level = data.player_cargo_level;
                    self.player_manager.player.warp_gates = data.player_warp_gates;

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
                    self.player_manager.player.cargo = cargo;

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
                    self.player_manager.player.storage = storage;

                    // Update camera to match player position immediately
                    self.camera.x = self.player_manager.player.x - SCREEN_WIDTH / 2.0
                        + self.player_manager.player.width / 2.0;
                    self.camera.y = self.player_manager.player.y - SCREEN_HEIGHT / 2.0
                        + self.player_manager.player.height / 2.0;

                    self.world_manager
                        .seed(data.world_seed_main, data.world_seed_ore);

                    self.world_manager
                        .generate_visible_chunks(self.camera.x, self.camera.y);

                    self.world_manager.apply_modifications(data.modified_chunks);

                    self.notification_manager.add_notification(
                        "Loaded!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                    self.on_title_screen = false;
                    self.is_menu_visible = false;
                }
                Err(msg) => {
                    self.notification_manager.add_notification(
                        msg,
                        "error",
                        game_renderer.get_font(),
                    );
                }
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
        let (target_cx, target_cy) = world_to_chunk_coords(world_mx, world_my);
        self.world_manager
            .ensure_chunk_exists_and_generated(target_cx, target_cy);

        let mut should_mark_modified = false;
        if let Some((_, _, _, _, block)) = self
            .world_manager
            .get_block_at_world_coords(world_mx, world_my)
            .filter(|(_, _, _, _, b)| !b.is_broken)
        {
            if block.max_hp == -1 {
                return;
            }
            block.current_hp -= self.player_manager.player.drill_level;
            block.last_damage_time = Some(get_time());

            if block.current_hp <= 0 {
                should_mark_modified = true;
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
        }

        if should_mark_modified {
            if let Some(chunk) = self.world_manager.get_chunk_mut(target_cx, target_cy) {
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
                // Auto-register if not in registry
                if !self
                    .player_manager
                    .player
                    .warp_gates
                    .iter()
                    .any(|w| w.x == block.x && w.y == block.y)
                {
                    self.player_manager
                        .player
                        .warp_gates
                        .push(crate::components::WarpGate {
                            x: block.x,
                            y: block.y,
                            name: block.name.clone().unwrap_or_else(|| "Home".to_string()),
                        });
                }
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
