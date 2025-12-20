use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::sprites::*;
use macroquad::prelude::*;

pub mod common;
pub mod game_hud;
pub mod menu_screens;

pub struct UIRenderer;

impl UIRenderer {
    pub fn draw(game: &mut Game, font: Option<&Font>, atlas: Option<&Texture2D>) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let target_aspect = SCREEN_WIDTH / SCREEN_HEIGHT;
        let screen_aspect = screen_width() / screen_height();

        let (render_width, render_height, offset_x, offset_y);
        if screen_aspect > target_aspect {
            render_height = screen_height();
            render_width = SCREEN_WIDTH * (render_height / SCREEN_HEIGHT);
            offset_x = (screen_width() - render_width) / 2.0;
            offset_y = 0.0;
        } else {
            render_width = screen_width();
            render_height = SCREEN_HEIGHT * (render_width / SCREEN_WIDTH);
            offset_x = 0.0;
            offset_y = (screen_height() - render_height) / 2.0;
        }

        let scale = render_width / SCREEN_WIDTH;
        let s_font_size = (FONT_SIZE * scale).floor() as u16;

        set_default_camera();

        if game.on_title_screen {
            menu_screens::draw_title_screen(game, font, scale, &mut events);
        } else if game.on_save_select_screen {
            menu_screens::draw_save_select_screen(
                game,
                font,
                scale,
                offset_x,
                offset_y,
                s_font_size,
                &mut events,
            );
        } else if game.on_new_game_input_screen {
            menu_screens::draw_new_game_input_screen(
                game,
                font,
                scale,
                offset_x,
                offset_y,
                s_font_size,
                &mut events,
            );
        } else if game.on_warp_place_screen {
            menu_screens::draw_warp_place_screen(
                game,
                font,
                scale,
                offset_x,
                offset_y,
                s_font_size,
                &mut events,
            );
        } else if game.on_warp_select_screen {
            menu_screens::draw_warp_select_screen(
                game,
                font,
                scale,
                offset_x,
                offset_y,
                s_font_size,
                &mut events,
            );
        } else {
            game_hud::draw_hud(game, font, scale, offset_x, offset_y, &mut events);
            if game.is_shop_open {
                game_hud::draw_shop(
                    game,
                    font,
                    scale,
                    offset_x,
                    offset_y,
                    s_font_size,
                    &mut events,
                );
            }
            if game.is_inventory_open {
                game_hud::draw_inventory(
                    game,
                    font,
                    scale,
                    offset_x,
                    offset_y,
                    s_font_size,
                    &mut events,
                );
            }
            if game.is_warehouse_open {
                game_hud::draw_warehouse(
                    game,
                    font,
                    scale,
                    offset_x,
                    offset_y,
                    s_font_size,
                    &mut events,
                );
            }
            if game.is_menu_visible {
                menu_screens::draw_pause_menu(
                    game,
                    font,
                    scale,
                    offset_x,
                    offset_y,
                    s_font_size,
                    &mut events,
                );
            }
        }

        game.notification_manager
            .draw_high_res(font, scale, offset_x, offset_y);

        let mouse_pos = mouse_position();
        if let Some(atlas_tex) = atlas {
            draw_texture_ex(
                atlas_tex,
                mouse_pos.0,
                mouse_pos.1,
                WHITE,
                DrawTextureParams {
                    source: Some(SPRITE_CURSOR),
                    dest_size: Some(vec2(8.0 * scale, 8.0 * scale)),
                    ..Default::default()
                },
            );
        }

        events
    }
}
