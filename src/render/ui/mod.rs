use crate::Game;
use crate::constants::{FONT_SIZE, SCREEN_WIDTH, SPRITE_CURSOR};
use crate::events::GameEvent;
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

        let rw = render_width.floor();
        let ox = offset_x.floor();
        let oy = offset_y.floor();
        let scale = rw / SCREEN_WIDTH;
        let s_font_size = (FONT_SIZE * scale).floor() as u16;

        set_default_camera();

        {
            let mut ctx = common::MenuRenderContext {
                font,
                atlas,
                scale,
                offset_x: ox,
                offset_y: oy,
                font_size: s_font_size,
                events: &mut events,
            };

            use crate::game::{GameState, UIOverlay};
            match game.state {
                GameState::Title => screens::draw_title_screen(game, &mut ctx),
                GameState::SaveSelect => screens::draw_save_select_screen(game, &mut ctx),
                GameState::NewGameInput => screens::draw_new_game_input_screen(game, &mut ctx),
                GameState::WarpPlace => screens::draw_warp_place_screen(game, &mut ctx),
                GameState::WarpSelect => screens::draw_warp_select_screen(game, &mut ctx),
                GameState::Playing => {
                    hud::draw_hud(game, &mut ctx);
                    match game.ui_overlay {
                        UIOverlay::Shop => hud::draw_shop(game, &mut ctx),
                        UIOverlay::Inventory => hud::draw_inventory(game, &mut ctx),
                        UIOverlay::Warehouse => hud::draw_warehouse(game, &mut ctx),
                        UIOverlay::Map => screens::draw_map_screen(game, &mut ctx),
                        UIOverlay::PauseMenu => screens::draw_pause_menu(game, &mut ctx),
                        UIOverlay::None => {}
                    }
                }
            }
        }

        game.notification_manager
            .draw_high_res(font, scale, offset_x, offset_y);

        let mouse_pos = mouse_position();
        let inside_render = crate::utils::get_game_mouse_position_if_inside_render().is_some();

        let screen_h = screen_height();
        let near_vertical_edge = mouse_pos.1 <= 1.0 || mouse_pos.1 >= screen_h - 1.0;
        let show_os_cursor = near_vertical_edge || !inside_render || atlas.is_none();

        if show_os_cursor {
            show_mouse(true);
        } else if let Some(atlas_tex) = atlas {
            show_mouse(false);
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
