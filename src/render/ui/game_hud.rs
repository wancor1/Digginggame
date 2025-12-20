use super::common::{ButtonParams, draw_button};
use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use macroquad::prelude::*;

pub fn draw_hud(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    events: &mut Vec<GameEvent>,
) {
    let player = &game.player_manager.player;
    let hud_y = offset_y + 5.0 * scale;
    let hud_x = offset_x + 5.0 * scale;
    let mini_font_size = (6.0 * scale) as u16;
    let s_font_size = (FONT_SIZE * scale).floor() as u16;

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
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!(
            "CARGO: {}/{}",
            player.total_cargo_weight(),
            player.max_cargo
        ),
        hud_x + 50.0 * scale,
        hud_y + 4.0 * scale,
        TextParams {
            font_size: mini_font_size,
            font,
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
            font,
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
            font,
            color: WHITE,
            ..Default::default()
        },
    );

    if game.on_surface {
        if !game.is_menu_visible && !game.is_shop_open && !game.is_warehouse_open {
            if draw_button(
                ButtonParams {
                    x: offset_x + (SCREEN_WIDTH - 40.0) * scale,
                    y: offset_y + 25.0 * scale,
                    w: 35.0 * scale,
                    h: 10.0 * scale,
                    text_key: "SHOP",
                    press_key: "SHOP",
                    lang: &game.lang_manager,
                    font_size: s_font_size,
                },
                font,
            ) {
                events.push(GameEvent::OpenShop);
            }
            if draw_button(
                ButtonParams {
                    x: offset_x + (SCREEN_WIDTH - 40.0) * scale,
                    y: offset_y + 37.0 * scale,
                    w: 35.0 * scale,
                    h: 10.0 * scale,
                    text_key: "WAREHOUSE",
                    press_key: "WAREHOUSE",
                    lang: &game.lang_manager,
                    font_size: s_font_size,
                },
                font,
            ) {
                events.push(GameEvent::OpenWarehouse);
            }
            if draw_button(
                ButtonParams {
                    x: offset_x + 5.0 * scale,
                    y: offset_y + 25.0 * scale,
                    w: 35.0 * scale,
                    h: 10.0 * scale,
                    text_key: "hud.warp_menu",
                    press_key: "hud.warp_menu.pressed",
                    lang: &game.lang_manager,
                    font_size: s_font_size,
                },
                font,
            ) {
                events.push(GameEvent::OpenWarpMenu);
            }
        }
    } else if game.player_manager.player.inventory_warp_gates > 0 {
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
                font,
                color: WHITE,
                ..Default::default()
            },
        );
        if draw_button(
            ButtonParams {
                x: offset_x + 5.0 * scale,
                y: offset_y + 35.0 * scale,
                w: 60.0 * scale,
                h: 10.0 * scale,
                text_key: "hud.place_gate",
                press_key: "hud.place_gate.pressed",
                lang: &game.lang_manager,
                font_size: s_font_size,
            },
            font,
        ) {
            events.push(GameEvent::StartPlaceWarpGate);
        }
    }
}

pub fn draw_shop(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    let (mw, mh) = (110.0 * scale, (SCREEN_HEIGHT - 20.0) * scale);
    let (mx, my) = (
        offset_x + ((SCREEN_WIDTH - 110.0) / 2.0).floor() * scale,
        offset_y + 10.0 * scale,
    );
    draw_rectangle(mx, my, mw, mh, LIGHTGRAY);
    draw_rectangle_lines(mx, my, mw, mh, 1.0, BLACK);

    let mut cur_y = (my + 10.0 * scale).floor();
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: "shop.back_to_game",
            press_key: "shop.back_to_game",
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::CloseMenu);
    }

    cur_y += 15.0 * scale;
    let mini_font_size = (6.0 * scale) as u16;
    draw_text_ex(
        &game.lang_manager.get_string("shop.title"),
        mx + 10.0 * scale,
        cur_y + 8.0 * scale,
        TextParams {
            font_size: mini_font_size,
            font,
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
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: &drill_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
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
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: &tank_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
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
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: &engine_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
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
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: &cargo_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::UpgradeCargo);
    }
    cur_y += 12.0 * scale;

    let wg_name = game.lang_manager.get_string("shop.buy.warpgate");
    let wg_label = format!("{} ($500)", wg_name);
    if draw_button(
        ButtonParams {
            x: mx + 5.0 * scale,
            y: cur_y,
            w: mw - 10.0 * scale,
            h: 10.0 * scale,
            text_key: &wg_label,
            press_key: &purchase_label,
            lang: &game.lang_manager,
            font_size: s_font_size,
        },
        font,
    ) {
        events.push(GameEvent::BuyWarpGate);
    }
}

pub fn draw_inventory(
    game: &Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    _events: &mut Vec<GameEvent>,
) {
    let (mw, mh) = (110.0 * scale, (SCREEN_HEIGHT - 20.0) * scale);
    let (mx, my) = (
        offset_x + ((SCREEN_WIDTH - 110.0) / 2.0).floor() * scale,
        offset_y + 10.0 * scale,
    );
    draw_rectangle(mx, my, mw, mh, Color::new(0.2, 0.2, 0.2, 0.9));
    draw_rectangle_lines(mx, my, mw, mh, 1.0, WHITE);

    let mini_font_size = (6.0 * scale) as u16;
    let mut cur_y = (my + 10.0 * scale).floor();

    draw_text_ex(
        &game.lang_manager.get_string("inventory.title"),
        mx + 10.0 * scale,
        cur_y,
        TextParams {
            font_size: s_font_size,
            font,
            color: YELLOW,
            ..Default::default()
        },
    );
    cur_y += 12.0 * scale;

    let player = &game.player_manager.player;
    draw_text_ex(
        &format!(
            "Weight: {}/{}",
            player.total_cargo_weight(),
            player.max_cargo
        ),
        mx + 10.0 * scale,
        cur_y,
        TextParams {
            font_size: mini_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );
    cur_y += 10.0 * scale;

    // Count items
    use std::collections::HashMap;
    let mut counts = HashMap::new();
    for item in &player.cargo {
        *counts.entry(&item.item_type).or_insert(0) += 1;
    }

    let mut item_types: Vec<_> = counts.keys().collect();
    item_types.sort();

    for it in item_types {
        let count = counts[it];
        let weight = crate::utils::get_item_weight(it) * count;
        let label = format!("{}: {} ({}kg)", it, count, weight);
        draw_text_ex(
            &label,
            mx + 10.0 * scale,
            cur_y,
            TextParams {
                font_size: mini_font_size,
                font,
                color: WHITE,
                ..Default::default()
            },
        );
        cur_y += 8.0 * scale;
    }

    if player.cargo.is_empty() {
        draw_text_ex(
            "Empty",
            mx + 10.0 * scale,
            cur_y,
            TextParams {
                font_size: mini_font_size,
                font,
                color: GRAY,
                ..Default::default()
            },
        );
    }
}

pub fn draw_warehouse(
    game: &mut Game,
    font: Option<&Font>,
    scale: f32,
    offset_x: f32,
    offset_y: f32,
    s_font_size: u16,
    events: &mut Vec<GameEvent>,
) {
    let (mw, mh) = (
        (SCREEN_WIDTH - 20.0) * scale,
        (SCREEN_HEIGHT - 20.0) * scale,
    );
    let (mx, my) = (offset_x + 10.0 * scale, offset_y + 10.0 * scale);
    draw_rectangle(mx, my, mw, mh, Color::new(0.1, 0.1, 0.2, 0.95));
    draw_rectangle_lines(mx, my, mw, mh, 1.0, WHITE);

    let mini_font_size = (5.0 * scale) as u16;
    let mut cur_y = (my + 5.0 * scale).floor();

    if draw_button(
        ButtonParams {
            x: mx + mw - 35.0 * scale,
            y: cur_y,
            w: 30.0 * scale,
            h: 8.0 * scale,
            text_key: "shop.back_to_game",
            press_key: "shop.back_to_game",
            lang: &game.lang_manager,
            font_size: mini_font_size,
        },
        font,
    ) {
        events.push(GameEvent::CloseMenu);
    }

    draw_text_ex(
        "WAREHOUSE",
        mx + 5.0 * scale,
        cur_y + 6.0 * scale,
        TextParams {
            font_size: s_font_size,
            font,
            color: YELLOW,
            ..Default::default()
        },
    );

    // Quantity toggle button
    let q_label = match game.warehouse_quantity {
        1 => "x1",
        10 => "x10",
        100 => "x100",
        0 => "ALL",
        _ => "x1",
    };

    if draw_button(
        ButtonParams {
            x: mx + mw - 75.0 * scale,
            y: cur_y,
            w: 35.0 * scale,
            h: 8.0 * scale,
            text_key: &format!("Qty: {}", q_label),
            press_key: &format!("Qty: {}", q_label),
            lang: &game.lang_manager,
            font_size: mini_font_size - 1,
        },
        font,
    ) {
        game.warehouse_quantity = match game.warehouse_quantity {
            1 => 10,
            10 => 100,
            100 => 0,
            _ => 1,
        };
    }

    cur_y += 15.0 * scale;

    let player = &game.player_manager.player;
    let current_q = game.warehouse_quantity;

    // Left: Inventory (Cargo) - Narrower
    let inv_x = mx + 5.0 * scale;
    draw_text_ex(
        &format!(
            "Cargo ({}/{})",
            player.total_cargo_weight(),
            player.max_cargo
        ),
        inv_x,
        cur_y,
        TextParams {
            font_size: mini_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );

    let mut item_y = cur_y + 8.0 * scale;
    use std::collections::HashMap;
    let mut cargo_counts = HashMap::new();
    for item in &player.cargo {
        *cargo_counts.entry(&item.item_type).or_insert(0) += 1;
    }
    let mut cargo_types: Vec<_> = cargo_counts.keys().collect();
    cargo_types.sort();

    for it in cargo_types {
        let count = cargo_counts[it];
        let label = format!("{}: {}", it, count);
        draw_text_ex(
            label.as_str(),
            inv_x,
            item_y + 6.0 * scale,
            TextParams {
                font_size: mini_font_size,
                font,
                color: WHITE,
                ..Default::default()
            },
        );

        if draw_button(
            ButtonParams {
                x: inv_x + 35.0 * scale,
                y: item_y,
                w: 22.0 * scale,
                h: 7.0 * scale,
                text_key: "STORE",
                press_key: "STORE",
                lang: &game.lang_manager,
                font_size: mini_font_size - 1,
            },
            font,
        ) {
            let actual_q = if current_q == 0 {
                count
            } else {
                current_q.min(count)
            };
            events.push(GameEvent::DepositItem((*it).clone(), actual_q));
        }
        item_y += 9.0 * scale;
    }

    // Right: Storage - Expanded
    let stor_x = mx + 70.0 * scale;
    draw_text_ex(
        &format!("Storage ({}/{})", player.storage.len(), player.max_storage),
        stor_x,
        cur_y,
        TextParams {
            font_size: mini_font_size,
            font,
            color: WHITE,
            ..Default::default()
        },
    );

    let mut stor_y = cur_y + 8.0 * scale;
    let mut stor_counts = HashMap::new();
    for item in &player.storage {
        *stor_counts.entry(&item.item_type).or_insert(0) += 1;
    }
    let mut stor_types: Vec<_> = stor_counts.keys().collect();
    stor_types.sort();

    for it in stor_types {
        let count = stor_counts[it];
        let label = format!("{}: {}", it, count);
        draw_text_ex(
            label.as_str(),
            stor_x,
            stor_y + 6.0 * scale,
            TextParams {
                font_size: mini_font_size,
                font,
                color: WHITE,
                ..Default::default()
            },
        );

        let mut bx = stor_x + 35.0 * scale;
        if draw_button(
            ButtonParams {
                x: bx,
                y: stor_y,
                w: 22.0 * scale,
                h: 7.0 * scale,
                text_key: "TAKE",
                press_key: "TAKE",
                lang: &game.lang_manager,
                font_size: mini_font_size - 1,
            },
            font,
        ) {
            let actual_q = if current_q == 0 {
                count
            } else {
                current_q.min(count)
            };
            events.push(GameEvent::WithdrawItem((*it).clone(), actual_q));
        }
        bx += 25.0 * scale;
        if draw_button(
            ButtonParams {
                x: bx,
                y: stor_y,
                w: 20.0 * scale,
                h: 7.0 * scale,
                text_key: "SELL",
                press_key: "SELL",
                lang: &game.lang_manager,
                font_size: mini_font_size - 1,
            },
            font,
        ) {
            let actual_q = if current_q == 0 {
                count
            } else {
                current_q.min(count)
            };
            events.push(GameEvent::SellItem((*it).clone(), actual_q));
        }
        stor_y += 9.0 * scale;
    }
}
