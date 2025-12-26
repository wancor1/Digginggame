use crate::Game;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_shop(game: &Game, ctx: &mut MenuRenderContext) {
    let (mw, mh) = (110.0 * ctx.scale, (SCREEN_HEIGHT - 20.0) * ctx.scale);
    let (mx, my) = (
        ctx.offset_x + ((SCREEN_WIDTH - 110.0) / 2.0).floor() * ctx.scale,
        ctx.offset_y + 10.0 * ctx.scale,
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
            text_key: "shop.back_to_game",
            press_key: "shop.back_to_game",
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::CloseMenu);
    }

    cur_y += 15.0 * ctx.scale;
    let mini_font_size = (6.0 * ctx.scale) as u16;
    draw_text_ex(
        &game.lang_manager.get_string("shop.title"),
        mx + 10.0 * ctx.scale,
        cur_y + 8.0 * ctx.scale,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: BLACK,
            ..Default::default()
        },
    );
    cur_y += 12.0 * ctx.scale;

    let dc = game.player_manager.player.drill_level * 100;
    let drill_name = game.lang_manager.get_string("shop.upgrade.drill");
    let drill_label = format!(
        "{drill_name} Lv{} (${dc})",
        game.player_manager.player.drill_level
    );
    let purchase_label = game.lang_manager.get_string("shop.purchase");
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: &drill_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::UpgradeDrill);
    }
    cur_y += 12.0 * ctx.scale;

    let tc = game.player_manager.player.tank_level * 80;
    let tank_name = game.lang_manager.get_string("shop.upgrade.tank");
    let tank_label = format!(
        "{tank_name} Lv{} (${tc})",
        game.player_manager.player.tank_level
    );
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: &tank_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::UpgradeTank);
    }
    cur_y += 12.0 * ctx.scale;

    let ec = game.player_manager.player.engine_level * 120;
    let engine_name = game.lang_manager.get_string("shop.upgrade.engine");
    let engine_label = format!(
        "{engine_name} Lv{} (${ec})",
        game.player_manager.player.engine_level
    );
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: &engine_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::UpgradeEngine);
    }
    cur_y += 12.0 * ctx.scale;

    let cc = game.player_manager.player.cargo_level * 150;
    let cargo_name = game.lang_manager.get_string("shop.upgrade.cargo");
    let cargo_label = format!(
        "{cargo_name} Lv{} (${cc})",
        game.player_manager.player.cargo_level
    );
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: &cargo_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::UpgradeCargo);
    }
    cur_y += 12.0 * ctx.scale;

    let hrc = game.player_manager.player.heat_resistance_level * 200;
    let heat_name = game.lang_manager.get_string("shop.upgrade.heat_res");
    let heat_label = format!(
        "{heat_name} Lv{} (${hrc})",
        game.player_manager.player.heat_resistance_level
    );
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: &heat_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::UpgradeHeatResistance);
    }
    cur_y += 12.0 * ctx.scale;

    let wg_name = game.lang_manager.get_string("shop.buy.warpgate");
    let wg_label = format!("{wg_name} ($500)");
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * ctx.scale,
            y: cur_y,
            w: mw - 10.0 * ctx.scale,
            h: 10.0 * ctx.scale,
            text_key: &wg_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: ctx.font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::BuyWarpGate);
    }
}
