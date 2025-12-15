use crate::components::Particle;
use crate::managers::persistence::BlockSaveData;
use ::rand::Rng;
use macroquad::prelude::*;

mod components;
mod constants;
mod events;
mod managers;
mod render;
mod ui;
mod utils;

use crate::components::Camera;
use constants::*;
use events::{CameraMoveIntent, GameEvent};
use managers::*;
use render::game_renderer::GameRenderer;
use ui::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Digging Game".to_owned(),
        window_width: SCREEN_WIDTH as i32 * 4, // Scale up for visibility
        window_height: SCREEN_HEIGHT as i32 * 4,
        ..Default::default()
    }
}

pub struct Game {
    world_manager: WorldManager,
    particle_manager: ParticleManager,
    persistence_manager: PersistenceManager,
    lang_manager: LanguageManager,
    notification_manager: NotificationManager,
    input_handler: InputHandler,
    select_block: SelectBlock,
    pub camera: Camera,

    // UI State
    on_title_screen: bool,
    is_menu_visible: bool,
    show_debug_blocks: bool,
}

impl Game {
    const CAMERA_SPEED_NORMAL: f32 = 8.0;
    const CAMERA_SPEED_FAST: f32 = 16.0;
    async fn new() -> Self {
        Self {
            world_manager: WorldManager::new(),
            particle_manager: ParticleManager::new(),
            persistence_manager: PersistenceManager::new(),
            lang_manager: LanguageManager::new(),
            notification_manager: NotificationManager::new(),
            input_handler: InputHandler::new(),
            select_block: SelectBlock::new(),
            camera: Camera::new(),
            on_title_screen: true,
            is_menu_visible: false,
            show_debug_blocks: false,
        }
    }

    fn update(&mut self) {
        if self.persistence_manager.is_loading {
            if let Some((success, data)) = self.persistence_manager.check_load_status() {
                if success {
                    // Apply loaded data
                    // Assuming data is Value::Object
                    if let serde_json::Value::Object(map) = data {
                        if let Some(cx) = map.get("camera_x").and_then(|v| v.as_f64()) {
                            self.camera.x = cx as f32;
                        }
                        if let Some(cy) = map.get("camera_y").and_then(|v| v.as_f64()) {
                            self.camera.y = cy as f32;
                        }
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

                        // Re-gen chunks
                        // ... simplified, just call generate_visible_chunks next frame or now
                        self.world_manager
                            .generate_visible_chunks(self.camera.x, self.camera.y);

                        // Apply mods if any
                        if let Some(serde_json::Value::Array(mods)) = map.get("modified_chunks") {
                            self.world_manager.apply_modifications(mods.clone());
                        }
                    }

                    self.notification_manager
                        .add_notification("Loaded!".to_string(), "success");
                    self.on_title_screen = false;
                    self.is_menu_visible = false;
                } else {
                    if let serde_json::Value::String(msg) = data {
                        self.notification_manager.add_notification(msg, "error");
                    }
                }
            }
            return; // Block updates while loading
        }

        if self.on_title_screen {
            // Check start button click in Draw
        } else if self.is_menu_visible {
            // Menu logic
            if is_key_pressed(KeyCode::Escape) {
                self.is_menu_visible = false;
            }
        } else {
            if is_key_pressed(KeyCode::Escape) {
                self.is_menu_visible = true;
            }

            let camera_intents = self.input_handler.handle_camera_movement();
            let mut moved = false;
            let speed = if is_key_down(KeyCode::LeftShift) {
                Self::CAMERA_SPEED_FAST
            } else {
                Self::CAMERA_SPEED_NORMAL
            };

            for intent in camera_intents {
                match intent {
                    CameraMoveIntent::Up => {
                        self.camera.y -= speed;
                        moved = true;
                    }
                    CameraMoveIntent::Down => {
                        self.camera.y += speed;
                        moved = true;
                    }
                    CameraMoveIntent::Left => {
                        self.camera.x -= speed;
                        moved = true;
                    }
                    CameraMoveIntent::Right => {
                        self.camera.x += speed;
                        moved = true;
                    }
                    _ => {} // None or other intents
                }
            }

            if moved {
                self.world_manager
                    .generate_visible_chunks(self.camera.x, self.camera.y);
            }

            // Mouse Interaction
            let mx = (mouse_position().0 / screen_width()) * SCREEN_WIDTH;
            let my = (mouse_position().1 / screen_height()) * SCREEN_HEIGHT;
            let world_mx = (mx + self.camera.x).round(); // Round to nearest integer
            let world_my = (my + self.camera.y).round(); // Round to nearest integer

            let hovered_block_coords = self
                .world_manager
                .get_block_at_world_coords(world_mx, world_my)
                .map(|b| {
                    if !b.is_broken {
                        Some((b.x, b.y)) // Return block's grid coordinates if not broken
                    } else {
                        None // Block is broken, no hover effect
                    }
                })
                .flatten(); // Flatten Option<Option<T>> to Option<T>
            self.select_block.update(hovered_block_coords);

            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(block) = self
                    .world_manager
                    .get_block_at_world_coords(world_mx, world_my)
                {
                    if block.is_broken {
                        // Already broken, do nothing
                    } else {
                        // Mark as modified if it's the first hit or already modified
                        if block.current_hp == block.max_hp {
                            block.is_modified = true;
                        }
                        block.current_hp -= 1;

                        if block.current_hp <= 0 {
                            block.current_hp = 0;
                            block.is_broken = true;
                            block.is_modified = true;
                            let count = ::rand::thread_rng().random_range(5..15); // Particle count
                            let particles = (0..count)
                                .map(|_| Particle::new(block.x, block.y, block.max_hp))
                                .collect();
                            self.particle_manager.add_particles(particles);
                        }
                    }
                }
            }

            let blocks = self
                .world_manager
                .get_active_blocks_in_view(self.camera.x, self.camera.y);
            self.particle_manager.update(&blocks); // Pass ref to slice
        }

        self.notification_manager.update();

        // Check Save
        if let Some((success, msg)) = self.persistence_manager.check_save_status() {
            let t = if success { "success" } else { "error" };
            self.notification_manager.add_notification(msg, t);
        }
    }

    fn draw(&mut self) {
        println!("Game: Drawing is now handled by GameRenderer.");
    }

    // Helper to gather save data
    fn make_save_data(&self) -> serde_json::Value {
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
            "world_seed_main": self.world_manager.world_seed_main,
            "world_seed_ore": self.world_manager.world_seed_ore,
            "modified_chunks": modified_chunks_data
        })
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;
    let mut game_renderer = GameRenderer::new().await;

    // Hide system cursor
    show_mouse(false);

    // Create render target once
    let render_target = render_target(SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32);
    render_target.texture.set_filter(FilterMode::Nearest);

    loop {
        game.update();

        // Render to the off-screen render target
        // Set camera to render target
        let mut camera_to_render_target =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT));
        camera_to_render_target.render_target = Some(render_target.clone());
        set_camera(&camera_to_render_target);
        clear_background(SKYBLUE); // Clear the render target
        let ui_events = game_renderer.draw(&mut game);
        set_default_camera(); // Switch back to drawing on screen, will unset the render target automatically

        // Process UI events
        for event in ui_events {
            match event {
                GameEvent::StartGame => {
                    game.persistence_manager.load_game();
                    // If load fails or file missing, we should probably start new game.
                    // For now, load_game triggers thread. If it returns false/empty, we init default.
                    // Logic simplifies to: just start for now.
                    game.world_manager.generate_visible_chunks(0.0, 0.0);
                    game.on_title_screen = false;
                    game.notification_manager
                        .add_notification("Welcome!".to_string(), "success");
                }
                GameEvent::SaveGame => {
                    game.persistence_manager.save_game(game.make_save_data());
                }
                GameEvent::QuitGame => {
                    std::process::exit(0);
                }
            }
        }

        // Calculate aspect ratio and scaling for letterboxing/pillarboxing
        let target_aspect = SCREEN_WIDTH / SCREEN_HEIGHT;
        let screen_aspect = screen_width() / screen_height();

        let (render_width, render_height, offset_x, offset_y);

        if screen_aspect > target_aspect {
            // Screen is wider, pillarbox
            render_height = screen_height();
            render_width = SCREEN_WIDTH * (render_height / SCREEN_HEIGHT);
            offset_x = (screen_width() - render_width) / 2.0;
            offset_y = 0.0;
        } else {
            // Screen is taller or same aspect, letterbox
            render_width = screen_width();
            render_height = SCREEN_HEIGHT * (render_width / SCREEN_WIDTH);
            offset_x = 0.0;
            offset_y = (screen_height() - render_height) / 2.0;
        }

        let render_width_floored = render_width.floor();
        let render_height_floored = render_height.floor();
        let offset_x_floored = offset_x.floor();
        let offset_y_floored = offset_y.floor();

        // Draw black bars (clear entire screen with black)
        clear_background(BLACK);

        // Draw the render target to the screen, scaled and positioned
        draw_texture_ex(
            &render_target.texture,
            offset_x_floored,
            offset_y_floored + render_height_floored, // Y position needs to be adjusted for macroquad's flipped Y with render targets
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(render_width_floored, -render_height_floored)), // Flip vertically
                ..Default::default()
            },
        );

        next_frame().await
    }
}
