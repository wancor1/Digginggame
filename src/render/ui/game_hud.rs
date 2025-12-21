use super::common::{ButtonParams, draw_button};
use crate::Game;
use crate::components::BlockType; // Added import
use crate::constants::*;
use crate::events::GameEvent;
use macroquad::prelude::*;

pub fn draw_hud(
    game: &Game,
    font: Option<&Font>,
    atlas: Option<&Texture2D>,
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

    // --- SELECTED ITEM ---
    let slot_size = 12.0 * scale;
    let sel_x = hud_x;
    let sel_y = hud_y + 15.0 * scale;

    draw_rectangle(
        sel_x,
        sel_y,
        slot_size,
        slot_size,
        Color::new(0.3, 0.3, 0.3, 0.8),
    );
    draw_rectangle_lines(sel_x, sel_y, slot_size, slot_size, 1.0, GRAY);

    if let Some(item) = player.cargo.get(game.selected_item_index) {
        if let Some(atlas_tex) = atlas {
            let sprite_rect = crate::utils::get_item_sprite(&item.item_type);
            if sprite_rect.w > 0.0 {
                draw_texture_ex(
                    atlas_tex,
                    sel_x + 2.0 * scale,
                    sel_y + 2.0 * scale,
                    WHITE,
                    DrawTextureParams {
                        source: Some(sprite_rect),
                        dest_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                        ..Default::default()
                    },
                );
            }
        }
        draw_text_ex(
            "SELECTED",
            sel_x + slot_size + 2.0 * scale,
            sel_y + 4.0 * scale,
            TextParams {
                font_size: (4.0 * scale) as u16,
                font,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
        draw_text_ex(
            &item.item_type,
            sel_x + slot_size + 2.0 * scale,
            sel_y + 10.0 * scale,
            TextParams {
                font_size: mini_font_size,
                font,
                color: WHITE,
                ..Default::default()
            },
        );
    }
    // --------------

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
    events: &mut Vec<GameEvent>,
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

    // Count items and track first index
    use std::collections::HashMap;
    let mut counts = HashMap::new();
    let mut first_indices = HashMap::new();
    for (idx, item) in player.cargo.iter().enumerate() {
        *counts.entry(&item.item_type).or_insert(0) += 1;
        first_indices.entry(&item.item_type).or_insert(idx);
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
            cur_y + 6.0 * scale,
            TextParams {
                font_size: mini_font_size,
                font,
                color: WHITE,
                ..Default::default()
            },
        );

        let is_placeable = BlockType::from_item_type(it).map_or(false, |bt| bt.is_placeable());
        if is_placeable {
            if draw_button(
                ButtonParams {
                    x: mx + mw - 35.0 * scale,
                    y: cur_y,
                    w: 30.0 * scale,
                    h: 8.0 * scale,
                    text_key: "SELECT",
                    press_key: "SELECT",
                    lang: &game.lang_manager,
                    font_size: mini_font_size - 1,
                },
                font,
            ) {
                if let Some(&idx) = first_indices.get(it) {
                    events.push(GameEvent::SetSelectedItemIndex(idx));
                }
            }
        }
        cur_y += 10.0 * scale;
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
        (SCREEN_WIDTH - 10.0) * scale,
        (SCREEN_HEIGHT - 20.0) * scale,
    );
    let (mx, my) = (offset_x + 5.0 * scale, offset_y + 10.0 * scale);
    draw_rectangle(mx, my, mw, mh, Color::new(0.1, 0.1, 0.2, 0.95));
    draw_rectangle_lines(mx, my, mw, mh, 1.0, WHITE);

    let mini_font_size = (5.0 * scale) as u16;
    let mut cur_y = (my + 5.0 * scale).floor();

    if draw_button(
        ButtonParams {
            x: mx + mw - 32.0 * scale,
            y: cur_y,
            w: 28.0 * scale,
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
        &game.lang_manager.get_string("warehouse.title"),
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
        1 => "1",
        10 => "10",
        100 => "100",
        0 => "ALL",
        _ => "1",
    };
    let qty_btn_text = game
        .lang_manager
        ._get_string_fmt("warehouse.qty", &[("qty", q_label)]);

    if draw_button(
        ButtonParams {
            x: mx + mw - 70.0 * scale,
            y: cur_y,
            w: 35.0 * scale,
            h: 8.0 * scale,
            text_key: &qty_btn_text,
            press_key: &qty_btn_text,
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

    // Left: Inventory (Cargo)
    let inv_x = mx + 5.0 * scale;
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
                x: inv_x + 38.0 * scale,
                y: item_y,
                w: 24.0 * scale,
                h: 7.0 * scale,
                text_key: "warehouse.store",
                press_key: "warehouse.store",
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

    // Right: Storage
    let stor_x = mx + mw / 2.0 + 3.0 * scale;
    let storage_header = format!(
        "{}: {}/{}",
        game.lang_manager.get_string("warehouse.storage"),
        player.storage.len(),
        player.max_storage
    );
    draw_text_ex(
        &storage_header,
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

        let mut bx = stor_x + 28.0 * scale;
        if draw_button(
            ButtonParams {
                x: bx,
                y: stor_y,
                w: 20.0 * scale,
                h: 7.0 * scale,
                text_key: "warehouse.take",
                press_key: "warehouse.take",
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
        bx += 22.0 * scale;
        if draw_button(
            ButtonParams {
                x: bx,
                y: stor_y,
                w: 18.0 * scale,
                h: 7.0 * scale,
                text_key: "warehouse.sell",
                press_key: "warehouse.sell",
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
