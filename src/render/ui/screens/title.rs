use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_title_screen(game: &Game, ctx: &mut MenuRenderContext) {
    let title = "Digging Game";
    let center_x = screen_width() / 2.0;
    let center_y = screen_height() / 2.0;

    let params = TextParams {
        font_size: (FONT_SIZE * ctx.scale * 2.0) as u16,
        font: ctx.font,
        color: WHITE,
        ..Default::default()
    };
    let t_measure = measure_text(title, ctx.font, params.font_size, 1.0);
    draw_text_ex(
        title,
        (center_x - t_measure.width / 2.0).floor(),
        (center_y * 0.5).floor(),
        params,
    );

    let bw = 60.0 * ctx.scale;
    let bh = 10.0 * ctx.scale;
    let bx = (screen_width() - bw) / 2.0;
    let by = center_y;

    let s_font_size = (FONT_SIZE * ctx.scale).floor() as u16;

    if draw_button(
        ButtonParams {
            x: bx,
            y: by,
            w: bw,
            h: bh,
            text_key: "button.title_screen.start.default",
            press_key: "button.title_screen.start.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::OpenSaveSelection);
    }
    if draw_button(
        ButtonParams {
            x: bx,
            y: by + 15.0 * ctx.scale,
            w: bw,
            h: bh,
            text_key: "button.menu.quit.default",
            press_key: "button.menu.quit.pressed",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::QuitGame);
    }
}
