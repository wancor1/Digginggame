
use macroquad::prelude::*;
use macroquad::text::load_ttf_font_from_bytes;

mod components;
mod constants;
mod managers;
mod ui;
mod utils;

use constants::*;
use managers::*;
use ui::*;
use utils::{calculate_text_center_position};

fn window_conf() -> Conf {
    Conf {
        window_title: "Digging Game".to_owned(),
        window_width: SCREEN_WIDTH as i32 * 4, // Scale up for visibility
        window_height: SCREEN_HEIGHT as i32 * 4,
        ..Default::default()
    }
}

pub struct DiggingGame {
    world_manager: WorldManager,
    particle_manager: ParticleManager,
    persistence_manager: PersistenceManager,
    lang_manager: LanguageManager,
    notification_manager: NotificationManager,
    input_handler: InputHandler,
    select_block: SelectBlock,

    // UI State
    on_title_screen: bool,
    is_menu_visible: bool,
    show_debug_blocks: bool,

    // Assets
    atlas: Option<Texture2D>,
    font: Option<Font>,
}

impl DiggingGame {
    async fn new() -> Self {
        // Load Assets
        let atlas_bytes = include_bytes!("../src/atlas.png");
        let dynamic_image = image::load_from_memory(atlas_bytes).unwrap();
        let rgba_image = dynamic_image.to_rgba8(); // Convert to RGBA8 format

        // Create macroquad::Image from rgba_image
        let mq_image = macroquad::texture::Image {
            width: rgba_image.width() as u16,
            height: rgba_image.height() as u16,
            bytes: rgba_image.into_raw(), // Get raw bytes
        };
        let atlas = Some(Texture2D::from_image(&mq_image));
        atlas.as_ref().unwrap().set_filter(FilterMode::Nearest);

        let font_bytes = include_bytes!("../src/misaki_gothic.ttf");
        let font = Some(load_ttf_font_from_bytes(font_bytes).unwrap());

        Self {
            world_manager: WorldManager::new(),
            particle_manager: ParticleManager::new(),
            persistence_manager: PersistenceManager::new(),
            lang_manager: LanguageManager::new(),
            notification_manager: NotificationManager::new(),
            input_handler: InputHandler::new(),
            select_block: SelectBlock::new(),
            on_title_screen: true,
            is_menu_visible: false,
            show_debug_blocks: false,
            atlas,
            font,
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
                            self.input_handler.camera_x = cx as f32;
                        }
                        if let Some(cy) = map.get("camera_y").and_then(|v| v.as_f64()) {
                            self.input_handler.camera_y = cy as f32;
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
                        self.world_manager.generate_visible_chunks(
                            self.input_handler.camera_x,
                            self.input_handler.camera_y,
                        );

                        // Apply mods if any
                        if let Some(serde_json::Value::Array(mods)) = map.get("modified_chunks") {
                            self.world_manager.apply_modifications(mods.clone());
                        }
                    }

                    self.notification_manager.add_notification(
                        "Loaded!".to_string(),
                        "success",
                        self.font.as_ref(),
                    );
                    self.on_title_screen = false;
                    self.is_menu_visible = false;
                } else {
                    if let serde_json::Value::String(msg) = data {
                        self.notification_manager.add_notification(
                            msg,
                            "error",
                            self.font.as_ref(),
                        );
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

            if self.input_handler.handle_camera_movement() {
                self.world_manager.generate_visible_chunks(
                    self.input_handler.camera_x,
                    self.input_handler.camera_y,
                );
            }

            // Mouse Interaction
            let mx = (mouse_position().0 / screen_width()) * SCREEN_WIDTH;
            let my = (mouse_position().1 / screen_height()) * SCREEN_HEIGHT;
            let world_mx = mx + self.input_handler.camera_x;
            let world_my = my + self.input_handler.camera_y;

            let is_hover = self
                .world_manager
                .get_block_at_world_coords(world_mx, world_my)
                .map(|b| !b.is_broken)
                .unwrap_or(false);
            self.select_block.update(is_hover);

            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(block) = self
                    .world_manager
                    .get_block_at_world_coords(world_mx, world_my)
                {
                    if let Some(particles) = block.handle_click() {
                        self.particle_manager.add_particles(particles);
                        // Sound?
                    }
                }
            }

            let blocks = self.world_manager.get_active_blocks_in_view(
                self.input_handler.camera_x,
                self.input_handler.camera_y,
            );
            self.particle_manager.update(&blocks); // Pass ref to slice
        }

        self.notification_manager.update();

        // Check Save
        if let Some((success, msg)) = self.persistence_manager.check_save_status() {
            let t = if success { "success" } else { "error" };
            self.notification_manager
                .add_notification(msg, t, self.font.as_ref());
        }
    }

    fn draw(&mut self) {
        clear_background(SKYBLUE);

        // Simulate Camera
        // Macroquad has set_camera, usually used with Camera2D.
        // We implemented manual camera offsets.
        // We'll use set_camera for everything world-related.

        if self.on_title_screen {
            let title = "Digging Game";
            let (tx, ty) = calculate_text_center_position(
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                title,
                self.font.as_ref(),
            );
            draw_text_ex(
                title,
                (tx + 1.0).floor(),
                (ty * 0.5 + 1.0).floor(),
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: self.font.as_ref().or_else(|| TextParams::default().font),
                    color: DARKGRAY,
                    ..Default::default()
                },
            );
            draw_text_ex(
                title,
                tx.floor(),
                (ty * 0.5).floor(),
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: self.font.as_ref().or_else(|| TextParams::default().font),
                    color: WHITE,
                    ..Default::default()
                },
            );

            let bw = 60.0;
            let bh = 10.0;
            let bx = ((SCREEN_WIDTH - bw) / 2.0).floor();
            let by = ((SCREEN_HEIGHT - bh) / 2.0 * 1.25).floor();

            if ButtonBox::draw_button(
                bx,
                by,
                bw,
                bh,
                "button.title_screen.start.default",
                "button.title_screen.start.pressed",
                &self.lang_manager,
                self.font.as_ref(),
            ) {
                self.persistence_manager.load_game();
                // If load fails or file missing, we should probably start new game.
                // For now, load_game triggers thread. If it returns false/empty, we init default.
                // Logic simplifies to: just start for now.
                self.world_manager.generate_visible_chunks(0.0, 0.0);
                self.on_title_screen = false;
                self.notification_manager.add_notification(
                    "Welcome!".to_string(),
                    "success",
                    self.font.as_ref(),
                );
            }
        } else {
            // Game World
            let _cam = Camera2D {
                target: vec2(
                    self.input_handler.camera_x + SCREEN_WIDTH / 2.0,
                    self.input_handler.camera_y + SCREEN_HEIGHT / 2.0,
                ),
                zoom: vec2(1.0 / (SCREEN_WIDTH / 2.0), 1.0 / (SCREEN_HEIGHT / 2.0)), // No longer invert Y
                ..Default::default()
            };
            // Macroquad coordinate system with camera typically: +Y up.
            // Our game logic is +Y down (pixel coords).
            // To keep it simple, we use a camera that maps 0..SCREEN to -1..1 but with Y flipped?
            // Or just subtract camera offset manually?
            // Let's use manual offset since we have mix of UI and world.
            // Manual offset: draw_texture(x - cam_x, y - cam_y ...)

            // Or better: use `set_camera` with a custom camera that matches screen pixel coords?
            // `set_default_camera` resets.

            // Pixel-perfect camera setup:
            let _zoom = vec2(2.0 / SCREEN_WIDTH, 2.0 / SCREEN_HEIGHT); // flips Y by default? No.
                                                                      // Standard 2D: (0,0) center.
                                                                      // We want (0,0) top-left.
                                                                      // Let's stick to manual subtraction for clarity in porting without wrestling coord systems.

            // Actually, Macroquad `draw_...` uses screen coords by default.
        }

        if !self.on_title_screen {
            // Draw World elements with manual camera offset

            let cx = self.input_handler.camera_x;

            let cy = self.input_handler.camera_y;

            let blocks = self.world_manager.get_active_blocks_in_view(cx, cy);
            for block in blocks {
                let draw_x = (block.x - cx).floor();
                let draw_y = (block.y - cy).floor();

                if let (Some(rect), Some(atlas)) = (block.sprite_rect, self.atlas.as_ref()) {
                    // Draw block from atlas.
                    draw_texture_ex(
                        atlas,
                        draw_x,
                        draw_y,
                        WHITE,
                        DrawTextureParams {
                            source: Some(rect),
                            ..Default::default()
                        },
                    );

                    // Crack overlay?
                    if block.current_hp < block.max_hp && block.max_hp > 0 {
                        let damage = (block.max_hp - block.current_hp) as f32 / block.max_hp as f32;
                        let frame = (damage * 5.0).ceil() as i32;
                        if frame > 0 {
                            // We need crack anim Rects.
                            // SPRITE_BREAK_ANIM_V_START ...
                            let anim_v = SPRITE_BREAK_ANIM_V_START
                                + ((frame - 1).max(0) as f32) * BLOCK_SIZE;
                            let crack_rect =
                                Rect::new(SPRITE_BREAK_ANIM_U, anim_v, BLOCK_SIZE, BLOCK_SIZE);
                            draw_texture_ex(
                                atlas,
                                draw_x,
                                draw_y,
                                WHITE,
                                DrawTextureParams {
                                    source: Some(crack_rect),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                } else {
                    // Fallback rect
                    draw_rectangle(draw_x, draw_y, BLOCK_SIZE, BLOCK_SIZE, BROWN);
                }
            }

            // Particles
            for p in &self.particle_manager.active_particles {
                draw_rectangle((p.x - cx).floor(), (p.y - cy).floor(), 1.0, 1.0, p.color);
            }

            // Highlight
            // SelectBlock logic draws using grid aligned mouse, but we need to account for camera?
            // Mouse used in update was screen+camera.
            // Draw should be screen relative?
            // SelectBlock logic in UI.rs used simple blt(grid).
            // If we pass mouse_x/y (screen) to it, it calculates grid.
            // But grid is World Grid.
            // So we pass (world_mouse_x, world_mouse_y).
            // And we draw at (grid_x - cx, grid_y - cy).
            // We'll adjust SelectBlock to return rect or take offset.
            // For now, let's skip drawing select block cursor to keep it simple or implement quickly:
            if let Some(_atlas) = self.atlas.as_ref() {
                let mx = mouse_position().0 + cx;
                let my = mouse_position().1 + cy;
                let grid_x = (mx / BLOCK_SIZE).floor() * BLOCK_SIZE;
                let grid_y = (my / BLOCK_SIZE).floor() * BLOCK_SIZE;

                // Reuse SelectBlock logic manually here for rendering due to camera offset complexity
                // Or pass offset to SelectBlock.draw
                // Let's manually draw for simplicity in port.
                if self.select_block.is_effect_active() {
                    // Need to expose getter or pub field
                    // draw_texture_ex...
                    draw_rectangle_lines(
                        (grid_x - cx).floor(),
                        (grid_y - cy).floor(),
                        BLOCK_SIZE,
                        BLOCK_SIZE,
                        1.0,
                        WHITE,
                    );
                }
            }

            // UI Overlay
            if self.is_menu_visible {
                // Draw menu background
                let menu_w = 100.0;
                let menu_h = SCREEN_HEIGHT - 20.0;
                let menu_x = ((SCREEN_WIDTH - menu_w) / 2.0).floor();
                let menu_y = ((SCREEN_HEIGHT - menu_h) / 2.0).floor();

                draw_rectangle(menu_x, menu_y, menu_w, menu_h, LIGHTGRAY);
                draw_rectangle_lines(menu_x, menu_y, menu_w, menu_h, 1.0, BLACK);

                // Menu buttons...
                let mut current_y = (menu_y + 10.0).floor();
                let btn_h = 10.0;

                if ButtonBox::draw_button(
                    menu_x + 5.0,
                    current_y,
                    menu_w - 10.0,
                    btn_h,
                    "button.menu.save.default",
                    "button.menu.save.pressed",
                    &self.lang_manager,
                    self.font.as_ref(),
                ) {
                    self.persistence_manager.save_game(self.make_save_data());
                }
                current_y = (current_y + 15.0).floor();
                if ButtonBox::draw_button(
                    menu_x + 5.0,
                    current_y,
                    menu_w - 10.0,
                    btn_h,
                    "button.menu.quit.default",
                    "button.menu.quit.pressed",
                    &self.lang_manager,
                    self.font.as_ref(),
                ) {
                    // std::process::exit(0);
                    // macroquad has no quit? usually just return?
                    // We can just set a flag or let user close window.
                }
            }
        }

        self.notification_manager.draw(self.font.as_ref());

        // Cursor
        let curs_x = ((mouse_position().0 / screen_width()) * SCREEN_WIDTH).floor();
        let curs_y = ((mouse_position().1 / screen_height()) * SCREEN_HEIGHT).floor();
        if let Some(atlas) = self.atlas.as_ref() {
            draw_texture_ex(
                atlas,
                curs_x,
                curs_y,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_CURSOR),
                    ..Default::default()
                },
            );
        } else {
            draw_rectangle(curs_x, curs_y, 4.0, 4.0, RED);
        }
    }

    // Helper to gather save data
    fn make_save_data(&self) -> serde_json::Value {
        // Collect chunk data
        // ... simplified
        serde_json::json!({
            "camera_x": self.input_handler.camera_x,
            "camera_y": self.input_handler.camera_y,
            "world_seed_main": self.world_manager.world_seed_main,
            "world_seed_ore": self.world_manager.world_seed_ore
            // "modified_chunks": ...
        })
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = DiggingGame::new().await;

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
        game.draw();
        set_default_camera(); // Switch back to drawing on screen, will unset the render target automatically
                              // Draw the render target to the screen, scaled
        draw_texture_ex(
            &render_target.texture,
            0.0,
            screen_height(), // Move to bottom to compensate for negative height
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), -screen_height())), // Flip vertically
                ..Default::default()
            },
        );

        next_frame().await
    }
}
