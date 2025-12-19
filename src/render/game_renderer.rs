use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::ui::ButtonBox;
use crate::utils::calculate_text_center_position;
use ::rand::Rng;
use macroquad::prelude::*;
use macroquad::text::load_ttf_font_from_bytes;

pub struct GameRenderer {
    atlas: Option<Texture2D>,
    atlas_image: Option<macroquad::texture::Image>, // Add this line
    font: Option<Font>,
}

impl GameRenderer {
    pub async fn new() -> Self {
        // Load Assets
        let atlas_bytes = include_bytes!("../../src/atlas.png"); // Adjusted path
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

        let font_bytes = include_bytes!("../../src/misaki_gothic.ttf"); // Adjusted path
        let font = Some(load_ttf_font_from_bytes(font_bytes).unwrap());

        Self {
            atlas,
            atlas_image: Some(mq_image),
            font,
        }
    }

    pub fn get_average_block_color(&self, rect: Rect) -> Color {
        if let Some(atlas_image) = &self.atlas_image {
            let mut total_r: u32 = 0;
            let mut total_g: u32 = 0;
            let mut total_b: u32 = 0;
            let mut total_a: u32 = 0;
            let mut pixel_count: u32 = 0;

            let img_width = atlas_image.width as usize;
            let img_height = atlas_image.height as usize;

            let rect_x_start = rect.x as usize;
            let rect_y_start = rect.y as usize;
            let rect_x_end = (rect.x + rect.w) as usize;
            let rect_y_end = (rect.y + rect.h) as usize;

            for y in rect_y_start..rect_y_end {
                for x in rect_x_start..rect_x_end {
                    if x < img_width && y < img_height {
                        let index = (y * img_width + x) * 4; // 4 bytes per pixel (RGBA)
                        if index + 3 < atlas_image.bytes.len() {
                            total_r += atlas_image.bytes[index] as u32;
                            total_g += atlas_image.bytes[index + 1] as u32;
                            total_b += atlas_image.bytes[index + 2] as u32;
                            total_a += atlas_image.bytes[index + 3] as u32;
                            pixel_count += 1;
                        }
                    }
                }
            }

            if pixel_count > 0 {
                let avg_r = (total_r / pixel_count) as u8;
                let avg_g = (total_g / pixel_count) as u8;
                let avg_b = (total_b / pixel_count) as u8;
                let avg_a = (total_a / pixel_count) as u8;
                Color::new(
                    avg_r as f32 / 255.0,
                    avg_g as f32 / 255.0,
                    avg_b as f32 / 255.0,
                    avg_a as f32 / 255.0,
                )
            } else {
                // If rect is out of bounds or empty, return a default color
                WHITE
            }
        } else {
            // If atlas_image is not loaded, return a default color
            WHITE
        }
    }

    pub fn get_random_pixel_color(&self, rect: Rect) -> Color {
        if let Some(atlas_image) = &self.atlas_image {
            let img_width = atlas_image.width as usize;
            let img_height = atlas_image.height as usize;

            let rect_x_start = rect.x as usize;
            let rect_y_start = rect.y as usize;
            let rect_x_end = (rect.x + rect.w) as usize;
            let rect_y_end = (rect.y + rect.h) as usize;

            let mut rng = ::rand::thread_rng();

            if rect_x_start >= img_width || rect_y_start >= img_height {
                return WHITE; // Rect starts out of image bounds
            }

            let valid_x_range_start = rect_x_start;
            let valid_x_range_end = (rect_x_end).min(img_width);
            let valid_y_range_start = rect_y_start;
            let valid_y_range_end = (rect_y_end).min(img_height);

            if valid_x_range_start >= valid_x_range_end || valid_y_range_start >= valid_y_range_end
            {
                return WHITE; // No valid pixels in rect or rect out of bounds
            }

            let rand_x = rng.random_range(valid_x_range_start..valid_x_range_end);
            let rand_y = rng.random_range(valid_y_range_start..valid_y_range_end);

            let index = (rand_y * img_width + rand_x) * 4; // 4 bytes per pixel (RGBA)

            if index + 3 < atlas_image.bytes.len() {
                let r = atlas_image.bytes[index] as u32;
                let g = atlas_image.bytes[index + 1] as u32;
                let b = atlas_image.bytes[index + 2] as u32;
                let a = atlas_image.bytes[index + 3] as u32;
                Color::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                )
            } else {
                WHITE // Fallback if random pixel index is somehow out of bounds
            }
        } else {
            WHITE // If atlas_image is not loaded, return a default color
        }
    }

    pub fn get_font(&self) -> Option<&Font> {
        self.font.as_ref()
    }

    pub fn draw(&mut self, game: &mut Game) -> Vec<GameEvent> {
        let mut events = Vec::new();
        clear_background(SKYBLUE);

        // Simulate Camera
        // Macroquad has set_camera, usually used with Camera2D.
        // We implemented manual camera offsets.
        // We'll use set_camera for everything world-related.

        if game.on_title_screen {
            let title = "Digging Game";
            let (tx, ty) = calculate_text_center_position(SCREEN_WIDTH, SCREEN_HEIGHT, title);
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
                &game.lang_manager,
                self.font.as_ref(),
            ) {
                // events.push(GameEvent::StartGame); // Changed flow
                events.push(GameEvent::OpenSaveSelection);
            }

            // Quit Game button on title screen
            let quit_by = by + bh + 5.0; // Position below the start button
            if ButtonBox::draw_button(
                bx,
                quit_by,
                bw,
                bh,
                "button.menu.quit.default",
                "button.menu.quit.pressed",
                &game.lang_manager,
                self.font.as_ref(),
            ) {
                events.push(GameEvent::QuitGame);
            }
        } else if game.on_save_select_screen {
            let title = "Select Save File";
            let (tx, ty) = calculate_text_center_position(SCREEN_WIDTH, SCREEN_HEIGHT, title);
            draw_text_ex(
                title,
                tx.floor(),
                20.0,
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: self.font.as_ref().or_else(|| TextParams::default().font),
                    color: WHITE,
                    ..Default::default()
                },
            );

            let btn_h = 10.0;
            let start_y = 35.0;
            let mut current_y = start_y;

            // New Game Button (Always at top)
            if ButtonBox::draw_button(
                10.0,
                current_y,
                SCREEN_WIDTH - 20.0,
                btn_h,
                "button.menu.new_game.default",
                "button.menu.new_game.pressed",
                &game.lang_manager,
                self.font.as_ref(),
            ) {
                events.push(GameEvent::ReturnToTitle);
            }
            if ButtonBox::draw_button(
                2.0,
                2.0,
                30.0,
                btn_h,
                "button.menu.return.default",
                "button.menu.return.pressed",
                &game.lang_manager,
                self.font.as_ref(),
            ) {
                events.push(GameEvent::ReturnToTitlesScreenButThisIsLoadScreenOnly);
            }
            current_y += 15.0;

            // List Save Files
            for file in &game.save_files {
                if ButtonBox::draw_button(
                    10.0,
                    current_y,
                    SCREEN_WIDTH - 20.0,
                    btn_h,
                    file,
                    file,
                    &game.lang_manager,
                    self.font.as_ref(),
                ) {
                    events.push(GameEvent::LoadSave(file.clone()));
                }
                current_y += 12.0;
            }
        } else if game.on_new_game_input_screen {
            let title = "Enter Filename:";
            draw_text_ex(
                title,
                10.0,
                30.0,
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: self.font.as_ref().or_else(|| TextParams::default().font),
                    color: WHITE,
                    ..Default::default()
                },
            );

            // Draw box for input
            draw_rectangle(10.0, 40.0, SCREEN_WIDTH - 20.0, 12.0, DARKGRAY);
            draw_text_ex(
                &format!(
                    "{}{}",
                    game.input_buffer,
                    if (get_time() * 2.0) as i32 % 2 == 0 {
                        "|"
                    } else {
                        ""
                    }
                ), // simple cursor
                12.0,
                49.0,
                TextParams {
                    font_size: FONT_SIZE as u16,
                    font: self.font.as_ref().or_else(|| TextParams::default().font),
                    color: WHITE,
                    ..Default::default()
                },
            );

            if ButtonBox::draw_button(
                10.0,
                60.0,
                60.0,
                10.0,
                "Confirm",
                "Confirm",
                &game.lang_manager,
                self.font.as_ref(),
            ) {
                events.push(GameEvent::ConfirmNewGame(game.input_buffer.clone()));
            }
        } else {
            // This 'else' covers the case where none of the UI screens are active.
            // Game World
            let _cam = Camera2D {
                target: vec2(
                    game.camera.x + SCREEN_WIDTH / 2.0,
                    game.camera.y + SCREEN_HEIGHT / 2.0,
                ),
                zoom: vec2(1.0 / (SCREEN_WIDTH / 2.0), 1.0 / (SCREEN_HEIGHT / 2.0)), // No longer invert Y
                ..Default::default()
            };

            // Draw World elements with manual camera offset

            let cx = game.camera.x;
            let cy = game.camera.y;

            let blocks = game.world_manager.get_active_blocks_in_view(cx, cy);
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
            for p in &game.particle_manager.active_particles {
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
            if let Some(atlas) = self.atlas.as_ref() {
                game.select_block.draw(cx, cy, atlas);
            }

            // UI Overlay
            if game.is_menu_visible {
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
                    &game.lang_manager,
                    self.font.as_ref(),
                ) {
                    events.push(GameEvent::SaveGame);
                }
                current_y = (current_y + 15.0).floor();
                if ButtonBox::draw_button(
                    menu_x + 5.0,
                    current_y,
                    menu_w - 10.0,
                    btn_h,
                    "button.menu.quit_to_title.default",
                    "button.menu.quit_to_title.pressed",
                    &game.lang_manager,
                    self.font.as_ref(),
                ) {
                    events.push(GameEvent::ReturnToTitle);
                }
            }
        }

        game.notification_manager.draw(self.font.as_ref());

        // Cursor
        let curs_x = ((mouse_position().0 / screen_width()) * SCREEN_WIDTH).floor();
        let curs_y = ((mouse_position().1 / screen_height()) * SCREEN_HEIGHT).floor(); // cursor_offset_y removed.
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
        return events;
    }
}
