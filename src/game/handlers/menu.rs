use crate::game::{Game, GameState, UIOverlay};
use crate::managers::PersistenceManager;
use crate::render::game_renderer::GameRenderer;

pub fn open_save_selection(game: &mut Game) {
    game.save_files = PersistenceManager::list_save_files();
    if game.save_files.is_empty() {
        game.state = GameState::NewGameInput;
        game.input_buffer.clear();
    } else {
        game.state = GameState::SaveSelect;
    }
}

pub fn load_save(game: &mut Game, filename: String) {
    game.current_save_name.clone_from(&filename);
    game.persistence_manager.load_game(filename);
    game.state = GameState::Playing;
}

pub fn start_new_game_setup(game: &mut Game) {
    game.state = GameState::NewGameInput;
    game.input_buffer.clear();
}

pub fn confirm_new_game(game: &mut Game, name: &str, renderer: &GameRenderer) {
    let mut filename = name.to_string();
    if !std::path::Path::new(&filename)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("dat"))
    {
        filename.push_str(".dat");
    }
    game.current_save_name = filename;
    game.state = GameState::Playing;
    game.input_buffer.clear();
    game.reset_player_state();
    game.world_manager.seed(::rand::random(), ::rand::random());
    game.world_manager.generate_visible_chunks(0.0, 0.0);

    // Add Initial Warp Gate Registry
    let player_start_x_block = (crate::constants::PLAYER_INITIAL_X / crate::constants::BLOCK_SIZE)
        .floor()
        * crate::constants::BLOCK_SIZE;
    let player_start_y_block = (crate::constants::PLAYER_INITIAL_Y / crate::constants::BLOCK_SIZE)
        .floor()
        * crate::constants::BLOCK_SIZE;
    game.player_manager
        .player
        .warp_gates
        .push(crate::components::WarpGate {
            x: player_start_x_block,
            y: player_start_y_block,
            name: "Home".to_string(),
        });

    game.notification_manager
        .add_notification("New Game!", "success", renderer.get_font());
}

pub fn save_game(game: &mut Game) {
    let data = game.make_save_data();
    game.persistence_manager
        .save_game(game.current_save_name.clone(), data);
}

pub fn quit_game() {
    std::process::exit(0);
}

pub fn return_to_title(game: &mut Game, renderer: &GameRenderer) {
    game.return_to_title_screen(renderer);
}

pub fn return_to_title_from_save_select(game: &mut Game) {
    game.return_to_title_from_save_select();
}

pub fn close_menu(game: &mut Game) {
    game.ui_overlay = UIOverlay::None;
    if game.state == GameState::WarpPlace || game.state == GameState::WarpSelect {
        game.state = GameState::Playing;
    }
}
