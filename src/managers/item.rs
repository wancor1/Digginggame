use crate::components::{Block, Item, Player};
use crate::constants::{BLOCK_SIZE, PLAYER_FRICTION_AIR, PLAYER_FRICTION_GROUND, PLAYER_GRAVITY};
use crate::utils::get_item_weight;
use macroquad::prelude::*;

pub struct ItemManager {
    pub items: Vec<Item>,
}

impl Default for ItemManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemManager {
    #[must_use]
    pub const fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn spawn_item(
        &mut self,
        x: f32,
        y: f32,
        item_type: String,
        sprite_rect: Rect,
        is_natural: bool,
    ) {
        let weight = get_item_weight(&item_type);
        self.items
            .push(Item::new(x, y, item_type, sprite_rect, weight, is_natural));
    }

    pub fn update(&mut self, player: &mut Player, blocks: &[&Block]) {
        let player_rect = player.rect();

        for item in &mut self.items {
            if !item.alive {
                continue;
            }

            // Physics
            item.vy += PLAYER_GRAVITY;
            item.vx *= PLAYER_FRICTION_AIR;
            item.vy *= PLAYER_FRICTION_AIR;

            let mut x = item.x + item.vx;
            let mut y = item.y;

            // Check X Collision
            let mut item_rect_x = Rect::new(x, y, 4.0, 4.0);
            item_rect_x.y += 0.1;
            item_rect_x.h -= 0.2;

            for block in blocks {
                if block.is_broken || !block.block_type.is_solid() {
                    continue;
                }
                let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                if item_rect_x.overlaps(&block_rect) {
                    if item.vx > 0.0 {
                        x = block.x - 4.0;
                    } else if item.vx < 0.0 {
                        x = block.x + BLOCK_SIZE;
                    }
                    item.vx *= -0.1; // Bounce X
                    break;
                }
            }
            item.x = x;

            y = item.y + item.vy;
            let mut item_rect_y = Rect::new(x, y, 4.0, 4.0);
            item_rect_y.x += 0.1;
            item_rect_y.w -= 0.2;

            let mut _on_ground = false;

            for block in blocks {
                if block.is_broken || !block.block_type.is_solid() {
                    continue;
                }
                let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                if item_rect_y.overlaps(&block_rect) {
                    if item.vy > 0.0 {
                        y = block.y - 4.0;
                        item.vy = 0.0;
                        item.vx *= PLAYER_FRICTION_GROUND;
                        _on_ground = true;
                    } else if item.vy < 0.0 {
                        y = block.y + BLOCK_SIZE;
                        item.vy = 0.0;
                    }
                    break;
                }
            }
            item.y = y;

            // Collection by player
            if player_rect.overlaps(&item.rect())
                && player.total_cargo_weight() + item.weight <= player.max_cargo
            {
                player.cargo.push(crate::components::OwnedItem {
                    item_type: item.item_type.clone(),
                    is_natural: item.is_natural,
                    is_auto_stored: item.is_natural,
                });
                item.alive = false;
            }
        }

        // Cleanup dead items
        self.items.retain(|i| i.alive);
    }
}
