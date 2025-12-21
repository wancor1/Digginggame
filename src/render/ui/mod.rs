use crate::Game;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::sprites::*;
use macroquad::prelude::*;
use macroquad::text::Font;
use macroquad::texture::Texture2D;

pub mod common;
pub mod hud;
pub mod screens;

pub struct UIRenderer;

impl UIRenderer {
    pub fn draw(game: &mut Game, font: Option<&Font>, atlas: Option<&Texture2D>) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let (render_width, _, offset_x, offset_y) = crate::utils::get_render_dimensions();

        let scale = render_width / SCREEN_WIDTH;
        let s_font_size = (FONT_SIZE * scale).floor() as u16;

        set_default_camera();

        {
            let mut ctx = common::MenuRenderContext {
                font,
                atlas,
                scale,
                offset_x,
                offset_y,
                font_size: s_font_size,
                events: &mut events,
            };

            if game.on_title_screen {
                screens::draw_title_screen(game, &mut ctx);
            } else if game.on_save_select_screen {
                screens::draw_save_select_screen(game, &mut ctx);
            } else if game.on_new_game_input_screen {
                screens::draw_new_game_input_screen(game, &mut ctx);
            } else if game.on_warp_place_screen {
                screens::draw_warp_place_screen(game, &mut ctx);
            } else if game.on_warp_select_screen {
                screens::draw_warp_select_screen(game, &mut ctx);
            } else {
                hud::draw_hud(game, &mut ctx);
                if game.is_shop_open {
                    hud::draw_shop(game, &mut ctx);
                }
                if game.is_inventory_open {
                    hud::draw_inventory(game, &mut ctx);
                }
                if game.is_warehouse_open {
                    hud::draw_warehouse(game, &mut ctx);
                }
                if game.is_menu_visible {
                    screens::draw_pause_menu(game, &mut ctx);
                }
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
