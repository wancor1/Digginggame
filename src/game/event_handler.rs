use super::Game;
use super::handlers;
use crate::events::GameEvent;
use crate::render::game_renderer::GameRenderer;

impl Game {
    pub fn handle_event(&mut self, event: GameEvent, game_renderer: &GameRenderer) {
        match event {
            GameEvent::OpenSaveSelection => handlers::menu::open_save_selection(self),
            GameEvent::LoadSave(filename) => handlers::menu::load_save(self, filename),
            GameEvent::StartNewGameSetup => handlers::menu::start_new_game_setup(self),
            GameEvent::ConfirmNewGame(name) => {
                handlers::menu::confirm_new_game(self, name, game_renderer)
            }
            GameEvent::SaveGame => handlers::menu::save_game(self),
            GameEvent::QuitGame => handlers::menu::quit_game(),
            GameEvent::ReturnToTitle => handlers::menu::return_to_title(self, game_renderer),
            GameEvent::ReturnToTitleFromSaveSelect => {
                handlers::menu::return_to_title_from_save_select(self)
            }
            GameEvent::UpgradeDrill => handlers::shop::upgrade_drill(self, game_renderer),
            GameEvent::UpgradeTank => handlers::shop::upgrade_tank(self, game_renderer),
            GameEvent::UpgradeEngine => handlers::shop::upgrade_engine(self, game_renderer),
            GameEvent::UpgradeCargo => handlers::shop::upgrade_cargo(self, game_renderer),
            GameEvent::BuyWarpGate => handlers::shop::buy_warp_gate(self, game_renderer),
            GameEvent::StartPlaceWarpGate => handlers::warp::start_place_warp_gate(self),
            GameEvent::ConfirmWarpGateName(name) => {
                handlers::warp::confirm_warp_gate_name(self, name, game_renderer)
            }
            GameEvent::OpenWarpMenu => handlers::warp::open_warp_menu(self),
            GameEvent::TeleportToWarp(idx) => {
                handlers::warp::teleport_to_warp(self, idx, game_renderer)
            }
            GameEvent::OpenShop => handlers::shop::open_shop(self),
            GameEvent::OpenWarehouse => handlers::inventory::open_warehouse(self),
            GameEvent::SellItem(item_type, quantity) => {
                handlers::inventory::sell_item(self, item_type, quantity)
            }
            GameEvent::DepositItem(item_type, quantity) => {
                handlers::inventory::deposit_item(self, item_type, quantity, game_renderer)
            }
            GameEvent::WithdrawItem(item_type, quantity) => {
                handlers::inventory::withdraw_item(self, item_type, quantity, game_renderer)
            }
            GameEvent::CloseMenu => handlers::menu::close_menu(self),
            GameEvent::Respawn => handlers::gameplay::respawn(self, game_renderer),
            GameEvent::SetSelectedItemIndex(idx) => {
                handlers::inventory::set_selected_item_index(self, idx)
            }
        }
    }
}
