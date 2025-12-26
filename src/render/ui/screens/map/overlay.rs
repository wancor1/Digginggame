use crate::Game;
use crate::constants::{BLOCK_SIZE, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_overlay_ui(
    game: &Game,
    ctx: &MenuRenderContext,
    view_x: f32,
    view_y: f32,
    blocks_per_pixel: f32,
) {
    let screen_w = SCREEN_WIDTH * ctx.scale;
    let screen_h = SCREEN_HEIGHT * ctx.scale;

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

pub fn draw_confirmation_dialog(game: &mut Game, ctx: &mut MenuRenderContext) {
    let screen_w = SCREEN_WIDTH * ctx.scale;
    let screen_h = SCREEN_HEIGHT * ctx.scale;

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
            let prompt = format!("Warp to {gate_name}?");
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
}
