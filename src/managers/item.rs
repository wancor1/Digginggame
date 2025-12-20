use crate::components::{Block, Item, Player};
use crate::constants::*;
use macroquad::prelude::*;

pub struct ItemManager {
    pub items: Vec<Item>,
}

impl ItemManager {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn spawn_item(&mut self, x: f32, y: f32, item_type: String, sprite_rect: Rect) {
        self.items.push(Item::new(x, y, item_type, sprite_rect));
    }

    pub fn update(&mut self, player: &mut Player, blocks: &[&Block]) {
        let player_rect = player.rect();

        for item in self.items.iter_mut() {
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
            let item_rect_x = Rect::new(x, y, 4.0, 4.0);
            for block in blocks {
                let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                if item_rect_x.overlaps(&block_rect) {
                    if item.vx > 0.0 {
                        x = block.x - 4.0;
                    } else if item.vx < 0.0 {
                        x = block.x + BLOCK_SIZE;
                    }
                    item.vx *= -0.5; // Bounce X
                }
            }
            item.x = x;

            y = item.y + item.vy;
            let item_rect_y = Rect::new(x, y, 4.0, 4.0);
            let mut on_ground = false;

            for block in blocks {
                let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
                if item_rect_y.overlaps(&block_rect) {
                    if item.vy > 0.0 {
                        y = block.y - 4.0;
                        item.vy = 0.0;
                        item.vx *= PLAYER_FRICTION_GROUND;
                        on_ground = true;
                    } else if item.vy < 0.0 {
                        y = block.y + BLOCK_SIZE;
                        item.vy = 0.0;
                    }
                }
            }
            item.y = y;

            // Collection by player
            if player_rect.overlaps(&item.rect()) {
                if player.cargo.len() < player.max_cargo {
                    player.cargo.push(item.item_type.clone());
                    item.alive = false;
                }
            }
        }

        // Cleanup dead items
        self.items.retain(|i| i.alive);
    }
}
