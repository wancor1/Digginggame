use crate::components::Camera;
use crate::constants::*;
use crate::managers::{
    ItemManager, LanguageManager, NotificationManager, ParticleManager, PersistenceManager,
    PlayerManager, WorldManager,
};
use crate::render::game_renderer::GameRenderer;
use crate::ui::SelectBlock;
use macroquad::prelude::*;

pub mod event_handler;
pub mod persistence;
pub mod update_logic;

pub struct Game {
    pub world_manager: WorldManager,
    pub particle_manager: ParticleManager,
    pub persistence_manager: PersistenceManager,
    pub lang_manager: LanguageManager,
    pub notification_manager: NotificationManager,
    pub select_block: SelectBlock,
    pub camera: Camera,
    pub player_manager: PlayerManager,
    pub item_manager: ItemManager,

    // UI State
    pub on_title_screen: bool,
    pub on_save_select_screen: bool,
    pub on_new_game_input_screen: bool,
    pub is_menu_visible: bool,
    pub is_shop_open: bool,
    pub on_warp_place_screen: bool,
    pub on_warp_select_screen: bool,
    pub on_surface: bool,

    // Save/Load State
    pub save_files: Vec<String>,
    pub current_save_name: String,
    pub input_buffer: String,
}

impl Game {
    pub async fn new() -> Self {
        Self {
            world_manager: WorldManager::new(),
            particle_manager: ParticleManager::new(),
            persistence_manager: PersistenceManager::new(),
            lang_manager: LanguageManager::new(),
            notification_manager: NotificationManager::new(),
            select_block: SelectBlock::new(),
            camera: Camera::new(),
            player_manager: PlayerManager::new(PLAYER_INITIAL_X, PLAYER_INITIAL_Y),
            item_manager: ItemManager::new(),
            on_title_screen: true,
            on_save_select_screen: false,
            on_new_game_input_screen: false,
            is_menu_visible: false,
            is_shop_open: false,
            on_warp_place_screen: false,
            on_warp_select_screen: false,
            on_surface: true,
            save_files: Vec::new(),
            current_save_name: "savegame.json".to_string(),
            input_buffer: String::new(),
        }
    }

    pub fn update(&mut self, game_renderer: &GameRenderer) {
        if self.persistence_manager.is_loading {
            self.handle_loading(game_renderer);
            return;
        }

        if self.on_title_screen
            || self.on_save_select_screen
            || self.on_new_game_input_screen
            || self.is_menu_visible
        {
            if self.is_menu_visible && is_key_pressed(KeyCode::Escape) {
                self.is_menu_visible = false;
            }
        } else {
            self.handle_gameplay_update(game_renderer);
        }

        self.notification_manager.update();

        if let Some((success, msg)) = self.persistence_manager.check_save_status() {
            let t = if success { "success" } else { "error" };
            self.notification_manager
                .add_notification(msg, t, game_renderer.get_font());
        }
    }

    pub fn return_to_title_screen(&mut self, game_renderer: &GameRenderer) {
        self.world_manager = WorldManager::new();
        self.particle_manager = ParticleManager::new();
        self.camera = Camera::new();
        self.reset_player_state();
        self.on_title_screen = true;
        self.on_save_select_screen = false;
        self.on_new_game_input_screen = false;
        self.is_menu_visible = false;
        self.is_shop_open = false;
        self.on_warp_place_screen = false;
        self.on_warp_select_screen = false;

        self.world_manager.reset();
        self.current_save_name = "savegame.json".to_string();
        self.input_buffer = String::new();
        self.notification_manager.add_notification(
            "Returned to Title Screen".to_string(),
            "info",
            game_renderer.get_font(),
        );
    }

    pub fn reset_player_state(&mut self) {
        self.player_manager = PlayerManager::new(PLAYER_INITIAL_X, PLAYER_INITIAL_Y);
        self.camera.x = PLAYER_INITIAL_X - SCREEN_WIDTH / 2.0;
        self.camera.y = PLAYER_INITIAL_Y - SCREEN_HEIGHT / 2.0;
    }

    pub fn return_to_title_from_save_select(&mut self) {
        self.on_title_screen = true;
        self.on_save_select_screen = false;
        self.on_new_game_input_screen = false;
        self.is_menu_visible = false;
    }
}
