use crate::Game;
use crate::components::BlockType;
use crate::constants::{
    BLOCK_SIZE, CAMERA_DEADZONE_RADIUS, SCREEN_HEIGHT, SCREEN_WIDTH, SURFACE_Y_LEVEL,
};
use crate::game::UIOverlay;
use crate::render::game_renderer::GameRenderer;
use macroquad::prelude::*;
use num_traits::ToPrimitive;

pub fn handle_gameplay_update(game: &mut Game, game_renderer: &GameRenderer) {
    update_ui_state(game);

    if game.ui_overlay == UIOverlay::None {
        game.player_manager.update(&mut game.world_manager);
    }

    update_camera(game);

    game.on_surface = game.player_manager.player.y
        < (SURFACE_Y_LEVEL.to_f32().unwrap_or(0.0)).mul_add(BLOCK_SIZE, 8.0);

    update_world(game);

    let world_mouse = get_world_mouse_coords(&game.camera);

    if game.state == crate::game::GameState::Playing && game.ui_overlay == UIOverlay::None {
        if let Some((world_mx, world_my)) = world_mouse {
            update_interaction_preview(game, world_mx, world_my);
            handle_interactions(game, world_mx, world_my, game_renderer);
        } else {
            game.select_block.update(None, None, true);
        }
    } else {
        game.select_block.update(None, None, true);
    }

    update_managers(game);
}

fn update_ui_state(game: &mut Game) {
    if game.is_key_pressed_buffered(KeyCode::Escape) {
        match game.ui_overlay {
            UIOverlay::None => game.ui_overlay = UIOverlay::PauseMenu,
            _ => game.ui_overlay = UIOverlay::None,
        }
        game.clear_inputs();
    }

    if (game.is_key_pressed_buffered(KeyCode::I) || game.is_key_pressed_buffered(KeyCode::Tab))
        && game.ui_overlay == UIOverlay::None
    {
        game.ui_overlay = UIOverlay::Inventory;
        game.clear_inputs();
    }

    if game.is_key_pressed_buffered(KeyCode::M) && game.ui_overlay == UIOverlay::None {
        game.ui_overlay = UIOverlay::Map;
        game.map_view_x = game.player_manager.player.x;
        game.map_view_y = game.player_manager.player.y;
        crate::game::handlers::warp::sync_warp_gates(game);
        game.clear_inputs();
    }
}

fn update_camera(game: &mut Game) {
    let player_center_x = game.player_manager.player.x + game.player_manager.player.width / 2.0;
    let player_center_y = game.player_manager.player.y + game.player_manager.player.height / 2.0;

    let camera_center_x = game.camera.x + SCREEN_WIDTH / 2.0;
    let camera_center_y = game.camera.y + SCREEN_HEIGHT / 2.0;

    let dx = player_center_x - camera_center_x;
    let dy = player_center_y - camera_center_y;
    let distance = dx.hypot(dy);

    if distance > CAMERA_DEADZONE_RADIUS {
        let angle = dy.atan2(dx);
        let move_dist = distance - CAMERA_DEADZONE_RADIUS;

        game.camera.x += angle.cos() * move_dist;
        game.camera.y += angle.sin() * move_dist;
    }
}

fn update_world(game: &mut Game) {
    game.world_manager
        .generate_visible_chunks(game.camera.x, game.camera.y);
    game.world_manager
        .update_liquids(game.camera.x, game.camera.y);
    game.world_manager.update();
}

fn get_world_mouse_coords(camera: &crate::components::Camera) -> Option<(f32, f32)> {
    let (mx, my) = crate::utils::get_game_mouse_position_if_inside_render()?;
    Some(((mx + camera.x).round(), (my + camera.y).round()))
}

fn update_interaction_preview(game: &mut Game, world_mx: f32, world_my: f32) {
    let target_block_info = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my);

    let hovered_block_coords = target_block_info
        .as_ref()
        .map(|(_, _, _, _, block)| (block.x, block.y));

    let mut preview_sprite = None;
    let mut is_valid = false;

    // Use a labeled block to allow breaking out early while keeping the final update call reachable
    'logic: {
        let current_item_type = match game
            .player_manager
            .player
            .cargo
            .get(game.selected_item_index)
        {
            Some(it) => &it.item_type,
            None => break 'logic,
        };

        let Some(bt) = BlockType::from_item_type(current_item_type) else {
            break 'logic;
        };

        let Some(sprite) = crate::managers::block::BlockType::get_sprite(&bt) else {
            break 'logic;
        };

        let Some((_, _, _, _, block)) = target_block_info else {
            break 'logic;
        };

        // Logic determines if the interaction is valid
        if bt == BlockType::WarpGate {
            if block.block_type == BlockType::Air {
                preview_sprite = Some(sprite);
                is_valid = true;
            }
        } else if block.is_broken {
            // Placing a block
            preview_sprite = Some(sprite);
            let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
            let player_rect = game.player_manager.player.rect();

            // Valid if it doesn't overlap the player
            is_valid = !block_rect.overlaps(&player_rect);
        }
    }

    game.select_block
        .update(hovered_block_coords, preview_sprite, is_valid);
}

fn handle_interactions(
    game: &mut Game,
    world_mx: f32,
    world_my: f32,
    game_renderer: &GameRenderer,
) {
    if game.is_mouse_button_pressed_buffered(MouseButton::Left) {
        game.handle_block_interaction(world_mx, world_my, game_renderer);
    }
    if game.is_mouse_button_pressed_buffered(MouseButton::Right) {
        game.handle_right_click(world_mx, world_my, game_renderer);
    }
}

fn update_managers(game: &mut Game) {
    let blocks = game
        .world_manager
        .get_active_blocks_in_view(game.camera.x, game.camera.y);
    game.particle_manager.update(&blocks, &game.camera);
    game.item_manager
        .update(&mut game.player_manager.player, &blocks);
}
