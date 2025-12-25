use crate::Game;
use crate::constants::*;
use crate::render::ui::common::MenuRenderContext;
use macroquad::prelude::*;

pub fn draw_player_marker(
    game: &Game,
    ctx: &MenuRenderContext,
    view_x: f32,
    view_y: f32,
    blocks_per_pixel: f32,
) {
    let player_rel_x = (game.player_manager.player.x - view_x) / (BLOCK_SIZE * blocks_per_pixel);
    let player_rel_y = (game.player_manager.player.y - view_y) / (BLOCK_SIZE * blocks_per_pixel);

    draw_circle(
        ctx.offset_x + (SCREEN_WIDTH / 2.0 + player_rel_x) * ctx.scale,
        ctx.offset_y + (SCREEN_HEIGHT / 2.0 + player_rel_y) * ctx.scale,
        3.0 * ctx.scale,
        RED,
    );
}

pub fn draw_warp_gates(
    game: &mut Game,
    ctx: &mut MenuRenderContext,
    view_x: f32,
    view_y: f32,
    blocks_per_pixel: f32,
) {
    let screen_w = SCREEN_WIDTH * ctx.scale;
    let screen_h = SCREEN_HEIGHT * ctx.scale;

    for (idx, gate) in game.player_manager.player.warp_gates.iter().enumerate() {
        // Center of the block
        let gate_world_center_x = gate.x + BLOCK_SIZE / 2.0;
        let gate_world_center_y = gate.y + BLOCK_SIZE / 2.0;

        let gate_rel_x = (gate_world_center_x - view_x) / (BLOCK_SIZE * blocks_per_pixel);
        let gate_rel_y = (gate_world_center_y - view_y) / (BLOCK_SIZE * blocks_per_pixel);

        let gx = ctx.offset_x + (SCREEN_WIDTH / 2.0 + gate_rel_x) * ctx.scale;
        let gy = ctx.offset_y + (SCREEN_HEIGHT / 2.0 + gate_rel_y) * ctx.scale;

        // Always draw markers even if they are slightly off-screen to help navigation
        let margin = 20.0 * ctx.scale;
        if gx > ctx.offset_x - margin
            && gx < ctx.offset_x + screen_w + margin
            && gy > ctx.offset_y - margin
            && gy < ctx.offset_y + screen_h + margin
        {
            let time = get_time();
            let pulse = (time * 5.0).sin() as f32 * 0.5 + 0.5; // Faster pulse

            // Interaction Check
            let (rel_mx, rel_my) = crate::utils::get_game_mouse_position();
            let gate_gx = SCREEN_WIDTH / 2.0 + gate_rel_x;
            let gate_gy = SCREEN_HEIGHT / 2.0 + gate_rel_y;
            let dist_sq = (rel_mx - gate_gx).powi(2) + (rel_my - gate_gy).powi(2);
            let is_hovered = dist_sq < 8.0f32.powi(2) && game.pending_warp_index.is_none();

            // Scale marker size slightly with zoom
            let mut zoom_factor = (game.map_zoom.sqrt()).clamp(0.5, 2.0);
            if is_hovered {
                zoom_factor *= 1.3;
            }

            let base_size = 4.0 * ctx.scale * zoom_factor;
            let outer_size = base_size * (1.1 + pulse * 0.5);

            // 1. Draw outer glow/pulse
            let glow_color = if is_hovered {
                Color::from_rgba(255, 255, 0, (200.0 * (1.0 - pulse)) as u8) // Yellow glow when hovered
            } else {
                Color::from_rgba(255, 0, 255, (150.0 * (1.0 - pulse)) as u8)
            };
            draw_poly(gx, gy, 4, outer_size, 0.0, glow_color);

            // 2. Draw main diamond
            let main_color = if is_hovered { YELLOW } else { MAGENTA };
            draw_poly(gx, gy, 4, base_size, 0.0, main_color);
            draw_poly_lines(gx, gy, 4, base_size, 0.0, 1.5 * ctx.scale, WHITE);

            // Handle Click
            if is_hovered && is_mouse_button_pressed(MouseButton::Left) {
                game.pending_warp_index = Some(idx);
            }

            // 3. Draw name label
            if let Some(font) = ctx.font {
                let font_size = (ctx.font_size as f32 * 0.7) as u16;
                let text = &gate.name;
                let text_size = measure_text(text, Some(font), font_size, 1.0);

                let tx = gx - text_size.width / 2.0;
                let ty = gy - base_size - 4.0 * ctx.scale;

                // Label background
                draw_rectangle(
                    tx - 2.0,
                    ty - text_size.height,
                    text_size.width + 4.0,
                    text_size.height + 2.0,
                    Color::from_rgba(0, 0, 0, 180),
                );

                draw_text_ex(
                    text,
                    tx,
                    ty,
                    TextParams {
                        font: Some(font),
                        font_size,
                        color: YELLOW,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
