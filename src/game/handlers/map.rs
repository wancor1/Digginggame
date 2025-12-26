use crate::game::{Game, UIOverlay};
use macroquad::prelude::*;

pub fn handle_map_input(game: &mut Game) {
    if game.is_key_pressed_buffered(KeyCode::Escape) || game.is_key_pressed_buffered(KeyCode::M) {
        game.ui_overlay = UIOverlay::None;
        game.clear_inputs();
    }

    if game.is_key_pressed_buffered(KeyCode::Equal) || game.is_key_pressed_buffered(KeyCode::KpAdd)
    {
        game.map_zoom = (game.map_zoom * 2.0).min(32.0);
    }
    if game.is_key_pressed_buffered(KeyCode::Minus)
        || game.is_key_pressed_buffered(KeyCode::KpSubtract)
    {
        game.map_zoom = (game.map_zoom / 2.0).max(1.0 / 32.0);
    }

    let move_speed = 5.0 / game.map_zoom;
    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        game.map_view_y -= move_speed;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        game.map_view_y += move_speed;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        game.map_view_x -= move_speed;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        game.map_view_x += move_speed;
    }
}
