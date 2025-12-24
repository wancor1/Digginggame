use super::Game;
use crate::render::game_renderer::GameRenderer;

pub mod gameplay;
pub mod interaction;
pub mod loading;

impl Game {
    pub fn handle_loading(&mut self, game_renderer: &GameRenderer) {
        loading::handle_loading(self, game_renderer);
    }

    pub fn handle_gameplay_update(&mut self, game_renderer: &GameRenderer) {
        gameplay::handle_gameplay_update(self, game_renderer);
    }

    pub fn handle_block_interaction(
        &mut self,
        world_mx: f32,
        world_my: f32,
        game_renderer: &GameRenderer,
    ) {
        interaction::handle_block_interaction(self, world_mx, world_my, game_renderer);
    }

    pub fn handle_right_click(
        &mut self,
        world_mx: f32,
        world_my: f32,
        game_renderer: &GameRenderer,
    ) {
        interaction::handle_right_click(self, world_mx, world_my, game_renderer);
    }
}
