use crate::Game;
use crate::components::BlockType;
use crate::constants::*;
use crate::render::game_renderer::GameRenderer;
use crate::render::sprites::*;
use macroquad::prelude::*;

pub fn handle_gameplay_update(game: &mut Game, game_renderer: &GameRenderer) {
    if game.is_key_pressed_buffered(KeyCode::Escape) {
        if game.is_shop_open {
            game.is_shop_open = false;
        } else if game.is_inventory_open {
            game.is_inventory_open = false;
        } else if game.is_warehouse_open {
            game.is_warehouse_open = false;
        } else if game.on_warp_select_screen {
            game.on_warp_select_screen = false;
        } else if game.on_warp_place_screen {
            game.on_warp_place_screen = false;
        } else {
            game.is_menu_visible = true;
        }
    }

    if (game.is_key_pressed_buffered(KeyCode::I) || game.is_key_pressed_buffered(KeyCode::Tab))
        && !game.is_menu_visible
        && !game.is_shop_open
        && !game.on_warp_place_screen
        && !game.on_warp_select_screen
    {
        game.is_inventory_open = !game.is_inventory_open;
    }

    if !game.is_menu_visible
        && !game.is_shop_open
        && !game.is_inventory_open
        && !game.is_warehouse_open
        && !game.on_warp_place_screen
        && !game.on_warp_select_screen
    {
        game.player_manager.update(&mut game.world_manager);
    }

    game.camera.x =
        game.player_manager.player.x - SCREEN_WIDTH / 2.0 + game.player_manager.player.width / 2.0;
    game.camera.y = game.player_manager.player.y - SCREEN_HEIGHT / 2.0
        + game.player_manager.player.height / 2.0;

    game.on_surface = game.player_manager.player.y < (SURFACE_Y_LEVEL as f32 * BLOCK_SIZE) + 8.0;

    game.world_manager
        .generate_visible_chunks(game.camera.x, game.camera.y);

    game.world_manager.update();

    let mx = (mouse_position().0 / screen_width()) * SCREEN_WIDTH;
    let my = (mouse_position().1 / screen_height()) * SCREEN_HEIGHT;
    let world_mx = (mx + game.camera.x).round();
    let world_my = (my + game.camera.y).round();

    let hovered_block_coords = game
        .world_manager
        .get_block_at_world_coords(world_mx, world_my)
        .map(|(_, _, _, _, block)| (block.x, block.y));

    let mut preview_sprite = None;
    let mut is_valid = true;

    let mut current_item_type = None;
    if game.selected_item_index < game.player_manager.player.cargo.len() {
        current_item_type = Some(
            game.player_manager.player.cargo[game.selected_item_index]
                .item_type
                .clone(),
        );
    }

    if let Some(item_type) = &current_item_type {
        let bt = BlockType::from_item_type(item_type);
        let potential_sprite = match bt {
            Some(BlockType::Dirt) => Some(SPRITE_BLOCK_DIRT),
            Some(BlockType::Stone) => Some(SPRITE_BLOCK_STONE),
            Some(BlockType::Coal) => Some(SPRITE_BLOCK_COAL),
            Some(BlockType::Grass) => Some(SPRITE_BLOCK_GRASS),
            Some(BlockType::WarpGate) => Some(SPRITE_BLOCK_WARPGATE),
            _ => None,
        };

        if let Some(sprite) = potential_sprite {
            if let Some((_, _, _, _, block)) = game
                .world_manager
                .get_block_at_world_coords(world_mx, world_my)
            {
                if let Some(BlockType::WarpGate) = bt {
                    if block.block_type != BlockType::Air {
                        is_valid = false;
                        preview_sprite = None;
                    } else {
                        preview_sprite = Some(sprite);
                    }
                } else if !block.is_broken {
                    is_valid = false;
                    preview_sprite = None;
                } else {
                    preview_sprite = Some(sprite);
                    let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                    let player_rect = game.player_manager.player.rect();
                    if block_rect.overlaps(&player_rect) {
                        is_valid = false;
                    }
                }
            } else {
                is_valid = false;
                preview_sprite = None;
            }
        }
    }

    game.select_block
        .update(hovered_block_coords, preview_sprite, is_valid);

    if game.is_mouse_button_pressed_buffered(MouseButton::Left) {
        game.handle_block_interaction(world_mx, world_my, game_renderer);
    }
    if game.is_mouse_button_pressed_buffered(MouseButton::Right) {
        game.handle_right_click(world_mx, world_my, game_renderer);
    }

    let blocks = game
        .world_manager
        .get_active_blocks_in_view(game.camera.x, game.camera.y);
    game.particle_manager.update(&blocks, &game.camera);
    game.item_manager
        .update(&mut game.player_manager.player, &blocks);
}
