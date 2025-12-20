use crate::components::Player;
use crate::constants::*;
use crate::managers::world::WorldManager;
use macroquad::prelude::*;

pub struct PlayerManager {
    pub player: Player,
}

impl PlayerManager {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            player: Player::new(x, y),
        }
    }

    pub fn update(&mut self, world_manager: &mut WorldManager) {
        // Horizontal movement
        let mut move_vec = Vec2::ZERO;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            move_vec.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            move_vec.x += 1.0;
        }

        // Tuning constants
        let base_accel = 0.4 + (self.player.engine_level as f32 - 1.0) * 0.15;
        let base_thrust = 0.25 + (self.player.engine_level as f32 - 1.0) * 0.1;
        let mut dash_mult = 1.0;

        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            dash_mult = 1.8;
            if self.player.fuel > 0.0 {
                self.player.fuel -= 0.05; // Extra cost for dashing
            } else {
                dash_mult = 1.0; // Can't dash without fuel
            }
        }

        if move_vec.x != 0.0 {
            self.player.vx += move_vec.x * base_accel * dash_mult;
        } else {
            self.player.vx *= PLAYER_FRICTION_GROUND;
        }

        // Vertical movement (Thrust - Acceleration based)
        if (is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space))
            && self.player.fuel > 0.0
        {
            // Increase base thrust for easier ascent
            self.player.vy -= (base_thrust * 1.5) * dash_mult;
            self.player.fuel -= 0.1 * dash_mult; // Consume fuel
        }

        // Gravity (Acceleration based)
        self.player.vy += PLAYER_GRAVITY;

        // Friction and Terminal Velocity
        self.player.vx *= PLAYER_FRICTION_AIR;
        // Vertical friction (Damping) for smoother float
        self.player.vy *= 0.98;

        let max_vel = PLAYER_TERMINAL_VELOCITY
            * (1.0 + (self.player.engine_level as f32 - 1.0) * 0.2)
            * dash_mult;
        self.player.vx = self.player.vx.clamp(-max_vel, max_vel);
        self.player.vy = self.player.vy.clamp(-max_vel, max_vel);

        // Collision Detection - X
        self.player.x += self.player.vx;
        self.handle_collisions(world_manager, true);

        // Collision Detection - Y
        self.player.y += self.player.vy;
        self.handle_collisions(world_manager, false);

        // Surface Station Logic
        if self.player.y < (SURFACE_Y_LEVEL as f32 * BLOCK_SIZE) + 4.0 {
            // Sell items
            if !self.player.cargo.is_empty() {
                for item in self.player.cargo.drain(..) {
                    self.player.money += match item.as_str() {
                        "Coal" => 10,
                        "Stone" => 2,
                        "Dirt" => 1,
                        _ => 0,
                    };
                }
            }
            // Refuel (Free on surface for now, maybe cost later)
            if self.player.fuel < self.player.max_fuel {
                self.player.fuel = (self.player.fuel + 1.0).min(self.player.max_fuel);
            }
        }
    }

    fn handle_collisions(&mut self, world_manager: &mut WorldManager, is_x: bool) {
        // Get surrounding blocks - use a slightly expanded area but check with current rect
        let player_box = self.player.rect();
        let start_x = (player_box.x / BLOCK_SIZE).floor() as i32 - 1;
        let start_y = (player_box.y / BLOCK_SIZE).floor() as i32 - 1;
        let end_x = ((player_box.x + player_box.w) / BLOCK_SIZE).floor() as i32 + 1;
        let end_y = ((player_box.y + player_box.h) / BLOCK_SIZE).floor() as i32 + 1;

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                let world_x = x as f32 * BLOCK_SIZE;
                let world_y = y as f32 * BLOCK_SIZE;

                if let Some((_, _, _, _, block)) =
                    world_manager.get_block_at_world_coords(world_x, world_y)
                    && !block.is_broken
                {
                    let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);

                    // Use a fresh rect each time because self.player.x/y might have changed
                    let mut current_player_rect = self.player.rect();

                    // Shrink the perpendicular axis slightly to avoid "catching" on floors while moving horizontally (and vice versa)
                    if is_x {
                        current_player_rect.y += 0.2;
                        current_player_rect.h -= 0.4;
                    } else {
                        current_player_rect.x += 0.2;
                        current_player_rect.w -= 0.4;
                    }

                    if let Some(intersect) = current_player_rect.intersect(block_rect) {
                        if is_x {
                            if self.player.vx > 0.0 {
                                self.player.x -= intersect.w;
                            } else if self.player.vx < 0.0 {
                                self.player.x += intersect.w;
                            }
                            self.player.vx = 0.0;
                        } else {
                            if self.player.vy > 0.0 {
                                self.player.y -= intersect.h;
                            } else if self.player.vy < 0.0 {
                                self.player.y += intersect.h;
                            }
                            self.player.vy = 0.0;
                        }
                    }
                }
            }
        }
    }
}
