use crate::Game;
use crate::constants::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;
use std::collections::HashMap;

pub fn draw_warehouse(game: &mut Game, ctx: &mut MenuRenderContext) {
    let (mw, mh) = (
        (SCREEN_WIDTH - 10.0) * ctx.scale,
        (SCREEN_HEIGHT - 20.0) * ctx.scale,
    );
    let (mx, my) = (
        ctx.offset_x + 5.0 * ctx.scale,
        ctx.offset_y + 10.0 * ctx.scale,
    );
    draw_rectangle(mx, my, mw, mh, Color::new(0.1, 0.1, 0.2, 0.95));
    draw_rectangle_lines(mx, my, mw, mh, 1.0, WHITE);

    let mini_font_size = (5.0 * ctx.scale) as u16;
    let mut cur_y = (my + 5.0 * ctx.scale).floor();

    if draw_button(
        ButtonParams {
            x: mx + mw - 32.0 * ctx.scale,
            y: cur_y,
            w: 28.0 * ctx.scale,
            h: 8.0 * ctx.scale,
            text_key: "shop.back_to_game",
            press_key: "shop.back_to_game",
            lang: &game.lang_manager,
            font_size: mini_font_size,
        },
        ctx.font,
    ) {
        ctx.events.push(GameEvent::CloseMenu);
    }

    draw_text_ex(
        &game.lang_manager.get_string("warehouse.title"),
        mx + 5.0 * ctx.scale,
        cur_y + 6.0 * ctx.scale,
        TextParams {
            font_size: ctx.font_size,
            font: ctx.font,
            color: YELLOW,
            ..Default::default()
        },
    );

    // Quantity toggle button
    let q_label = match game.warehouse_quantity {
        10 => "10",
        100 => "100",
        0 => "ALL",
        _ => "1",
    };
    let qty_btn_text = game
        .lang_manager
        .get_string(&format!("warehouse.qty (qty: {q_label})")); // Simplified for brevity as _get_string_fmt was used

    if draw_button(
        ButtonParams {
            x: mx + mw - 70.0 * ctx.scale,
            y: cur_y,
            w: 35.0 * ctx.scale,
            h: 8.0 * ctx.scale,
            text_key: &qty_btn_text,
            press_key: &qty_btn_text,
            lang: &game.lang_manager,
            font_size: mini_font_size - 1,
        },
        ctx.font,
    ) {
        game.warehouse_quantity = match game.warehouse_quantity {
            1 => 10,
            10 => 100,
            100 => 0,
            _ => 1,
        };
    }

    cur_y += 15.0 * ctx.scale;

    let player = &game.player_manager.player;
    let current_q = game.warehouse_quantity;

    // Left: Inventory (Cargo)
    let inv_x = mx + 5.0 * ctx.scale;
    let cargo_header = format!(
        "{}: {}/{}",
        game.lang_manager.get_string("warehouse.cargo"),
        player.total_cargo_weight(),
        player.max_cargo
    );
    draw_text_ex(
        &cargo_header,
        inv_x,
        cur_y,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );

    let mut item_y = cur_y + 8.0 * ctx.scale;
    let mut cargo_counts = HashMap::new();
    for item in &player.cargo {
        *cargo_counts.entry(&item.item_type).or_insert(0) += 1;
    }
    let mut cargo_types: Vec<_> = cargo_counts.keys().collect();
    cargo_types.sort();

    for it in cargo_types {
        let count = cargo_counts[it];
        let label = format!(
            "{}: {}",
            game.lang_manager.get_string(&format!("block.{it}.name")),
            count
        );
        draw_text_ex(
            label.as_str(),
            inv_x,
            item_y + 6.0 * ctx.scale,
            TextParams {
                font_size: mini_font_size,
                font: ctx.font,
                color: WHITE,
                ..Default::default()
            },
        );

        if draw_button(
            ButtonParams {
                x: inv_x + 38.0 * ctx.scale,
                y: item_y,
                w: 24.0 * ctx.scale,
                h: 7.0 * ctx.scale,
                text_key: "warehouse.store",
                press_key: "warehouse.store",
                lang: &game.lang_manager,
                font_size: mini_font_size - 1,
            },
            ctx.font,
        ) {
            let actual_q = if current_q == 0 {
                count
            } else {
                current_q.min(count)
            };
            ctx.events
                .push(GameEvent::DepositItem((*it).clone(), actual_q));
        }
        item_y += 9.0 * ctx.scale;
    }

    // Right: Storage
    let stor_x = mx + mw / 2.0 + 3.0 * ctx.scale;
    let storage_header = format!(
        "{}: {}/{}",
        game.lang_manager.get_string("warehouse.cargo"),
        player.storage.len(),
        player.max_storage
    );
    draw_text_ex(
        &storage_header,
        stor_x,
        cur_y,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );

    let mut stor_y = cur_y + 8.0 * ctx.scale;
    let mut stor_counts = HashMap::new();
    for item in &player.storage {
        *stor_counts.entry(&item.item_type).or_insert(0) += 1;
    }
    let mut stor_types: Vec<_> = stor_counts.keys().collect();
    stor_types.sort();

    for it in stor_types {
        let count = stor_counts[it];
        let label = format!(
            "{}: {}",
            game.lang_manager.get_string(&format!("block.{it}.name")),
            count
        );
        draw_text_ex(
            label.as_str(),
            stor_x,
            stor_y + 6.0 * ctx.scale,
            TextParams {
                font_size: mini_font_size,
                font: ctx.font,
                color: WHITE,
                ..Default::default()
            },
        );

        let mut bx = stor_x + 28.0 * ctx.scale;
        if draw_button(
            ButtonParams {
                x: bx,
                y: stor_y,
                w: 20.0 * ctx.scale,
                h: 7.0 * ctx.scale,
                text_key: "warehouse.take",
                press_key: "warehouse.take",
                lang: &game.lang_manager,
                font_size: mini_font_size - 1,
            },
            ctx.font,
        ) {
            let actual_q = if current_q == 0 {
                count
            } else {
                current_q.min(count)
            };
            ctx.events
                .push(GameEvent::WithdrawItem((*it).clone(), actual_q));
        }
        bx += 22.0 * ctx.scale;
        if draw_button(
            ButtonParams {
                x: bx,
                y: stor_y,
                w: 18.0 * ctx.scale,
                h: 7.0 * ctx.scale,
                text_key: "warehouse.sell",
                press_key: "warehouse.sell",
                lang: &game.lang_manager,
                font_size: mini_font_size - 1,
            },
            ctx.font,
        ) {
            let actual_q = if current_q == 0 {
                count
            } else {
                current_q.min(count)
            };
            ctx.events
                .push(GameEvent::SellItem((*it).clone(), actual_q));
        }
        stor_y += 9.0 * ctx.scale;
    }
}
