use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
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



    pub fn get_random_pixel_color(&self, rect: Rect) -> Color {
        if let Some(atlas_image) = &self.atlas_image {
            let img_width = atlas_image.width as usize;
            let img_height = atlas_image.height as usize;

            let rect_x_start = rect.x as usize;
            let rect_y_start = rect.y as usize;
            let rect_x_end = (rect.x + rect.w) as usize;
            let rect_y_end = (rect.y + rect.h) as usize;

            let mut rng = ::rand::rng();

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

    pub fn draw_world(&mut self, game: &mut Game) {
        let cx = game.camera.x;
        let cy = game.camera.y;

        let blocks = game.world_manager.get_active_blocks_in_view(cx, cy);
        for block in blocks {
            let draw_x = (block.x - cx).floor();
            let draw_y = (block.y - cy).floor();

            if let (Some(rect), Some(atlas)) = (block.sprite_rect, self.atlas.as_ref()) {
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

                if block.current_hp < block.max_hp && block.max_hp > 0 {
                    let damage = (block.max_hp - block.current_hp) as f32 / block.max_hp as f32;
                    let frame = (damage * 5.0).ceil() as i32;
                    if frame > 0 {
                        let anim_v =
                            SPRITE_BREAK_ANIM_V_START + ((frame - 1).max(0) as f32) * BLOCK_SIZE;
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
                draw_rectangle(draw_x, draw_y, BLOCK_SIZE, BLOCK_SIZE, BROWN);
            }
        }

        for p in &game.particle_manager.active_particles {
            draw_rectangle((p.x - cx).floor(), (p.y - cy).floor(), 1.0, 1.0, p.color);
        }

        let player = &game.player_manager.player;
        draw_rectangle(
            (player.x - cx).floor(),
            (player.y - cy).floor(),
            player.width,
            player.height,
            ORANGE,
        );
        draw_rectangle(
            (player.x - cx + 1.0).floor(),
            (player.y - cy + 1.0).floor(),
            player.width - 2.0,
            2.0,
            YELLOW,
        );

        if let Some(atlas) = self.atlas.as_ref() {
            game.select_block.draw(cx, cy, atlas);
        }

        for gate in &game.player_manager.player.warp_gates {
            draw_rectangle(
                (gate.x - cx).floor(),
                (gate.y - cy).floor(),
                8.0,
                8.0,
                PURPLE,
            );
            draw_rectangle_lines(
                (gate.x - cx).floor(),
                (gate.y - cy).floor(),
                8.0,
                8.0,
                1.0,
                WHITE,
            );
        }

        for item in &game.item_manager.items {
            let draw_x = (item.x - cx).floor();
            let draw_y = (item.y - cy).floor();
            if let Some(atlas) = self.atlas.as_ref() {
                draw_texture_ex(
                    atlas,
                    draw_x,
                    draw_y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(item.sprite_rect),
                        dest_size: Some(vec2(4.0, 4.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }

    pub fn draw_ui(&mut self, game: &mut Game) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Calculate scaling and offsets to match the world view's letterboxing/pillarboxing
        let target_aspect = SCREEN_WIDTH / SCREEN_HEIGHT;
        let screen_aspect = screen_width() / screen_height();

        let (render_width, render_height, offset_x, offset_y);
        if screen_aspect > target_aspect {
            render_height = screen_height();
            render_width = SCREEN_WIDTH * (render_height / SCREEN_HEIGHT);
            offset_x = (screen_width() - render_width) / 2.0;
            offset_y = 0.0;
        } else {
            render_width = screen_width();
            render_height = SCREEN_HEIGHT * (render_width / SCREEN_WIDTH);
            offset_x = 0.0;
            offset_y = (screen_height() - render_height) / 2.0;
        }

        let scale = render_width / SCREEN_WIDTH;
        let s_font_size = (FONT_SIZE * scale).floor() as u16;

        // Use default camera (screen space) for high-res UI
        set_default_camera();

        if game.on_title_screen {
            let title = "Digging Game";
            let center_x = screen_width() / 2.0;
            let center_y = screen_height() / 2.0;

            // Draw Title centered
            let params = TextParams {
                font_size: (FONT_SIZE * scale * 2.0) as u16,
                font: self.font.as_ref(),
                color: WHITE,
                ..Default::default()
            };
            let t_measure = measure_text(title, self.font.as_ref(), params.font_size, 1.0);
            draw_text_ex(
                title,
                (center_x - t_measure.width / 2.0).floor(),
                (center_y * 0.5).floor(),
                params,
            );

            let bw = 60.0 * scale;
            let bh = 10.0 * scale;
            let bx = (screen_width() - bw) / 2.0;
            let by = center_y;

            if self.draw_high_res_button(
                bx,
                by,
                bw,
                bh,
                "button.title_screen.start.default",
                "button.title_screen.start.pressed",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::OpenSaveSelection);
            }
            if self.draw_high_res_button(
                bx,
                by + 15.0 * scale,
                bw,
                bh,
                "button.menu.quit.default",
                "button.menu.quit.pressed",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::QuitGame);
            }
        } else if game.on_save_select_screen {
            draw_text_ex(
                "Select Save File",
                offset_x + 10.0 * scale,
                offset_y + 20.0 * scale,
                TextParams {
                    font_size: s_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            let mut cy = offset_y + 35.0 * scale;
            if self.draw_high_res_button(
                offset_x + 10.0 * scale,
                cy,
                (SCREEN_WIDTH - 20.0) * scale,
                10.0 * scale,
                "button.menu.new_game.default",
                "button.menu.new_game.pressed",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::StartNewGameSetup);
            }
            if self.draw_high_res_button(
                offset_x + 2.0 * scale,
                offset_y + 2.0 * scale,
                30.0 * scale,
                10.0 * scale,
                "button.menu.return.default",
                "button.menu.return.pressed",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::ReturnToTitleFromSaveSelect);
            }
            cy += 15.0 * scale;
            for file in &game.save_files {
                if self.draw_high_res_button(
                    offset_x + 10.0 * scale,
                    cy,
                    (SCREEN_WIDTH - 20.0) * scale,
                    10.0 * scale,
                    file,
                    file,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::LoadSave(file.clone()));
                }
                cy += 12.0 * scale;
            }
        } else if game.on_new_game_input_screen {
            draw_text_ex(
                "Enter Filename:",
                offset_x + 10.0 * scale,
                offset_y + 30.0 * scale,
                TextParams {
                    font_size: s_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            draw_rectangle(
                offset_x + 10.0 * scale,
                offset_y + 40.0 * scale,
                (SCREEN_WIDTH - 20.0) * scale,
                12.0 * scale,
                DARKGRAY,
            );
            let cur = if (get_time() * 2.0) as i32 % 2 == 0 {
                "|"
            } else {
                ""
            };
            draw_text_ex(
                &format!("{}{}", game.input_buffer, cur),
                offset_x + 12.0 * scale,
                offset_y + 49.0 * scale,
                TextParams {
                    font_size: s_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            if self.draw_high_res_button(
                offset_x + 10.0 * scale,
                offset_y + 60.0 * scale,
                60.0 * scale,
                10.0 * scale,
                "Confirm",
                "Confirm",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::ConfirmNewGame(game.input_buffer.clone()));
            }
        } else if game.on_warp_place_screen {
            draw_text_ex(
                &game.lang_manager.get_string("warp.name_prompt"),
                offset_x + 10.0 * scale,
                offset_y + 30.0 * scale,
                TextParams {
                    font_size: s_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            draw_rectangle(
                offset_x + 10.0 * scale,
                offset_y + 40.0 * scale,
                (SCREEN_WIDTH - 20.0) * scale,
                12.0 * scale,
                DARKGRAY,
            );
            let cur = if (get_time() * 2.0) as i32 % 2 == 0 {
                "|"
            } else {
                ""
            };
            draw_text_ex(
                &format!("{}{}", game.input_buffer, cur),
                offset_x + 12.0 * scale,
                offset_y + 49.0 * scale,
                TextParams {
                    font_size: s_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            if self.draw_high_res_button(
                offset_x + 10.0 * scale,
                offset_y + 60.0 * scale,
                60.0 * scale,
                10.0 * scale,
                "Confirm",
                "Confirm",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::ConfirmWarpGateName(game.input_buffer.clone()));
            }
        } else if game.on_warp_select_screen {
            draw_text_ex(
                &game.lang_manager.get_string("warp.title"),
                offset_x + 10.0 * scale,
                offset_y + 20.0 * scale,
                TextParams {
                    font_size: s_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );

            if self.draw_high_res_button(
                offset_x + 2.0 * scale,
                offset_y + 2.0 * scale,
                30.0 * scale,
                10.0 * scale,
                "shop.back_to_game",
                "shop.back_to_game",
                &game.lang_manager,
                s_font_size,
            ) {
                events.push(GameEvent::CloseMenu);
            }

            let mut cy = offset_y + 35.0 * scale;
            for (i, gate) in game.player_manager.player.warp_gates.iter().enumerate() {
                let label = format!("> {}", gate.name);
                if self.draw_high_res_button(
                    offset_x + 10.0 * scale,
                    cy,
                    (SCREEN_WIDTH - 20.0) * scale,
                    10.0 * scale,
                    &label,
                    &label,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::TeleportToWarp(i));
                }
                cy += 12.0 * scale;
            }
        } else {
            // HUD
            let player = &game.player_manager.player;
            let hud_y = offset_y + 5.0 * scale;
            let hud_x = offset_x + 5.0 * scale;
            let mini_font_size = (6.0 * scale) as u16;

            draw_rectangle(hud_x, hud_y, 40.0 * scale, 4.0 * scale, DARKGRAY);
            let fuel_ratio = player.fuel / player.max_fuel;
            draw_rectangle(
                hud_x,
                hud_y,
                40.0 * scale * fuel_ratio,
                4.0 * scale,
                if fuel_ratio > 0.3 { GREEN } else { RED },
            );

            draw_text_ex(
                "FUEL",
                hud_x,
                hud_y + 8.0 * scale,
                TextParams {
                    font_size: mini_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            draw_text_ex(
                &format!("CARGO: {}/{}", player.cargo.len(), player.max_cargo),
                hud_x + 50.0 * scale,
                hud_y + 4.0 * scale,
                TextParams {
                    font_size: mini_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );
            draw_text_ex(
                &format!("$: {}", player.money),
                offset_x + (SCREEN_WIDTH - 45.0) * scale,
                hud_y + 4.0 * scale,
                TextParams {
                    font_size: mini_font_size,
                    font: self.font.as_ref(),
                    color: YELLOW,
                    ..Default::default()
                },
            );

            let depth = (player.y / BLOCK_SIZE).floor() as i32 - SURFACE_Y_LEVEL;
            draw_text_ex(
                &format!("DEPTH: {}m", depth.max(0)),
                offset_x + (SCREEN_WIDTH - 45.0) * scale,
                hud_y + 12.0 * scale,
                TextParams {
                    font_size: mini_font_size,
                    font: self.font.as_ref(),
                    color: WHITE,
                    ..Default::default()
                },
            );

            if game.on_surface {
                if !game.is_menu_visible && !game.is_shop_open {
                    if self.draw_high_res_button(
                        offset_x + (SCREEN_WIDTH - 40.0) * scale,
                        offset_y + 25.0 * scale,
                        35.0 * scale,
                        10.0 * scale,
                        "SHOP",
                        "SHOP",
                        &game.lang_manager,
                        s_font_size,
                    ) {
                        game.is_shop_open = true;
                    }
                    if self.draw_high_res_button(
                        offset_x + 5.0 * scale,
                        offset_y + 25.0 * scale,
                        35.0 * scale,
                        10.0 * scale,
                        "hud.warp_menu",
                        "hud.warp_menu.pressed",
                        &game.lang_manager,
                        s_font_size,
                    ) {
                        events.push(GameEvent::OpenWarpMenu);
                    }
                }
            } else {
                // Underground HUD
                if game.player_manager.player.inventory_warp_gates > 0 {
                    let gate_txt = game.lang_manager.get_string("hud.warp_gates").replace(
                        "{count}",
                        &game.player_manager.player.inventory_warp_gates.to_string(),
                    );
                    draw_text_ex(
                        &gate_txt,
                        offset_x + 5.0 * scale,
                        offset_y + 25.0 * scale,
                        TextParams {
                            font_size: mini_font_size,
                            font: self.font.as_ref(),
                            color: WHITE,
                            ..Default::default()
                        },
                    );
                    if self.draw_high_res_button(
                        offset_x + 5.0 * scale,
                        offset_y + 35.0 * scale,
                        60.0 * scale,
                        10.0 * scale,
                        "hud.place_gate",
                        "hud.place_gate.pressed",
                        &game.lang_manager,
                        s_font_size,
                    ) {
                        events.push(GameEvent::StartPlaceWarpGate);
                    }
                }
            }

            // Shop UI
            if game.is_shop_open {
                let (mw, mh) = (110.0 * scale, (SCREEN_HEIGHT - 20.0) * scale);
                let (mx, my) = (
                    offset_x + ((SCREEN_WIDTH - 110.0) / 2.0).floor() * scale,
                    offset_y + 10.0 * scale,
                );
                draw_rectangle(mx, my, mw, mh, LIGHTGRAY);
                draw_rectangle_lines(mx, my, mw, mh, 1.0, BLACK);

                let mut cur_y = (my + 10.0 * scale).floor();
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    "shop.back_to_game",
                    "shop.back_to_game",
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::CloseMenu);
                }

                cur_y += 15.0 * scale;
                draw_text_ex(
                    &game.lang_manager.get_string("shop.title"),
                    mx + 10.0 * scale,
                    cur_y + 8.0 * scale,
                    TextParams {
                        font_size: mini_font_size,
                        font: self.font.as_ref(),
                        color: BLACK,
                        ..Default::default()
                    },
                );
                cur_y += 12.0 * scale;

                let dc = game.player_manager.player.drill_level * 100;
                let drill_name = game.lang_manager.get_string("shop.upgrade.drill");
                let drill_label = format!(
                    "{} Lv{} (${})",
                    drill_name, game.player_manager.player.drill_level, dc
                );
                let purchase_label = game.lang_manager.get_string("shop.purchase");
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    &drill_label,
                    &purchase_label,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::UpgradeDrill);
                }
                cur_y += 12.0 * scale;

                let tc = game.player_manager.player.tank_level * 80;
                let tank_name = game.lang_manager.get_string("shop.upgrade.tank");
                let tank_label = format!(
                    "{} Lv{} (${})",
                    tank_name, game.player_manager.player.tank_level, tc
                );
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    &tank_label,
                    &purchase_label,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::UpgradeTank);
                }
                cur_y += 12.0 * scale;

                let ec = game.player_manager.player.engine_level * 120;
                let engine_name = game.lang_manager.get_string("shop.upgrade.engine");
                let engine_label = format!(
                    "{} Lv{} (${})",
                    engine_name, game.player_manager.player.engine_level, ec
                );
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    &engine_label,
                    &purchase_label,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::UpgradeEngine);
                }
                cur_y += 12.0 * scale;

                let cc = game.player_manager.player.cargo_level * 150;
                let cargo_name = game.lang_manager.get_string("shop.upgrade.cargo");
                let cargo_label = format!(
                    "{} Lv{} (${})",
                    cargo_name, game.player_manager.player.cargo_level, cc
                );
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    &cargo_label,
                    &purchase_label,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::UpgradeCargo);
                }
                cur_y += 12.0 * scale;

                let wg_name = game.lang_manager.get_string("shop.buy.warpgate");
                let wg_label = format!("{} ($500)", wg_name);
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    &wg_label,
                    &purchase_label,
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::BuyWarpGate);
                }
            }

            // Pause Menu UI (ESC)
            if game.is_menu_visible {
                let (mw, mh) = (80.0 * scale, 60.0 * scale);
                let (mx, my) = (
                    offset_x + ((SCREEN_WIDTH - 80.0) / 2.0).floor() * scale,
                    offset_y + ((SCREEN_HEIGHT - 60.0) / 2.0).floor() * scale,
                );
                draw_rectangle(mx, my, mw, mh, LIGHTGRAY);
                draw_rectangle_lines(mx, my, mw, mh, 1.0, BLACK);

                let mut cur_y = (my + 10.0 * scale).floor();
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    "button.menu.return.default",
                    "button.menu.return.pressed",
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::CloseMenu);
                }
                cur_y += 15.0 * scale;
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    "button.menu.save.default",
                    "button.menu.save.pressed",
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::SaveGame);
                }
                cur_y += 12.0 * scale;
                if self.draw_high_res_button(
                    mx + 5.0 * scale,
                    cur_y,
                    mw - 10.0 * scale,
                    10.0 * scale,
                    "button.menu.quit_to_title.default",
                    "button.menu.quit_to_title.pressed",
                    &game.lang_manager,
                    s_font_size,
                ) {
                    events.push(GameEvent::ReturnToTitle);
                }
            }
        }

        game.notification_manager
            .draw_high_res(self.font.as_ref(), scale, offset_x, offset_y);

        // Draw Cursor
        let mouse_pos = mouse_position();
        if let Some(atlas) = self.atlas.as_ref() {
            draw_texture_ex(
                atlas,
                mouse_pos.0,
                mouse_pos.1,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_CURSOR),
                    dest_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                    ..Default::default()
                },
            );
        }

        events
    }

    fn draw_high_res_button(
        &self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        text_key: &str,
        press_key: &str,
        lang: &crate::managers::LanguageManager,
        font_size: u16,
    ) -> bool {
        let mouse_pos = mouse_position();
        let is_hover =
            mouse_pos.0 >= x && mouse_pos.0 < x + w && mouse_pos.1 >= y && mouse_pos.1 < y + h;
        let is_pressed = is_hover && is_mouse_button_down(MouseButton::Left);
        let is_released = is_hover && is_mouse_button_released(MouseButton::Left);

        let bg_col = if is_pressed {
            COLOR_BUTTON_PRESSED_BG
        } else {
            COLOR_BUTTON_BG
        };
        draw_rectangle(x.floor(), y.floor(), w, h, bg_col);
        draw_rectangle_lines(x.floor(), y.floor(), w, h, 1.0, COLOR_BUTTON_BORDER);

        let key = if is_pressed { press_key } else { text_key };
        let label = lang.get_string(key);

        let t_measure = measure_text(&label, self.font.as_ref(), font_size, 1.0);
        let tx = x + (w - t_measure.width) / 2.0;
        let ty = y + (h + t_measure.height) / 2.0;

        draw_text_ex(
            &label,
            tx.floor(),
            ty.floor(),
            TextParams {
                font_size,
                font: self.font.as_ref(),
                color: COLOR_BUTTON_TEXT,
                ..Default::default()
            },
        );

        is_released
    }
}
