use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_pause_menu(game: &Game, ctx: &mut MenuRenderContext) {
    let (mw, mh) = (80.0 * ctx.scale, 75.0 * ctx.scale);
    let (mx, my) = (
        ctx.offset_x + ((SCREEN_WIDTH - 80.0) / 2.0).floor() * ctx.scale,
        ctx.offset_y + ((SCREEN_HEIGHT - 75.0) / 2.0).floor() * ctx.scale,
    );
    draw_rectangle(mx, my, mw, mh, LIGHTGRAY);
    draw_rectangle_lines(mx, my, mw, mh, 1.0, BLACK);

    let mut cur_y = (my + 10.0 * ctx.scale).floor();
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.menu.return.default",
            press_key: "button.menu.return.pressed",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::CloseMenu);
    }
    cur_y += 15.0 * ctx.scale;
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.menu.save.default",
            press_key: "button.menu.save.pressed",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::SaveGame);
    }
    cur_y += 12.0 * ctx.scale;
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.menu.respawn.default",
            press_key: "button.menu.respawn.pressed",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::Respawn);
    }
    cur_y += 12.0 * ctx.scale;
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.menu.quit_to_title.default",
            press_key: "button.menu.quit_to_title.pressed",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::ReturnToTitle);
    }
}
