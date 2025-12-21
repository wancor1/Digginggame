use crate::constants::*;
use crate::game::{Game, GameState, UIOverlay};
use crate::render::game_renderer::GameRenderer;

pub fn start_place_warp_gate(game: &mut Game) {
    game.state = GameState::WarpPlace;
    game.input_buffer.clear();
    game.ui_overlay = UIOverlay::None;
}

pub fn confirm_warp_gate_name(game: &mut Game, name: String, renderer: &GameRenderer) {
    if let Some(pos) = game
        .player_manager
        .player
        .cargo
        .iter()
        .position(|it| it.item_type == "WarpGate")
    {
        game.player_manager.player.cargo.remove(pos);
    }

    // Get placement coordinates
    let (wx, wy) = if let Some(target) = game.warp_placement_target {
        target
    } else {
        // Fallback to player pos aligned
        (
            (game.player_manager.player.x / BLOCK_SIZE).round() * BLOCK_SIZE,
            (game.player_manager.player.y / BLOCK_SIZE).round() * BLOCK_SIZE,
        )
    };

    game.player_manager
        .player
        .warp_gates
        .push(crate::components::WarpGate {
            x: wx,
            y: wy,
            name: name.clone(),
        });

    // We also need to set the block in the world!
    if let Some((cx, cy, _, _, block)) = game.world_manager.get_block_at_world_coords(wx, wy) {
        let bt = crate::components::BlockType::WarpGate;
        block.block_type = bt;
        block.sprite_rect = bt.get_sprite();
        block.max_hp = bt.get_base_hardness();
        block.current_hp = block.max_hp;
        block.is_broken = false;
        block.is_modified = true;
        block.name = Some(name);

        if let Some(chunk) = game.world_manager.get_chunk_mut(cx, cy) {
            chunk.is_modified_in_session = true;
        }
    }
    game.warp_placement_target = None;

    game.state = GameState::Playing;
    game.input_buffer.clear();
    game.notification_manager.add_notification(
        "Warp Gate Placed!".to_string(),
        "success",
        renderer.get_font(),
    );
}

pub fn open_warp_menu(game: &mut Game) {
    game.state = GameState::WarpSelect;
    game.ui_overlay = UIOverlay::None;
}

pub fn teleport_to_warp(game: &mut Game, idx: usize, renderer: &GameRenderer) {
    if let Some(gate) = game.player_manager.player.warp_gates.get(idx) {
        game.player_manager.player.x = gate.x;
        game.player_manager.player.y = gate.y;
        game.player_manager.player.vx = 0.0;
        game.player_manager.player.vy = 0.0;
        game.state = GameState::Playing;
        game.notification_manager.add_notification(
            format!("Warped to {}!", gate.name),
            "success",
            renderer.get_font(),
        );
    }
}
