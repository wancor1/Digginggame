use crate::Game;
use crate::constants::*;
use crate::render::sprites::*;
use macroquad::prelude::*;

pub struct WorldRenderer;

impl WorldRenderer {
    pub fn draw(game: &mut Game, atlas: Option<&Texture2D>) {
        let cx = game.camera.x;
        let cy = game.camera.y;

        let blocks = game.world_manager.get_active_blocks_in_view(cx, cy);
        for block in blocks {
            let draw_x = (block.x - cx).floor();
            let draw_y = (block.y - cy).floor();

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
            draw_rectangle((p.x - cx).floor(), (p.y - cy).floor(), 1.0, 1.0, p.color);
        }

        let player = &game.player_manager.player;
        draw_rectangle(
            (player.x - cx).floor(),
            (player.y - cy).floor(),
            player.width,
            player.height,
            ORANGE,
        );
        draw_rectangle(
            (player.x - cx + 1.0).floor(),
            (player.y - cy + 1.0).floor(),
            player.width - 2.0,
            2.0,
            YELLOW,
        );

        if let Some(atlas_tex) = atlas {
            game.select_block.draw(cx, cy, atlas_tex);
        }

        for gate in &game.player_manager.player.warp_gates {
            draw_rectangle(
                (gate.x - cx).floor(),
                (gate.y - cy).floor(),
                8.0,
                8.0,
                PURPLE,
            );
            draw_rectangle_lines(
                (gate.x - cx).floor(),
                (gate.y - cy).floor(),
                8.0,
                8.0,
                1.0,
                WHITE,
            );
        }

        for item in &game.item_manager.items {
            let draw_x = (item.x - cx).floor();
            let draw_y = (item.y - cy).floor();
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
