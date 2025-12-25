use crate::Game;
use crate::components::BlockType;
use crate::events::GameEvent;
use crate::render::game_renderer::GameRenderer;

pub fn try_interact_functional_block(
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
