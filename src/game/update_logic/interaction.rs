use crate::Game;
use crate::components::{BlockType, Particle};
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::game_renderer::GameRenderer;
use crate::utils::{get_temperature, world_to_chunk_coords};
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

        let temp = get_temperature(game.player_manager.player.y);
        let resistance =
            (game.player_manager.player.heat_resistance_level - 1) as f32 * HEAT_RESISTANCE_STEP;
        let effective_temp = temp - resistance;

        let mut drill_power = game.player_manager.player.drill_level as f32;

        if effective_temp >= TEMPERATURE_DEBUFF_THRESHOLD {
            drill_power /= 2.0;
            let excess = effective_temp - TEMPERATURE_DEBUFF_THRESHOLD;
            drill_power -= excess;
        }

        block.current_hp -= drill_power.max(1.0) as i32;
        block.last_damage_time = Some(get_time());

        if block.current_hp <= 0 {
            should_mark_modified = true;

            // Break block
            let old_sprite_rect = block.sprite_rect;
            let old_block_type = block.block_type;
            let block_x = block.x;
            let block_y = block.y;

            // Handle WarpGate special removal
            if old_block_type == BlockType::WarpGate
                && let Some(pos) = game
                    .player_manager
                    .player
                    .warp_gates
                    .iter()
                    .position(|w| w.x == block_x && w.y == block_y)
            {
                game.player_manager.player.warp_gates.remove(pos);
                game.notification_manager.add_notification(
                    "Warp Gate Destroyed!".to_string(),
                    "info",
                    game_renderer.get_font(),
                );
            }

            block.current_hp = 0;
            block.is_broken = true;
            block.is_modified = true;
            block.block_type = BlockType::Air;

            spawn_break_particles(
                &mut game.particle_manager,
                block_x,
                block_y,
                old_sprite_rect,
                game_renderer,
            );

            if let Some(it) = old_block_type.get_data().and_then(|d| d.item_type.clone())
                && let Some(rect) = old_sprite_rect
            {
                game.item_manager
                    .spawn_item(block_x + 2.0, block_y + 2.0, it, rect, true);
            }
        }
    }

    if should_mark_modified
        && let Some(chunk) = game.world_manager.get_chunk_mut(target_cx, target_cy)
    {
        chunk.is_modified_in_session = true;
    }
}

fn spawn_break_particles(
    particle_manager: &mut crate::managers::ParticleManager,
    x: f32,
    y: f32,
    sprite_rect: Option<Rect>,
    game_renderer: &GameRenderer,
) {
    let count = ::rand::rng().random_range(5..15);
    let particles: Vec<Particle> = (0..count)
        .map(|_| {
            let particle_color =
                sprite_rect.map_or(WHITE, |rect| game_renderer.get_random_pixel_color(rect));
            Particle::new(x, y, particle_color)
        })
        .collect();
    particle_manager.add_particles(particles);
}

pub fn handle_right_click(
    game: &mut Game,
    world_mx: f32,
    world_my: f32,
    game_renderer: &GameRenderer,
) {
    let (cx, cy) = world_to_chunk_coords(world_mx, world_my);
    game.world_manager.ensure_chunk_exists_and_generated(cx, cy);

    if try_interact_functional_block(game, world_mx, world_my, game_renderer) {
        return;
    }

    try_place_block(game, world_mx, world_my, game_renderer, cx, cy);
}

fn try_interact_functional_block(
    game: &mut Game,
    world_mx: f32,
    world_my: f32,
    game_renderer: &GameRenderer,
) -> bool {
    if let Some((_, _, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        && !block.is_broken
        && block.block_type == BlockType::WarpGate
    {
        let block_x = block.x;
        let block_y = block.y;
        let block_name = block.name.clone();

        if !game
            .player_manager
            .player
            .warp_gates
            .iter()
            .any(|w| w.x == block_x && w.y == block_y)
        {
            game.player_manager
                .player
                .warp_gates
                .push(crate::components::WarpGate {
                    x: block_x,
                    y: block_y,
                    name: block_name.unwrap_or_else(|| "Home".to_string()),
                });
        }
        game.handle_event(GameEvent::OpenWarpMenu, game_renderer);
        return true;
    }
    false
}

fn try_place_block(
    game: &mut Game,
    world_mx: f32,
    world_my: f32,
    game_renderer: &GameRenderer,
    cx: i32,
    cy: i32,
) {
    if game.selected_item_index >= game.player_manager.player.cargo.len() {
        return;
    }

    let it_type = game.player_manager.player.cargo[game.selected_item_index]
        .item_type
        .clone();
    let block_type_to_place = BlockType::from_item_type(&it_type);

    if !block_type_to_place
        .as_ref()
        .is_some_and(|bt| bt.is_placeable())
    {
        return;
    }

    let player_rect = game.player_manager.player.rect();

    if let Some((_, _, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        && (block.block_type == BlockType::Air || block.is_broken)
    {
        let bt = block_type_to_place.unwrap();
        let will_be_solid = bt.is_solid();
        let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
        let block_x = block.x;
        let block_y = block.y;

        if !will_be_solid || !block_rect.overlaps(&player_rect) {
            if bt == BlockType::WarpGate {
                game.warp_placement_target = Some((block_x, block_y));
                game.handle_event(GameEvent::StartPlaceWarpGate, game_renderer);
                return;
            }

            // Place standard block
            block.is_broken = false;
            block.is_modified = true;
            block.block_type = bt;
            block.sprite_rect = bt.get_sprite();

            let hp = bt.get_base_hardness();
            let y_block = (block_y / BLOCK_SIZE).floor() as i32;
            let depth = (y_block - SURFACE_Y_LEVEL).max(0) as f64;
            let multiplier = 1.0 + depth * HARDNESS_DEPTH_MULTIPLIER;
            block.max_hp = (hp as f64 * multiplier).floor() as i32;
            block.current_hp = block.max_hp;

            spawn_break_particles(
                &mut game.particle_manager,
                block_x,
                block_y,
                block.sprite_rect,
                game_renderer,
            );

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
