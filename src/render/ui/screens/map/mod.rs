pub mod markers;
pub mod overlay;
pub mod terrain;

use crate::Game;
use crate::render::ui::common::MenuRenderContext;

pub fn draw_map_screen(game: &mut Game, ctx: &mut MenuRenderContext) {
    // Center of the map is the view position
    let view_x = game.map_view_x;
    let view_y = game.map_view_y;

    // Zoom: how many blocks one SCREEN pixel represents.
    let blocks_per_pixel = 1.0 / game.map_zoom;

    terrain::draw_terrain(game, ctx, view_x, view_y, blocks_per_pixel);
    markers::draw_player_marker(game, ctx, view_x, view_y, blocks_per_pixel);
    markers::draw_warp_gates(game, ctx, view_x, view_y, blocks_per_pixel);
    overlay::draw_overlay_ui(game, ctx, view_x, view_y, blocks_per_pixel);
    overlay::draw_confirmation_dialog(game, ctx);
}
