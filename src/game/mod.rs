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
pub mod handlers;
pub mod persistence;
pub mod update_logic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Title,
    SaveSelect,
    NewGameInput,
    Playing,
    WarpPlace,
    WarpSelect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIOverlay {
    None,
    PauseMenu,
    Shop,
    Inventory,
    Warehouse,
}

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

    // State Management
    pub state: GameState,
    pub ui_overlay: UIOverlay,
    pub on_surface: bool,

    // Save/Load State
    pub save_files: Vec<String>,
    pub current_save_name: String,
    pub input_buffer: String,
    pub warehouse_quantity: usize, // 1, 10, 100, or 0 for ALL
    pub selected_item_index: usize,

    // Input buffering
    pub key_presses: Vec<KeyCode>,
    pub mouse_presses: Vec<MouseButton>,

    pub alpha: f32, // Interpolation factor
    pub warp_placement_target: Option<(f32, f32)>,
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
            state: GameState::Title,
            ui_overlay: UIOverlay::None,
            on_surface: true,
            save_files: Vec::new(),
            current_save_name: "savegame.json".to_string(),
            input_buffer: String::new(),
            warehouse_quantity: 1,
            selected_item_index: 0,
            key_presses: Vec::new(),
            mouse_presses: Vec::new(),
            alpha: 0.0,
            warp_placement_target: None,
        }
    }

    pub fn update(&mut self, game_renderer: &GameRenderer) {
        // Record previous positions for interpolation
        self.player_manager.player.old_x = self.player_manager.player.x;
        self.player_manager.player.old_y = self.player_manager.player.y;
        self.camera.old_x = self.camera.x;
        self.camera.old_y = self.camera.y;

        if self.persistence_manager.is_loading {
            self.handle_loading(game_renderer);
            return;
        }

        match self.state {
            GameState::Title | GameState::SaveSelect | GameState::NewGameInput => {
                // Potential state-specific updates could go here
            }
            GameState::Playing | GameState::WarpPlace | GameState::WarpSelect => {
                if self.ui_overlay == UIOverlay::PauseMenu {
                    if self.is_key_pressed_buffered(KeyCode::Escape) {
                        self.ui_overlay = UIOverlay::None;
                    }
                } else {
                    self.handle_gameplay_update(game_renderer);
                }
            }
        }

        self.notification_manager.update();

        if let Some(res) = self.persistence_manager.check_save_status() {
            let (t, msg) = match res {
                Ok(msg) => ("success", msg),
                Err(msg) => ("error", msg),
            };
            self.notification_manager
                .add_notification(msg, t, game_renderer.get_font());
        }
    }

    pub fn capture_input(&mut self) {
        let keys_to_capture = [
            KeyCode::Escape,
            KeyCode::I,
            KeyCode::Tab,
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        for key in keys_to_capture {
            if is_key_pressed(key) {
                self.key_presses.push(key);
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            self.mouse_presses.push(MouseButton::Left);
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            self.mouse_presses.push(MouseButton::Right);
        }
    }

    pub fn is_key_pressed_buffered(&mut self, key: KeyCode) -> bool {
        if let Some(pos) = self.key_presses.iter().position(|&k| k == key) {
            self.key_presses.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn is_mouse_button_pressed_buffered(&mut self, button: MouseButton) -> bool {
        if let Some(pos) = self.mouse_presses.iter().position(|&b| b == button) {
            self.mouse_presses.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn return_to_title_screen(&mut self, game_renderer: &GameRenderer) {
        self.world_manager = WorldManager::new();
        self.particle_manager = ParticleManager::new();
        self.camera = Camera::new();
        self.reset_player_state();
        self.state = GameState::Title;
        self.ui_overlay = UIOverlay::None;

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
        self.state = GameState::Title;
        self.ui_overlay = UIOverlay::None;
    }
}
