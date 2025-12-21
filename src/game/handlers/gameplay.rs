use crate::Game;
use crate::constants::*;
use crate::render::game_renderer::GameRenderer;
use macroquad::prelude::BLUE;

pub fn respawn(game: &mut Game, renderer: &GameRenderer) {
    // Clear cargo
    game.player_manager.player.cargo.clear();

    // Money penalty (10%)
    let penalty = (game.player_manager.player.money as f32 * 0.1) as i32;
    game.player_manager.player.money -= penalty;

    // Reset position and state
    game.player_manager.player.x = PLAYER_INITIAL_X;
    game.player_manager.player.y = PLAYER_INITIAL_Y;
    game.player_manager.player.vx = 0.0;
    game.player_manager.player.vy = 0.0;
    game.player_manager.player.fuel = game.player_manager.player.max_fuel;

    // Reset camera
    game.camera.x = PLAYER_INITIAL_X - SCREEN_WIDTH / 2.0;
    game.camera.y = PLAYER_INITIAL_Y - SCREEN_HEIGHT / 2.0;
    game.camera.old_x = game.camera.x;
    game.camera.old_y = game.camera.y;

    // Add some particles for visual effect
    for _ in 0..30 {
        game.particle_manager
            .add_particles(vec![crate::components::Particle::new(
                game.player_manager.player.x,
                game.player_manager.player.y,
                BLUE,
            )]);
    }

    game.is_menu_visible = false;

    game.notification_manager.add_notification(
        game.lang_manager.get_string("notification.respawn.success"),
        "success",
        renderer.get_font(),
    );
}
