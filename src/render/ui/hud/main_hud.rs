use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::ui::common::{ButtonParams, MenuRenderContext, draw_button};
use macroquad::prelude::*;

pub fn draw_hud(game: &Game, ctx: &mut MenuRenderContext) {
    let player = &game.player_manager.player;
    let hud_y = ctx.offset_y + 5.0 * ctx.scale;
    let hud_x = ctx.offset_x + 5.0 * ctx.scale;
    let mini_font_size = (6.0 * ctx.scale) as u16;

    draw_rectangle(hud_x, hud_y, 40.0 * ctx.scale, 4.0 * ctx.scale, DARKGRAY);
    let fuel_ratio = player.fuel / player.max_fuel;
    draw_rectangle(
        hud_x,
        hud_y,
        40.0 * ctx.scale * fuel_ratio,
        4.0 * ctx.scale,
        if fuel_ratio > 0.3 { GREEN } else { RED },
    );

    draw_text_ex(
        "FUEL",
        hud_x,
        hud_y + 8.0 * ctx.scale,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
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
        hud_x + 50.0 * ctx.scale,
        hud_y + 4.0 * ctx.scale,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );
    draw_text_ex(
        &format!("$: {}", player.money),
        ctx.offset_x + (SCREEN_WIDTH - 45.0) * ctx.scale,
        hud_y + 4.0 * ctx.scale,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: YELLOW,
            ..Default::default()
        },
    );

    let depth = (player.y / BLOCK_SIZE).floor() as i32 - SURFACE_Y_LEVEL;
    draw_text_ex(
        &format!("DEPTH: {}m", depth.max(0)),
        ctx.offset_x + (SCREEN_WIDTH - 45.0) * ctx.scale,
        hud_y + 12.0 * ctx.scale,
        TextParams {
            font_size: mini_font_size,
            font: ctx.font,
            color: WHITE,
            ..Default::default()
        },
    );

    // --- SELECTED ITEM ---
    let slot_size = 12.0 * ctx.scale;
    let sel_x = hud_x;
    let sel_y = hud_y + 15.0 * ctx.scale;

    draw_rectangle(
        sel_x,
        sel_y,
        slot_size,
        slot_size,
        Color::new(0.3, 0.3, 0.3, 0.8),
    );
    draw_rectangle_lines(sel_x, sel_y, slot_size, slot_size, 1.0, GRAY);

    if let Some(item) = player.cargo.get(game.selected_item_index) {
        if let Some(atlas_tex) = ctx.atlas {
            let sprite_rect = crate::utils::get_item_sprite(&item.item_type);
            if sprite_rect.w > 0.0 {
                draw_texture_ex(
                    atlas_tex,
                    sel_x + 2.0 * ctx.scale,
                    sel_y + 2.0 * ctx.scale,
                    WHITE,
                    DrawTextureParams {
                        source: Some(sprite_rect),
                        dest_size: Some(vec2(8.0 * ctx.scale, 8.0 * ctx.scale)),
                        ..Default::default()
                    },
                );
            }
        }
        draw_text_ex(
            "SELECTED",
            sel_x + slot_size + 2.0 * ctx.scale,
            sel_y + 4.0 * ctx.scale,
            TextParams {
                font_size: (4.0 * ctx.scale) as u16,
                font: ctx.font,
                color: LIGHTGRAY,
                ..Default::default()
            },
        );
        draw_text_ex(
            &item.item_type,
            sel_x + slot_size + 2.0 * ctx.scale,
            sel_y + 10.0 * ctx.scale,
            TextParams {
                font_size: mini_font_size,
                font: ctx.font,
                color: WHITE,
                ..Default::default()
            },
        );
    }
    // --------------

    use crate::game::UIOverlay;
    if game.on_surface && game.ui_overlay == UIOverlay::None {
        if draw_button(
            ButtonParams {
                x: ctx.offset_x + (SCREEN_WIDTH - 40.0) * ctx.scale,
                y: ctx.offset_y + 25.0 * ctx.scale,
                w: 35.0 * ctx.scale,
                h: 10.0 * ctx.scale,
                text_key: "SHOP",
                press_key: "SHOP",
                lang: &game.lang_manager,
                font_size: ctx.font_size,
            },
            ctx.font,
        ) {
            ctx.events.push(GameEvent::OpenShop);
        }
        if draw_button(
            ButtonParams {
                x: ctx.offset_x + (SCREEN_WIDTH - 40.0) * ctx.scale,
                y: ctx.offset_y + 37.0 * ctx.scale,
                w: 35.0 * ctx.scale,
                h: 10.0 * ctx.scale,
                text_key: "WAREHOUSE",
                press_key: "WAREHOUSE",
                lang: &game.lang_manager,
                font_size: ctx.font_size,
            },
            ctx.font,
        ) {
            ctx.events.push(GameEvent::OpenWarehouse);
        }
    }
}
