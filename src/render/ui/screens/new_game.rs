use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_new_game_input_screen(game: &Game, ctx: &mut MenuRenderContext) {
    draw_text_ex(
        &game.lang_manager.get_string("menu.enter_filename"),
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
            .push(GameEvent::ConfirmNewGame(game.input_buffer.clone()));
    }
}
