use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_save_select_screen(game: &Game, ctx: &mut MenuRenderContext) {
    draw_text_ex(
        &game.lang_manager.get_string("menu.select_save"),
        ctx.offset_x + 10.0 * ctx.scale,
        ctx.offset_y + 20.0 * ctx.scale,
        TextParams {
            font_size: ctx.font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );
    let mut cy = ctx.offset_y + 35.0 * ctx.scale;
    if draw_button(
        ButtonParams {
            x: ctx.offset_x + 10.0 * ctx.scale,
            y: cy,
            w: (SCREEN_WIDTH - 20.0) * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.menu.new_game.default",
            press_key: "button.menu.new_game.pressed",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::StartNewGameSetup);
    }
    if draw_button(
        ButtonParams {
            x: ctx.offset_x + 2.0 * ctx.scale,
            y: ctx.offset_y + 2.0 * ctx.scale,
            w: 30.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: "button.menu.return.default",
            press_key: "button.menu.return.pressed",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::ReturnToTitleFromSaveSelect);
    }
    cy += 15.0 * ctx.scale;
    for file in &game.save_files {
        if draw_button(
            ButtonParams {
                x: ctx.offset_x + 10.0 * ctx.scale,
                y: cy,
                w: (SCREEN_WIDTH - 20.0) * ctx.scale,
                h: 10.0 * ctx.scale,
                text_key: file,
                press_key: file,
                lang: &game.lang_manager,
                font_size: ctx.font_size,
            },
            ctx.font,
        ) {
            ctx.events.push(GameEvent::LoadSave(file.clone()));
        }
        cy += 12.0 * ctx.scale;
    }
}
