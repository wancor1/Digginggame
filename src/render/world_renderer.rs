use crate::Game;
use crate::constants::{
    BLOCK_SIZE, SPRITE_BREAK_ANIM_U, SPRITE_BREAK_ANIM_V_START, SURFACE_Y_LEVEL,
};
use crate::managers::block::BlockType;
use macroquad::prelude::*;
use num_traits::ToPrimitive;

pub struct WorldRenderer;

impl WorldRenderer {
    fn get_background_info(
        world_manager: &crate::managers::world::WorldManager,
        block: &crate::components::Block,
    ) -> (f32, BlockType) {
        let bx = (block.x / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
        let by = (block.y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);

        let mut min_dist = 5;
        let mut nearest_solid_type = block.back_type;

        for dx in -4i32..=4i32 {
            for dy in -4i32..=4i32 {
                let dist = dx.abs().max(dy.abs());
                if dist == 0 || dist >= min_dist {
                    continue;
                }

                if let Some(nb) = world_manager.get_block_ref(bx + dx, by + dy)
                    && !nb.is_broken
                    && nb.block_type.is_solid()
                {
                    min_dist = dist;
                    nearest_solid_type = nb.block_type;
                }
            }
        }

        let brightness = if min_dist > 4 {
            0.0
        } else {
            0.5f32.powi(min_dist)
        };

        // Determine the background type:
        // 1. If original back_type is solid, keep it.
        // 2. If it was liquid, interpolate from nearest solid (to avoid liquid-texture walls).
        // 3. If it was Air (sky), keep it as Air to avoid showing walls in the empty sky.
        // 4. For others (like special blocks), allow interpolation if they aren't solid.
        let final_type = if block.back_type.is_solid() {
            block.back_type
        } else if block.back_type == BlockType::Air {
            BlockType::Air
        } else {
            // This covers liquids and other non-solid special blocks
            nearest_solid_type
        };

        (brightness, final_type)
    }

    pub fn draw(game: &mut Game, atlas: Option<&Texture2D>) {
        let alpha = game.alpha;
        let cx = game.camera.old_x + (game.camera.x - game.camera.old_x) * alpha;
        let cy = game.camera.old_y + (game.camera.y - game.camera.old_y) * alpha;

        // Ensure chunks are generated (mutable borrow)
        game.world_manager.get_active_blocks_in_view(cx, cy);

        // Re-borrow world_manager immutably for the rest of the drawing
        let world_manager = &game.world_manager;
        let blocks = game
            .world_manager
            .get_active_blocks_in_view_immutable(cx, cy);

        for block in blocks {
            let draw_x = (block.x - cx).round();
            let draw_y = (block.y - cy).round();

            // Draw black background if underground
            if block.y >= SURFACE_Y_LEVEL.to_f32().unwrap_or(0.0) * BLOCK_SIZE {
                draw_rectangle(draw_x, draw_y, BLOCK_SIZE, BLOCK_SIZE, BLACK);
            }

            if block.is_broken {
                // Render back wall if broken
                let (brightness, back_type) = Self::get_background_info(world_manager, block);
                if brightness > 0.0
                    && let (Some(rect), Some(atlas_tex)) = (back_type.get_sprite(), atlas)
                {
                    draw_texture_ex(
                        atlas_tex,
                        draw_x,
                        draw_y,
                        Color::new(brightness, brightness, brightness, 1.0),
                        DrawTextureParams {
                            source: Some(rect),
                            ..Default::default()
                        },
                    );
                }
            } else if block.block_type.is_liquid() {
                // Render back wall first for liquids
                let (brightness, back_type) = Self::get_background_info(world_manager, block);
                if brightness > 0.0
                    && let (Some(rect), Some(atlas_tex)) = (back_type.get_sprite(), atlas)
                {
                    draw_texture_ex(
                        atlas_tex,
                        draw_x,
                        draw_y,
                        Color::new(brightness, brightness, brightness, 1.0),
                        DrawTextureParams {
                            source: Some(rect),
                            ..Default::default()
                        },
                    );
                }

                if let (Some(rect), Some(atlas_tex)) = (block.sprite_rect, atlas) {
                    let level = f32::from(block.liquid_level.clamp(1, 8));
                    let height_ratio = level / 8.0;
                    let draw_height = BLOCK_SIZE * height_ratio;
                    let offset_y = BLOCK_SIZE - draw_height;

                    let mut source_rect = rect;
                    source_rect.h *= height_ratio;

                    draw_texture_ex(
                        atlas_tex,
                        draw_x,
                        draw_y + offset_y,
                        Color::new(1.0, 1.0, 1.0, 0.4),
                        DrawTextureParams {
                            source: Some(source_rect),
                            dest_size: Some(vec2(BLOCK_SIZE, draw_height)),
                            ..Default::default()
                        },
                    );
                }
            } else if let (Some(rect), Some(atlas_tex)) = (block.sprite_rect, atlas) {
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
                    let damage = (block.max_hp - block.current_hp).to_f32().unwrap_or(0.0)
                        / block.max_hp.to_f32().unwrap_or(0.0);
                    let frame = (damage * 5.0).ceil().to_i32().unwrap_or(0);
                    if frame > 0 {
                        let anim_v = ((frame - 1).max(0).to_f32().unwrap_or(0.0))
                            .mul_add(BLOCK_SIZE, SPRITE_BREAK_ANIM_V_START);
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
        let px = (player.x - player.old_x).mul_add(alpha, player.old_x);
        let py = (player.y - player.old_y).mul_add(alpha, player.old_y);

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
