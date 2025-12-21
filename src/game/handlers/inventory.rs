use crate::Game;
use crate::render::game_renderer::GameRenderer;

pub fn open_warehouse(game: &mut Game) {
    game.is_warehouse_open = true;
}

pub fn sell_item(game: &mut Game, item_type: String, quantity: usize) {
    let price = match item_type.as_str() {
        "Coal" => 10,
        "Stone" => 2,
        "Dirt" => 1,
        _ => 0,
    };
    let mut sold = 0;
    while sold < quantity {
        if let Some(pos) = game
            .player_manager
            .player
            .storage
            .iter()
            .position(|it| it.item_type == item_type)
        {
            game.player_manager.player.storage.remove(pos);
            game.player_manager.player.money += price;
            sold += 1;
        } else if let Some(pos) = game
            .player_manager
            .player
            .cargo
            .iter()
            .position(|it| it.item_type == item_type)
        {
            game.player_manager.player.cargo.remove(pos);
            game.player_manager.player.money += price;
            sold += 1;
        } else {
            break;
        }
    }
}

pub fn deposit_item(game: &mut Game, item_type: String, quantity: usize, renderer: &GameRenderer) {
    let mut moved = 0;
    while moved < quantity
        && game.player_manager.player.storage.len()
            < game.player_manager.player.max_storage as usize
    {
        if let Some(pos) = game
            .player_manager
            .player
            .cargo
            .iter()
            .position(|it| it.item_type == item_type)
        {
            let item = game.player_manager.player.cargo.remove(pos);
            game.player_manager.player.storage.push(item);
            moved += 1;
        } else {
            break;
        }
    }
    if moved < quantity
        && game.player_manager.player.storage.len()
            >= game.player_manager.player.max_storage as usize
    {
        game.notification_manager.add_notification(
            "Storage Full!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

pub fn withdraw_item(game: &mut Game, item_type: String, quantity: usize, renderer: &GameRenderer) {
    let weight = crate::utils::get_item_weight(&item_type);
    let mut moved = 0;
    while moved < quantity
        && game.player_manager.player.total_cargo_weight() + weight
            <= game.player_manager.player.max_cargo
    {
        if let Some(pos) = game
            .player_manager
            .player
            .storage
            .iter()
            .position(|it| it.item_type == item_type)
        {
            let mut item = game.player_manager.player.storage.remove(pos);
            item.is_auto_stored = false;
            game.player_manager.player.cargo.push(item);
            moved += 1;
        } else {
            break;
        }
    }

    if moved < quantity
        && game.player_manager.player.total_cargo_weight() + weight
            > game.player_manager.player.max_cargo
    {
        game.notification_manager.add_notification(
            "Cargo Full!".to_string(),
            "error",
            renderer.get_font(),
        );
    }
}

pub fn set_selected_item_index(game: &mut Game, idx: usize) {
    game.selected_item_index = idx;
    game.is_inventory_open = false;
}
