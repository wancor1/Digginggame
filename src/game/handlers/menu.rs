use crate::Game;
use crate::managers::PersistenceManager;
use crate::render::game_renderer::GameRenderer;

pub fn open_save_selection(game: &mut Game) {
    game.save_files = PersistenceManager::list_save_files();
    game.on_title_screen = false;
    if game.save_files.is_empty() {
        game.on_new_game_input_screen = true;
        game.input_buffer.clear();
    } else {
        game.on_save_select_screen = true;
    }
}

pub fn load_save(game: &mut Game, filename: String) {
    game.current_save_name = filename.clone();
    game.persistence_manager.load_game(filename);
    game.on_save_select_screen = false;
}

pub fn start_new_game_setup(game: &mut Game) {
    game.on_save_select_screen = false;
    game.on_new_game_input_screen = true;
    game.input_buffer.clear();
}

pub fn confirm_new_game(game: &mut Game, name: String, renderer: &GameRenderer) {
    let mut filename = name.clone();
    if !filename.ends_with(".json") {
        filename.push_str(".json");
    }
    game.current_save_name = filename;
    game.on_new_game_input_screen = false;
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

    game.notification_manager.add_notification(
        "New Game!".to_string(),
        "success",
        renderer.get_font(),
    );
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
    game.is_menu_visible = false;
    game.is_shop_open = false;
    game.is_inventory_open = false;
    game.is_warehouse_open = false;
    game.on_warp_select_screen = false;
    game.on_warp_place_screen = false;
}
