use crate::Game;
use crate::constants::*;
use crate::render::sprites::*;
use macroquad::prelude::*;

pub struct WorldRenderer;

impl WorldRenderer {
    pub fn draw(game: &mut Game, atlas: Option<&Texture2D>) {
        let alpha = game.alpha;
        let cx = game.camera.old_x + (game.camera.x - game.camera.old_x) * alpha;
        let cy = game.camera.old_y + (game.camera.y - game.camera.old_y) * alpha;

        let blocks = game.world_manager.get_active_blocks_in_view(cx, cy);
        for block in blocks {
            let draw_x = (block.x - cx).round();
            let draw_y = (block.y - cy).round();

            if let (Some(rect), Some(atlas_tex)) = (block.sprite_rect, atlas) {
                draw_texture_ex(
                    atlas_tex,
                    draw_x,
                    draw_y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(rect),
                        ..Default::default()
                    },
                );

                if block.current_hp < block.max_hp && block.max_hp > 0 {
                    let damage = (block.max_hp - block.current_hp) as f32 / block.max_hp as f32;
                    let frame = (damage * 5.0).ceil() as i32;
                    if frame > 0 {
                        let anim_v =
                            SPRITE_BREAK_ANIM_V_START + ((frame - 1).max(0) as f32) * BLOCK_SIZE;
                        let crack_rect =
                            Rect::new(SPRITE_BREAK_ANIM_U, anim_v, BLOCK_SIZE, BLOCK_SIZE);
                        draw_texture_ex(
                            atlas_tex,
                            draw_x,
                            draw_y,
                            WHITE,
                            DrawTextureParams {
                                source: Some(crack_rect),
                                ..Default::default()
                            },
                        );
                    }
                }
            } else {
                draw_rectangle(draw_x, draw_y, BLOCK_SIZE, BLOCK_SIZE, BROWN);
            }
        }

        for p in &game.particle_manager.active_particles {
            draw_rectangle((p.x - cx).round(), (p.y - cy).round(), 1.0, 1.0, p.color);
        }

        let player = &game.player_manager.player;
        let px = player.old_x + (player.x - player.old_x) * alpha;
        let py = player.old_y + (player.y - player.old_y) * alpha;

        draw_rectangle(
            (px - cx).round(),
            (py - cy).round(),
            player.width,
            player.height,
            ORANGE,
        );
        draw_rectangle(
            (px - cx + 1.0).round(),
            (py - cy + 1.0).round(),
            player.width - 2.0,
            2.0,
            YELLOW,
        );

        if let Some(atlas_tex) = atlas {
            game.select_block.draw(cx, cy, atlas_tex);
        }

        for item in &game.item_manager.items {
            let draw_x = (item.x - cx).round();
            let draw_y = (item.y - cy).round();
            if let Some(atlas_tex) = atlas {
                draw_texture_ex(
                    atlas_tex,
                    draw_x,
                    draw_y,
                    WHITE,
                    DrawTextureParams {
                        source: Some(item.sprite_rect),
                        dest_size: Some(vec2(4.0, 4.0)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
