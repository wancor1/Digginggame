use crate::Game;
use crate::components::BlockType;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;
use std::collections::HashMap;

pub fn draw_inventory(game: &Game, ctx: &mut MenuRenderContext) {
    let (mw, mh) = (110.0 * ctx.scale, (SCREEN_HEIGHT - 20.0) * ctx.scale);
    let (mx, my) = (
        ctx.offset_x + ((SCREEN_WIDTH - 110.0) / 2.0).floor() * ctx.scale,
        ctx.offset_y + 10.0 * ctx.scale,
    );
    draw_rectangle(mx, my, mw, mh, Color::new(0.2, 0.2, 0.2, 0.9));
    draw_rectangle_lines(mx, my, mw, mh, 1.0, WHITE);

    let mini_font_size = (6.0 * ctx.scale) as u16;
    let mut cur_y = (my + 10.0 * ctx.scale).floor();

    draw_text_ex(
        &game.lang_manager.get_string("inventory.title"),
        mx + 10.0 * ctx.scale,
        cur_y,
        TextParams {
            font_size: ctx.font_size,
            font: ctx.font,
            color: YELLOW,
            ..Default::default()
        },
    );
    cur_y += 12.0 * ctx.scale;

    let player = &game.player_manager.player;
    draw_text_ex(
        &format!(
            "Weight: {}/{}",
            player.total_cargo_weight(),
            player.max_cargo
        ),
        mx + 10.0 * ctx.scale,
        cur_y,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );
    cur_y += 10.0 * ctx.scale;

    // Count items and track first index
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
            mx + 10.0 * ctx.scale,
            cur_y + 6.0 * ctx.scale,
            TextParams {
                font_size: mini_font_size,
                font: ctx.font,
                color: WHITE,
                ..Default::default()
            },
        );

        let is_placeable = BlockType::from_item_type(it).is_some_and(|bt| bt.is_placeable());
        if is_placeable
            && draw_button(
                ButtonParams {
                    x: mx + mw - 35.0 * ctx.scale,
                    y: cur_y,
                    w: 30.0 * ctx.scale,
                    h: 8.0 * ctx.scale,
                    text_key: "SELECT",
                    press_key: "SELECT",
                    lang: &game.lang_manager,
                    font_size: mini_font_size - 1,
                },
                ctx.font,
            )
            && let Some(&idx) = first_indices.get(it)
        {
            ctx.events.push(GameEvent::SetSelectedItemIndex(idx));
        }
        cur_y += 10.0 * ctx.scale;
    }

    if player.cargo.is_empty() {
        draw_text_ex(
            "Empty",
            mx + 10.0 * ctx.scale,
            cur_y,
            TextParams {
                font_size: mini_font_size,
                font: ctx.font,
                color: GRAY,
                ..Default::default()
            },
        );
    }
}
