use super::breaking::spawn_break_particles;
use super::functional::try_interact_functional_block;
use crate::Game;
use crate::components::BlockType;
use crate::constants::*;
use crate::events::GameEvent;
use crate::render::game_renderer::GameRenderer;
use crate::utils::world_to_chunk_coords;
use macroquad::prelude::*;

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

    let mut liquid_to_activate = Vec::new();

    if let Some((_, _, _, _, block)) = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
    {
        let bt = block_type_to_place.unwrap();
        let will_be_solid = bt.is_solid();
        let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
        let block_x = block.x;
        let block_y = block.y;

        let mut placed = false;

        if bt.is_liquid() {
            if (block.is_broken || block.block_type == bt) && block.liquid_level < 8 {
                block.is_broken = false;
                block.is_modified = true;
                block.block_type = bt;
                block.sprite_rect = bt.get_sprite();
                block.liquid_level += 1;
                placed = true;
            }
        } else if (block.is_broken
            || block.block_type == BlockType::Air
            || block.block_type.is_liquid())
            && (!will_be_solid || !block_rect.overlaps(&player_rect))
        {
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
            block.liquid_level = 0;
            placed = true;
        }

        if placed {
            let bx = (block_x / BLOCK_SIZE).floor() as i32;
            let by = (block_y / BLOCK_SIZE).floor() as i32;
            liquid_to_activate.push((bx, by - 1));
            liquid_to_activate.push((bx, by + 1));
            liquid_to_activate.push((bx - 1, by));
            liquid_to_activate.push((bx + 1, by));
            liquid_to_activate.push((bx, by));

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

    for pos in liquid_to_activate {
        game.world_manager.active_liquids.insert(pos);
    }
}
