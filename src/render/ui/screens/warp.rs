use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_warp_place_screen(game: &Game, ctx: &mut MenuRenderContext) {
    draw_text_ex(
        &game.lang_manager.get_string("warp.name_prompt"),
        ctx.offset_x + 10.0 * ctx.scale,
        ctx.offset_y + 30.0 * ctx.scale,
        TextParams {
            font_size: ctx.font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );
    draw_rectangle(
        ctx.offset_x + 10.0 * ctx.scale,
        ctx.offset_y + 40.0 * ctx.scale,
        (SCREEN_WIDTH - 20.0) * ctx.scale,
        12.0 * ctx.scale,
        DARKGRAY,
    );
    let cur = if (get_time() * 2.0) as i32 % 2 == 0 {
        "|"
    } else {
        ""
    };
    draw_text_ex(
        &format!("{}{}", game.input_buffer, cur),
        ctx.offset_x + 12.0 * ctx.scale,
        ctx.offset_y + 49.0 * ctx.scale,
        TextParams {
            font_size: ctx.font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );
    if draw_button(
        ButtonParams {
            x: ctx.offset_x + 10.0 * ctx.scale,
            y: ctx.offset_y + 60.0 * ctx.scale,
            w: 60.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.confirm",
            press_key: "button.confirm",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events
            .push(GameEvent::ConfirmWarpGateName(game.input_buffer.clone()));
    }
}

pub fn draw_warp_select_screen(game: &Game, ctx: &mut MenuRenderContext) {
    draw_text_ex(
        &game.lang_manager.get_string("warp.title"),
        ctx.offset_x + 10.0 * ctx.scale,
        ctx.offset_y + 20.0 * ctx.scale,
        TextParams {
            font_size: ctx.font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );

    if draw_button(
        ButtonParams {
            x: ctx.offset_x + 2.0 * ctx.scale,
            y: ctx.offset_y + 2.0 * ctx.scale,
            w: 30.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "shop.back_to_game",
            press_key: "shop.back_to_game",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::CloseMenu);
    }

    let mut cy = ctx.offset_y + 35.0 * ctx.scale;
    for (i, gate) in game.player_manager.player.warp_gates.iter().enumerate() {
        let label = format!("> {}", gate.name);
        if draw_button(
            ButtonParams {
                x: ctx.offset_x + 10.0 * ctx.scale,
                y: cy,
                w: (SCREEN_WIDTH - 20.0) * ctx.scale,
                h: 10.0 * ctx.scale,
                text_key: &label,
                press_key: &label,
                lang: &game.lang_manager,
                font_size: ctx.font_size,
            },
            ctx.font,
        ) {
            ctx.events.push(GameEvent::TeleportToWarp(i));
        }
        cy += 12.0 * ctx.scale;
    }
}
