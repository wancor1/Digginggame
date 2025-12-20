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
use crate::utils::world_to_chunk_coords;
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
    on_save_select_screen: bool,
    on_new_game_input_screen: bool,
    is_menu_visible: bool,
    show_debug_blocks: bool,

    // Save/Load State
    pub save_files: Vec<String>,
    pub current_save_name: String,
    pub input_buffer: String,
}

impl Game {
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
            on_save_select_screen: false,
            on_new_game_input_screen: false,
            is_menu_visible: false,
            show_debug_blocks: false,
            save_files: Vec::new(),
            current_save_name: "savegame.json".to_string(), // Default
            input_buffer: String::new(),
        }
    }

    fn update(&mut self, game_renderer: &GameRenderer) {
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

                    self.notification_manager.add_notification(
                        "Loaded!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                    self.on_title_screen = false;
                    self.is_menu_visible = false;
                } else {
                    if let serde_json::Value::String(msg) = data {
                        self.notification_manager.add_notification(
                            msg,
                            "error",
                            game_renderer.get_font(),
                        );
                    }
                }
            }
            return; // Block updates while loading
        }

        // Handle UI screen logic if any of them are active
        if self.on_title_screen
            || self.on_save_select_screen
            || self.on_new_game_input_screen
            || self.is_menu_visible
        {
            // Specific update logic for these screens.
            // Currently, only the in-game menu needs an update check here.
            if self.is_menu_visible {
                if is_key_pressed(KeyCode::Escape) {
                    self.is_menu_visible = false;
                }
            }
        } else {
            // MAIN GAME LOGIC: This block will now only execute when NO overlay UI is active.
            if is_key_pressed(KeyCode::Escape) {
                self.is_menu_visible = true;
            }

            let camera_intents = self.input_handler.handle_camera_movement();
            let mut moved = false;
            let speed = if is_key_down(KeyCode::LeftShift) {
                CAMERA_SPEED_FAST
            } else {
                CAMERA_SPEED_NORMAL
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
                .map(|(_, _, _, _, block)| {
                    if !block.is_broken {
                        Some((block.x, block.y)) // Return block's grid coordinates if not broken
                    } else {
                        None // Block is broken, no hover effect
                    }
                })
                .flatten(); // Flatten Option<Option<T>> to Option<T>
            self.select_block.update(hovered_block_coords);

            if is_mouse_button_pressed(MouseButton::Left) {
                let (cx, cy) = world_to_chunk_coords(world_mx, world_my);
                self.world_manager.ensure_chunk_exists_and_generated(cx, cy);

                if let Some((cx, cy, _rel_x, _rel_y, block)) = self
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
                            let particles: Vec<Particle> = (0..count)
                                .map(|_| {
                                    let particle_color = block.sprite_rect.map_or(WHITE, |rect| {
                                        game_renderer.get_random_pixel_color(rect)
                                    });
                                    Particle::new(block.x, block.y, particle_color)
                                })
                                .collect();
                            self.particle_manager.add_particles(particles);
                        }

                        // Mark the parent chunk as modified
                        if let Some(chunk) = self.world_manager.get_chunk_mut(cx, cy) {
                            chunk.is_modified_in_session = true;
                        }
                    }
                }
            }

            let blocks = self
                .world_manager
                .get_active_blocks_in_view(self.camera.x, self.camera.y);
            self.particle_manager.update(&blocks, &self.camera); // Pass ref to slice
        }

        self.notification_manager.update();

        // Check Save
        if let Some((success, msg)) = self.persistence_manager.check_save_status() {
            let t = if success { "success" } else { "error" };
            self.notification_manager
                .add_notification(msg, t, game_renderer.get_font());
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

    fn return_to_title_screen(&mut self, game_renderer: &GameRenderer) {
        self.world_manager = WorldManager::new(); // Reset world
        self.particle_manager = ParticleManager::new(); // Reset particles
        self.camera = Camera::new(); // Reset camera
        self.on_title_screen = true;
        self.on_save_select_screen = false;
        self.on_new_game_input_screen = false;
        self.is_menu_visible = false;
        self.save_files = Vec::new(); // Clear save file list
        self.current_save_name = "savegame.json".to_string(); // Reset default
        self.input_buffer = String::new(); // Clear input buffer
        self.notification_manager.add_notification(
            "Returned to Title Screen".to_string(),
            "info",
            game_renderer.get_font(),
        );
    }

    fn return_to_title_from_save_select(&mut self) {
        self.on_title_screen = true;
        self.on_save_select_screen = false;
        self.on_new_game_input_screen = false;
        self.is_menu_visible = false;
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
        game.update(&game_renderer);

        // Render to the off-screen render target
        // Set camera to render target
        let mut camera_to_render_target =
            Camera2D::from_display_rect(Rect::new(0.0, 0.0, SCREEN_WIDTH, SCREEN_HEIGHT));
        camera_to_render_target.render_target = Some(render_target.clone());
        set_camera(&camera_to_render_target);
        clear_background(SKYBLUE); // Clear the render target
        let ui_events = game_renderer.draw(&mut game);
        set_default_camera(); // Switch back to drawing on screen, will unset the render target automatically

        let mut additional_ui_events = Vec::new(); // Moved here

        // Input handling for new game screen

        if game.on_new_game_input_screen {
            while let Some(c) = get_char_pressed() {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    game.input_buffer.push(c);
                }
            }

            if is_key_pressed(KeyCode::Backspace) {
                game.input_buffer.pop();
            }

            if is_key_pressed(KeyCode::Enter) {
                additional_ui_events.push(GameEvent::ConfirmNewGame(game.input_buffer.clone()));
            }
        }

        // Process UI events

        for event in ui_events
            .into_iter()
            .chain(additional_ui_events.into_iter())
        {
            match event {
                GameEvent::StartGame => {

                    // Legacy, not used directly now
                }

                GameEvent::OpenSaveSelection => {
                    game.save_files = PersistenceManager::list_save_files();

                    game.on_title_screen = false;

                    if game.save_files.is_empty() {
                        game.on_new_game_input_screen = true;

                        game.input_buffer.clear();
                    } else {
                        game.on_save_select_screen = true;
                    }
                }

                GameEvent::LoadSave(filename) => {
                    game.current_save_name = filename.clone();

                    game.persistence_manager.load_game(filename);

                    game.on_save_select_screen = false;

                    // Loading handled in update() via check_load_status
                }

                GameEvent::StartNewGameSetup => {
                    game.on_save_select_screen = false;

                    game.on_new_game_input_screen = true;

                    game.input_buffer.clear();
                }

                GameEvent::ConfirmNewGame(name) => {
                    let mut filename = name.clone();

                    if !filename.ends_with(".json") {
                        filename.push_str(".json");
                    }

                    game.current_save_name = filename;

                    game.on_new_game_input_screen = false;

                    // Start new game logic

                    game.world_manager.seed(::rand::random(), ::rand::random());

                    game.world_manager.generate_visible_chunks(0.0, 0.0);

                    game.notification_manager.add_notification(
                        "New Game!".to_string(),
                        "success",
                        game_renderer.get_font(),
                    );
                }

                GameEvent::SaveGame => {
                    game.persistence_manager
                        .save_game(game.current_save_name.clone(), game.make_save_data());
                }

                GameEvent::QuitGame => {
                    std::process::exit(0);
                }
                GameEvent::ReturnToTitle => {
                    game.return_to_title_screen(&game_renderer);
                }
                GameEvent::ReturnToTitleFromSaveSelect => {
                    game.return_to_title_from_save_select();
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
