use crate::Game;
use crate::render::game_renderer::GameRenderer;

pub fn upgrade_drill(game: &mut Game, renderer: &GameRenderer) {
    let cost = game.player_manager.player.drill_level * 100;
    if game.player_manager.player.money >= cost {
        game.player_manager.player.money -= cost;
        game.player_manager.player.drill_level += 1;
        game.notification_manager.add_notification(
            "Drill Upgraded!".to_string(),
            "success",
            renderer.get_font(),
        );
    } else {
        game.notification_manager.add_notification(
            "Not enough money!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

pub fn upgrade_tank(game: &mut Game, renderer: &GameRenderer) {
    let cost = game.player_manager.player.tank_level * 80;
    if game.player_manager.player.money >= cost {
        game.player_manager.player.money -= cost;
        game.player_manager.player.tank_level += 1;
        game.player_manager.player.max_fuel += 50.0;
        game.player_manager.player.fuel = game.player_manager.player.max_fuel;
        game.notification_manager.add_notification(
            "Tank Upgraded!".to_string(),
            "success",
            renderer.get_font(),
        );
    } else {
        game.notification_manager.add_notification(
            "Not enough money!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

pub fn upgrade_engine(game: &mut Game, renderer: &GameRenderer) {
    let cost = game.player_manager.player.engine_level * 120;
    if game.player_manager.player.money >= cost {
        game.player_manager.player.money -= cost;
        game.player_manager.player.engine_level += 1;
        game.notification_manager.add_notification(
            "Engine Upgraded!".to_string(),
            "success",
            renderer.get_font(),
        );
    } else {
        game.notification_manager.add_notification(
            "Not enough money!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

pub fn upgrade_cargo(game: &mut Game, renderer: &GameRenderer) {
    let cost = game.player_manager.player.cargo_level * 150;
    if game.player_manager.player.money >= cost {
        game.player_manager.player.money -= cost;
        game.player_manager.player.cargo_level += 1;
        game.player_manager.player.max_cargo += 250;
        game.notification_manager.add_notification(
            "Cargo Upgraded!".to_string(),
            "success",
            renderer.get_font(),
        );
    } else {
        game.notification_manager.add_notification(
            "Not enough money!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

pub fn buy_warp_gate(game: &mut Game, renderer: &GameRenderer) {
    if game.player_manager.player.money >= 500 {
        if game.player_manager.player.cargo.len() < game.player_manager.player.max_cargo as usize {
            game.player_manager.player.money -= 500;
            game.player_manager
                .player
                .cargo
                .push(crate::components::OwnedItem {
                    item_type: "WarpGate".to_string(),
                    is_natural: false,
                    is_auto_stored: false,
                });
            game.notification_manager.add_notification(
                "Warp Gate Purchased!".to_string(),
                "success",
                renderer.get_font(),
            );
        } else {
            game.notification_manager.add_notification(
                "Cargo Full!".to_string(),
                "error",
                renderer.get_font(),
            );
        }
    } else {
        game.notification_manager.add_notification(
            "Not enough money!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

use crate::game::UIOverlay;
pub fn open_shop(game: &mut Game) {
    game.ui_overlay = UIOverlay::Shop;
}
