use crate::Game;
use crate::constants::*;
use crate::render::ui::common::MenuRenderContext;
use macroquad::prelude::*;

pub fn draw_map_screen(game: &mut Game, ctx: &mut MenuRenderContext) {
    let screen_w = SCREEN_WIDTH * ctx.scale;
    let screen_h = SCREEN_HEIGHT * ctx.scale;

    // Center of the map is the view position
    let view_x = game.map_view_x;
    let view_y = game.map_view_y;

    // Zoom: how many blocks one SCREEN pixel represents.
    let blocks_per_pixel = 1.0 / game.map_zoom;

    // 1. Pre-load/ensure visited chunks in view
    let half_w_blocks = (SCREEN_WIDTH / 2.0) * blocks_per_pixel;
    let half_h_blocks = (SCREEN_HEIGHT / 2.0) * blocks_per_pixel;

    let start_x_blocks = (view_x / BLOCK_SIZE - half_w_blocks).floor() as i32;
    let end_x_blocks = (view_x / BLOCK_SIZE + half_w_blocks).ceil() as i32;
    let start_y_blocks = (view_y / BLOCK_SIZE - half_h_blocks).floor() as i32;
    let end_y_blocks = (view_y / BLOCK_SIZE + half_h_blocks).ceil() as i32;

    let start_cx = (start_x_blocks as f32 / CHUNK_SIZE_X_BLOCKS as f32).floor() as i32;
    let end_cx = (end_x_blocks as f32 / CHUNK_SIZE_X_BLOCKS as f32).ceil() as i32;
    let start_cy = (start_y_blocks as f32 / CHUNK_SIZE_Y_BLOCKS as f32).floor() as i32;
    let end_cy = (end_y_blocks as f32 / CHUNK_SIZE_Y_BLOCKS as f32).ceil() as i32;

    for cx in start_cx..=end_cx {
        for cy in start_cy..=end_cy {
            if game.world_manager.visited_chunks.contains(&(cx, cy)) {
                game.world_manager.ensure_chunk_exists_and_generated(cx, cy);
            }
        }
    }

    // 2. Render pixels
    let mut last_cx = i32::MAX;
    let mut last_cy = i32::MAX;
    let mut last_visited = false;
    let mut last_chunk: Option<&crate::components::Chunk> = None;

    for py_idx in 0..SCREEN_HEIGHT as i32 {
        let py = py_idx as f32;
        for px_idx in 0..SCREEN_WIDTH as i32 {
            let px = px_idx as f32;

            // Calculate world coordinates - Match marker centering
            let off_x = px - SCREEN_WIDTH / 2.0;
            let off_y = py - SCREEN_HEIGHT / 2.0;

            // world_x should be view_x at off_x = 0
            let world_x = view_x + off_x * blocks_per_pixel * BLOCK_SIZE;
            let world_y = view_y + off_y * blocks_per_pixel * BLOCK_SIZE;

            let (cx, cy) = crate::utils::world_to_chunk_coords(world_x, world_y);

            if cx != last_cx || cy != last_cy {
                last_cx = cx;
                last_cy = cy;
                last_visited = game.world_manager.visited_chunks.contains(&(cx, cy));
                last_chunk = if last_visited {
                    game.world_manager.get_chunk(cx, cy)
                } else {
                    None
                };
            }

            // Only draw if visited
            let color = if last_visited {
                if let Some(chunk) = last_chunk {
                    let (rel_x, rel_y) =
                        crate::utils::world_to_relative_in_chunk_coords(world_x, world_y);
                    if rel_x < CHUNK_SIZE_X_BLOCKS && rel_y < CHUNK_SIZE_Y_BLOCKS {
                        let block = &chunk.blocks[rel_x][rel_y];
                        if block.is_broken {
                            Color::from_rgba(20, 20, 40, 255)
                        } else {
                            block.block_type.get_map_color()
                        }
                    } else {
                        BLACK
                    }
                } else {
                    Color::from_rgba(10, 10, 10, 255)
                }
            } else {
                BLACK // Fog of war
            };

            draw_rectangle(
                ctx.offset_x + px * ctx.scale,
                ctx.offset_y + py * ctx.scale,
                ctx.scale,
                ctx.scale,
                color,
            );
        }
    }

    // Draw player marker relative to the view
    let player_rel_x = (game.player_manager.player.x - view_x) / (BLOCK_SIZE * blocks_per_pixel);
    let player_rel_y = (game.player_manager.player.y - view_y) / (BLOCK_SIZE * blocks_per_pixel);

    draw_circle(
        ctx.offset_x + (SCREEN_WIDTH / 2.0 + player_rel_x) * ctx.scale,
        ctx.offset_y + (SCREEN_HEIGHT / 2.0 + player_rel_y) * ctx.scale,
        3.0 * ctx.scale,
        RED,
    );

    // 3. Draw Warp Gates
    for (idx, gate) in game.player_manager.player.warp_gates.iter().enumerate() {
        // Center of the block
        let gate_world_center_x = gate.x + BLOCK_SIZE / 2.0;
        let gate_world_center_y = gate.y + BLOCK_SIZE / 2.0;

        let gate_rel_x = (gate_world_center_x - view_x) / (BLOCK_SIZE * blocks_per_pixel);
        let gate_rel_y = (gate_world_center_y - view_y) / (BLOCK_SIZE * blocks_per_pixel);

        let gx = ctx.offset_x + (SCREEN_WIDTH / 2.0 + gate_rel_x) * ctx.scale;
        let gy = ctx.offset_y + (SCREEN_HEIGHT / 2.0 + gate_rel_y) * ctx.scale;

        // Always draw markers even if they are slightly off-screen to help navigation
        let margin = 20.0 * ctx.scale;
        if gx > ctx.offset_x - margin
            && gx < ctx.offset_x + screen_w + margin
            && gy > ctx.offset_y - margin
            && gy < ctx.offset_y + screen_h + margin
        {
            let time = get_time();
            let pulse = (time * 5.0).sin() as f32 * 0.5 + 0.5; // Faster pulse

            // Interaction Check
            let (rel_mx, rel_my) = crate::utils::get_game_mouse_position();
            let gate_gx = SCREEN_WIDTH / 2.0 + gate_rel_x;
            let gate_gy = SCREEN_HEIGHT / 2.0 + gate_rel_y;
            let dist_sq = (rel_mx - gate_gx).powi(2) + (rel_my - gate_gy).powi(2);
            let is_hovered =
                dist_sq < 8.0f32.powi(2) && game.pending_warp_index.is_none();

            // Scale marker size slightly with zoom
            let mut zoom_factor = (game.map_zoom.sqrt()).clamp(0.5, 2.0);
            if is_hovered {
                zoom_factor *= 1.3;
            }

            let base_size = 4.0 * ctx.scale * zoom_factor;
            let outer_size = base_size * (1.1 + pulse * 0.5);

            // 1. Draw outer glow/pulse
            let glow_color = if is_hovered {
                Color::from_rgba(255, 255, 0, (200.0 * (1.0 - pulse)) as u8) // Yellow glow when hovered
            } else {
                Color::from_rgba(255, 0, 255, (150.0 * (1.0 - pulse)) as u8)
            };
            draw_poly(gx, gy, 4, outer_size, 0.0, glow_color);

            // 2. Draw main diamond
            let main_color = if is_hovered { YELLOW } else { MAGENTA };
            draw_poly(gx, gy, 4, base_size, 0.0, main_color);
            draw_poly_lines(gx, gy, 4, base_size, 0.0, 1.5 * ctx.scale, WHITE);

            // Handle Click
            if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                game.pending_warp_index = Some(idx);
            }

            // 3. Draw name label
            if let Some(font) = ctx.font {
                let font_size = (ctx.font_size as f32 * 0.7) as u16;
                let text = &gate.name;
                let text_size = measure_text(text, Some(font), font_size, 1.0);

                let tx = gx - text_size.width / 2.0;
                let ty = gy - base_size - 4.0 * ctx.scale;

                // Label background
                draw_rectangle(
                    tx - 2.0,
                    ty - text_size.height,
                    text_size.width + 4.0,
                    text_size.height + 2.0,
                    Color::from_rgba(0, 0, 0, 180),
                );

                draw_text_ex(
                    text,
                    tx,
                    ty,
                    TextParams {
                        font: Some(font),
                        font_size,
                        color: YELLOW,
                        ..Default::default()
                    },
                );
            }
        }
    }

    // --- Confirmation Dialog ---
    if let Some(idx) = game.pending_warp_index {
        let gate_name = game.player_manager.player.warp_gates[idx].name.clone();
        let dialog_w = 100.0 * ctx.scale;
        let dialog_h = 50.0 * ctx.scale;
        let dx = ctx.offset_x + (screen_w - dialog_w) / 2.0;
        let dy = ctx.offset_y + (screen_h - dialog_h) / 2.0;

        // Dim background
        draw_rectangle(
            ctx.offset_x,
            ctx.offset_y,
            screen_w,
            screen_h,
            Color::from_rgba(0, 0, 0, 150),
        );

        // Dialog box
        draw_rectangle(
            dx,
            dy,
            dialog_w,
            dialog_h,
            Color::from_rgba(30, 30, 40, 255),
        );
        draw_rectangle_lines(dx, dy, dialog_w, dialog_h, 2.0 * ctx.scale, WHITE);

        if let Some(font) = ctx.font {
            let prompt = format!("Warp to {}?", gate_name);
            let prompt_size = measure_text(&prompt, Some(font), ctx.font_size, 1.0);
            draw_text_ex(
                &prompt,
                dx + (dialog_w - prompt_size.width) / 2.0,
                dy + 15.0 * ctx.scale,
                TextParams {
                    font: Some(font),
                    font_size: ctx.font_size,
                    color: WHITE,
                    ..Default::default()
                },
            );

            // YES Button
            use crate::render::ui::common::{ButtonParams, draw_button};
            let btn_w = 35.0 * ctx.scale;
            let btn_h = 12.0 * ctx.scale;

            if draw_button(
                ButtonParams {
                    x: dx + 10.0 * ctx.scale,
                    y: dy + 30.0 * ctx.scale,
                    w: btn_w,
                    h: btn_h,
                    text_key: "YES",
                    press_key: "Y",
                    lang: &game.lang_manager,
                    font_size: ctx.font_size,
                },
                Some(font),
            ) || is_key_pressed(KeyCode::Y)
            {
                ctx.events
                    .push(crate::events::GameEvent::TeleportToWarp(idx));
                game.pending_warp_index = None;
            }

            // NO Button
            if draw_button(
                ButtonParams {
                    x: dx + dialog_w - btn_w - 10.0 * ctx.scale,
                    y: dy + 30.0 * ctx.scale,
                    w: btn_w,
                    h: btn_h,
                    text_key: "NO",
                    press_key: "N",
                    lang: &game.lang_manager,
                    font_size: ctx.font_size,
                },
                Some(font),
            ) || is_key_pressed(KeyCode::N)
                || is_key_pressed(KeyCode::Escape)
            {
                game.pending_warp_index = None;
            }
        }
    }
    // ---------------------------

    // 4. Draw info and coordinates
    if let Some(font) = ctx.font {
        let text = "Map Mode - [M/ESC] Close - [+/-] Zoom - [WASD/Arrows] Pan";
        let instruction_font_size = (ctx.font_size as f32 * 0.7) as u16;
        let text_size = measure_text(text, Some(font), instruction_font_size, 1.0);
        if game.pending_warp_index.is_none() {
            draw_text_ex(
                text,
                ctx.offset_x + (screen_w - text_size.width) / 2.0,
                ctx.offset_y + screen_h - 15.0 * ctx.scale,
                TextParams {
                    font: Some(font),
                    font_size: instruction_font_size,
                    color: Color::from_rgba(200, 200, 200, 255),
                    ..Default::default()
                },
            );
        }

        let zoom_text = format!("Zoom: {:.2}x", game.map_zoom);
        draw_text_ex(
            &zoom_text,
            ctx.offset_x + 10.0 * ctx.scale,
            ctx.offset_y + 20.0 * ctx.scale,
            TextParams {
                font: Some(font),
                font_size: (ctx.font_size as f32 * 0.8) as u16,
                color: YELLOW,
                ..Default::default()
            },
        );

        // Player Coordinates
        let player_coord_text = format!(
            "Player: X:{:.0} Y:{:.0}",
            game.player_manager.player.x / BLOCK_SIZE,
            game.player_manager.player.y / BLOCK_SIZE
        );
        draw_text_ex(
            &player_coord_text,
            ctx.offset_x + 10.0 * ctx.scale,
            ctx.offset_y + 35.0 * ctx.scale,
            TextParams {
                font: Some(font),
                font_size: (ctx.font_size as f32 * 0.7) as u16,
                color: WHITE,
                ..Default::default()
            },
        );

        // Mouse World Coordinates
        let (rel_mx, rel_my) = crate::utils::get_game_mouse_position();

        if (0.0..=SCREEN_WIDTH).contains(&rel_mx) && (0.0..=SCREEN_HEIGHT).contains(&rel_my) {
            let m_off_x = rel_mx - SCREEN_WIDTH / 2.0;
            let m_off_y = rel_my - SCREEN_HEIGHT / 2.0;
            let m_world_x = view_x + m_off_x * blocks_per_pixel * BLOCK_SIZE;
            let m_world_y = view_y + m_off_y * blocks_per_pixel * BLOCK_SIZE;

            let mouse_coord_text = format!(
                "Cursor: X:{:.0} Y:{:.0}",
                m_world_x / BLOCK_SIZE,
                m_world_y / BLOCK_SIZE
            );
            let m_text_measure = measure_text(
                &mouse_coord_text,
                Some(font),
                (ctx.font_size as f32 * 0.7) as u16,
                1.0,
            );

            draw_text_ex(
                &mouse_coord_text,
                ctx.offset_x + screen_w - m_text_measure.width - 10.0 * ctx.scale,
                ctx.offset_y + 20.0 * ctx.scale,
                TextParams {
                    font: Some(font),
                    font_size: (ctx.font_size as f32 * 0.7) as u16,
                    color: GREEN,
                    ..Default::default()
                },
            );
        }
    }
}
