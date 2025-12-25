use crate::Game;
use crate::components::{BlockType, Particle};
use crate::constants::*;
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
    let mut liquid_to_activate = Vec::new();

    if let Some((_, _, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        .filter(|(_, _, _, _, b)| !b.is_broken)
    {
        if block.block_type.is_liquid() {
            let old_type = block.block_type;
            let block_x = block.x;
            let block_y = block.y;
            let bx = (block_x / BLOCK_SIZE).floor() as i32;
            let by = (block_y / BLOCK_SIZE).floor() as i32;

            if block.liquid_level > 0 {
                if let Some(it) = old_type.get_data().and_then(|d| d.item_type.clone()) {
                    let weight = crate::utils::get_item_weight(&it);
                    if game.player_manager.player.total_cargo_weight() + weight
                        > game.player_manager.player.max_cargo
                    {
                        game.notification_manager.add_notification(
                            "Cargo Full!".to_string(),
                            "error",
                            game_renderer.get_font(),
                        );
                        return;
                    }

                    game.player_manager
                        .player
                        .cargo
                        .push(crate::components::OwnedItem {
                            item_type: it,
                            is_natural: true,
                            is_auto_stored: false,
                        });
                }

                block.liquid_level -= 1;
                if block.liquid_level == 0 {
                    block.block_type = BlockType::Air;
                    block.is_broken = true;
                    block.sprite_rect = None;
                }
                block.is_modified = true;
                should_mark_modified = true;

                liquid_to_activate.push((bx, by - 1));
                liquid_to_activate.push((bx, by + 1));
                liquid_to_activate.push((bx - 1, by));
                liquid_to_activate.push((bx + 1, by));
                liquid_to_activate.push((bx, by));
            }
        } else if block.max_hp != -1 {
            let temp = get_temperature(game.player_manager.player.y);
            let resistance = (game.player_manager.player.heat_resistance_level - 1) as f32
                * HEAT_RESISTANCE_STEP;
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

                let bx = (block_x / BLOCK_SIZE).floor() as i32;
                let by = (block_y / BLOCK_SIZE).floor() as i32;
                liquid_to_activate.push((bx, by - 1));
                liquid_to_activate.push((bx, by + 1));
                liquid_to_activate.push((bx - 1, by));
                liquid_to_activate.push((bx + 1, by));

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
    }

    for pos in liquid_to_activate {
        game.world_manager.active_liquids.insert(pos);
    }

    if should_mark_modified
        && let Some(chunk) = game.world_manager.get_chunk_mut(target_cx, target_cy)
    {
        chunk.is_modified_in_session = true;
    }
}

pub fn spawn_break_particles(
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
