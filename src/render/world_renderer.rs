use crate::Game;
use crate::constants::{
    BLOCK_SIZE, SPRITE_BREAK_ANIM_U, SPRITE_BREAK_ANIM_V_START, SURFACE_Y_LEVEL,
};
use crate::managers::block::BlockType;
use macroquad::prelude::*;
use noise::NoiseFn;
use num_traits::ToPrimitive;

pub struct WorldRenderer;

impl WorldRenderer {
    fn get_background_info(
        world_manager: &crate::managers::world::WorldManager,
        block: &crate::components::Block,
    ) -> (f32, BlockType, bool) {
        let bx = (block.x / BLOCK_SIZE).floor().to_i32().unwrap_or(0);
        let by = (block.y / BLOCK_SIZE).floor().to_i32().unwrap_or(0);

        let mut min_dist = 6;
        let mut nearest_solid_type = block.back_type;

        for dx in -5i32..=5i32 {
            for dy in -5i32..=5i32 {
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

        // Ambient Occlusion Light
        let ao_light = if min_dist > 5 {
            0.0
        } else {
            (1.0 - (min_dist as f32 / 6.0)).powf(1.5) * 0.4
        };

        // Sunlight simulation
        let mut has_sunlight = false;
        let sunlight = if by < SURFACE_Y_LEVEL {
            has_sunlight = true;
            1.0
        } else {
            let mut s = 1.0;
            let scan_limit = (by - SURFACE_Y_LEVEL).clamp(0, 20);
            for i in 1..=scan_limit {
                if let Some(nb) = world_manager.get_block_ref(bx, by - i)
                    && !nb.is_broken
                    && nb.block_type.is_solid()
                {
                    s = 0.0;
                    break;
                }
            }
            if s > 0.0 {
                has_sunlight = true;
            }
            s
        };

        let brightness = (sunlight * 0.8 + ao_light).min(1.0);

        // Determine the background type
        let final_type = if block.back_type.is_solid() {
            block.back_type
        } else if block.back_type == BlockType::Air {
            BlockType::Air
        } else {
            nearest_solid_type
        };

        (brightness, final_type, has_sunlight)
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

            let (brightness, back_type, has_sunlight) =
                Self::get_background_info(world_manager, block);

            // Draw background underground with subtle texture
            // Only draw black/dark background if there's no sunlight
            if block.y >= SURFACE_Y_LEVEL.to_f32().unwrap_or(0.0) * BLOCK_SIZE && !has_sunlight {
                let nx = block.x as f64 * 0.15;
                let ny = block.y as f64 * 0.15;
                let n = world_manager.noise_main.get([nx, ny]) as f32;
                let base_gray = 0.03 + n * 0.01;
                draw_rectangle(
                    draw_x,
                    draw_y,
                    BLOCK_SIZE,
                    BLOCK_SIZE,
                    Color::new(base_gray, base_gray, base_gray * 1.3, 1.0),
                );
            }

            if block.is_broken || block.block_type.is_liquid() {
                // Render back wall
                if brightness > 0.0
                    && let (Some(rect), Some(atlas_tex)) = (back_type.get_sprite(), atlas)
                {
                    // Background walls are significantly darker than foreground blocks (max 0.6)
                    let b = brightness * 0.6;
                    let tint = if has_sunlight {
                        Color::new(b, b, b, 1.0)
                    } else {
                        // Cooler, more atmospheric tint for deep areas
                        Color::new(b * 0.7, b * 0.75, b * 0.9, 1.0)
                    };

                    draw_texture_ex(
                        atlas_tex,
                        draw_x,
                        draw_y,
                        tint,
                        DrawTextureParams {
                            source: Some(rect),
                            ..Default::default()
                        },
                    );

                    // Add a subtle darkening overlay to background walls to further distinguish them
                    draw_rectangle(draw_x, draw_y, BLOCK_SIZE, BLOCK_SIZE, Color::new(0.0, 0.0, 0.0, 0.2));
                }

                if block.block_type.is_liquid()
                    && let (Some(rect), Some(atlas_tex)) = (block.sprite_rect, atlas)
                {
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
                // Draw a subtle drop shadow/border for foreground blocks to distinguish them from background
                draw_rectangle(draw_x + 1.0, draw_y + 1.0, BLOCK_SIZE, BLOCK_SIZE, Color::new(0.0, 0.0, 0.0, 0.5));

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
