use crate::Game;
use crate::components::{BlockType, Particle};
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::game_renderer::GameRenderer;
use crate::render::sprites::*;
use crate::utils::world_to_chunk_coords;
use ::rand::Rng;
use macroquad::prelude::*;

pub fn handle_block_interaction(
    game: &mut Game,
    world_mx: f32,
    world_my: f32,
    game_renderer: &GameRenderer,
) {
    let (target_cx, target_cy) = world_to_chunk_coords(world_mx, world_my);
    game.world_manager
        .ensure_chunk_exists_and_generated(target_cx, target_cy);

    let mut should_mark_modified = false;
    if let Some((_, _, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        .filter(|(_, _, _, _, b)| !b.is_broken)
    {
        if block.max_hp == -1 {
            return;
        }
        block.current_hp -= game.player_manager.player.drill_level;
        block.last_damage_time = Some(get_time());

        if block.current_hp <= 0 {
            should_mark_modified = true;
            // Special handling for WarpGate destruction
            if block.block_type == BlockType::WarpGate {
                // Remove from registry
                if let Some(pos) = game
                    .player_manager
                    .player
                    .warp_gates
                    .iter()
                    .position(|w| w.x == block.x && w.y == block.y)
                {
                    game.player_manager.player.warp_gates.remove(pos);
                    game.notification_manager.add_notification(
                        "Warp Gate Destroyed!".to_string(),
                        "info",
                        game_renderer.get_font(),
                    );
                }
            }

            block.current_hp = 0;
            block.is_broken = true;
            block.is_modified = true;
            block.block_type = BlockType::Air; // Reset type to Air
            let count = ::rand::rng().random_range(5..15);
            let particles: Vec<Particle> = (0..count)
                .map(|_| {
                    let particle_color = block
                        .sprite_rect
                        .map_or(WHITE, |rect| game_renderer.get_random_pixel_color(rect));
                    Particle::new(block.x, block.y, particle_color)
                })
                .collect();
            game.particle_manager.add_particles(particles);

            if let Some(rect) = block.sprite_rect {
                let item_type = if rect == SPRITE_BLOCK_COAL {
                    Some("Coal".to_string())
                } else if rect == SPRITE_BLOCK_STONE {
                    Some("Stone".to_string())
                } else if rect == SPRITE_BLOCK_DIRT {
                    Some("Dirt".to_string())
                } else {
                    None
                };
                if let Some(it) = item_type {
                    // Item is 4x4, Block is 8x8.
                    game.item_manager
                        .spawn_item(block.x + 2.0, block.y + 2.0, it, rect, true);
                }
            }
        }
    }

    if should_mark_modified
        && let Some(chunk) = game.world_manager.get_chunk_mut(target_cx, target_cy)
    {
        chunk.is_modified_in_session = true;
    }
}

pub fn handle_right_click(
    game: &mut Game,
    world_mx: f32,
    world_my: f32,
    game_renderer: &GameRenderer,
) {
    let (cx, cy) = world_to_chunk_coords(world_mx, world_my);
    game.world_manager.ensure_chunk_exists_and_generated(cx, cy);

    // 1. Check interaction with existing functional blocks
    if let Some((_, _, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        && !block.is_broken
        && block.block_type == BlockType::WarpGate
    {
        // Auto-register if not in registry
        if !game
            .player_manager
            .player
            .warp_gates
            .iter()
            .any(|w| w.x == block.x && w.y == block.y)
        {
            game.player_manager
                .player
                .warp_gates
                .push(crate::components::WarpGate {
                    x: block.x,
                    y: block.y,
                    name: block.name.clone().unwrap_or_else(|| "Home".to_string()),
                });
        }
        game.handle_event(GameEvent::OpenWarpMenu, game_renderer);
        return; // Interaction consumes the click
    }

    // 2. Check Item Placement
    if game.selected_item_index >= game.player_manager.player.cargo.len() {
        return;
    }

    let it_type = game.player_manager.player.cargo[game.selected_item_index]
        .item_type
        .clone();

    let block_type_to_place = BlockType::from_item_type(&it_type);
    let is_placeable = block_type_to_place
        .as_ref()
        .is_some_and(|bt| bt.is_placeable());

    if !is_placeable {
        return;
    }

    if let Some((cx, cy, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        && (block.block_type == BlockType::Air || block.is_broken)
    {
        let will_be_solid = block_type_to_place.as_ref().is_some_and(|bt| bt.is_solid());

        let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
        let player_rect = game.player_manager.player.rect();

        if !will_be_solid || !block_rect.overlaps(&player_rect) {
            if let Some(BlockType::WarpGate) = block_type_to_place {
                game.warp_placement_target = Some((block.x, block.y));
                game.handle_event(GameEvent::StartPlaceWarpGate, game_renderer);
                return;
            }

            // Place standard block
            block.is_broken = false;
            block.is_modified = true;

            let (hp, sprite, b_type) = match block_type_to_place {
                Some(BlockType::Dirt) => (HARDNESS_DIRT, SPRITE_BLOCK_DIRT, BlockType::Dirt),
                Some(BlockType::Stone) => (HARDNESS_STONE, SPRITE_BLOCK_STONE, BlockType::Stone),
                Some(BlockType::Coal) => (HARDNESS_COAL, SPRITE_BLOCK_COAL, BlockType::Coal),
                Some(BlockType::Grass) => (HARDNESS_GRASS, SPRITE_BLOCK_GRASS, BlockType::Grass),
                _ => (HARDNESS_DIRT, SPRITE_BLOCK_DIRT, BlockType::Dirt),
            };

            let y_block = (block.y / BLOCK_SIZE).floor() as i32;
            let depth = (y_block - SURFACE_Y_LEVEL).max(0) as f64;
            let multiplier = 1.0 + depth * HARDNESS_DEPTH_MULTIPLIER;
            block.max_hp = (hp as f64 * multiplier).floor() as i32;
            block.current_hp = block.max_hp;
            block.sprite_rect = Some(sprite);
            block.block_type = b_type;

            let count = 5;
            let particles: Vec<Particle> = (0..count)
                .map(|_| {
                    let particle_color = block
                        .sprite_rect
                        .map_or(WHITE, |rect| game_renderer.get_random_pixel_color(rect));
                    Particle::new(block.x, block.y, particle_color)
                })
                .collect();
            game.particle_manager.add_particles(particles);

            if let Some(chunk) = game.world_manager.get_chunk_mut(cx, cy) {
                chunk.is_modified_in_session = true;
            }

            game.player_manager
                .player
                .cargo
                .remove(game.selected_item_index);
        }
    }
}
