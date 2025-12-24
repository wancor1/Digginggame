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
        let (move_vec, dash_mult) = self.process_input();
        self.apply_movement(move_vec, dash_mult);
        self.apply_physics(dash_mult);

        self.perform_movement_and_collisions(world_manager);

        self.handle_surface_logic();
    }

    fn process_input(&mut self) -> (Vec2, f32) {
        let mut move_vec = Vec2::ZERO;
        if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
            move_vec.x -= 1.0;
        }
        if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
            move_vec.x += 1.0;
        }

        let mut dash_mult = 1.0;
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            dash_mult = 1.4;
            if self.player.fuel > 0.0 {
                self.player.fuel -= 0.05;
            } else {
                dash_mult = 1.0;
            }
        }
        (move_vec, dash_mult)
    }

    fn apply_movement(&mut self, move_vec: Vec2, dash_mult: f32) {
        let base_accel = 0.2 + (self.player.engine_level as f32 - 1.0) * 0.1;

        if move_vec.x != 0.0 {
            self.player.vx += move_vec.x * base_accel * dash_mult;
        } else {
            self.player.vx *= PLAYER_FRICTION_GROUND;
        }
    }

    fn apply_physics(&mut self, dash_mult: f32) {
        let base_thrust = 0.15 + (self.player.engine_level as f32 - 1.0) * 0.08;

        // Vertical movement (Thrust)
        if (is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space))
            && self.player.fuel > 0.0
        {
            self.player.vy -= (base_thrust * 1.5) * dash_mult;
            self.player.fuel -= 0.1 * dash_mult;
        }

        // Gravity
        self.player.vy += PLAYER_GRAVITY;

        // Friction
        self.player.vx *= PLAYER_FRICTION_AIR;
        self.player.vy *= PLAYER_FRICTION_AIR;

        // Clamp Velocity
        let max_xvel = PLAYER_TERMINAL_XVELOCITY
            * (1.0 + (self.player.engine_level as f32 - 1.0) * 0.2)
            * dash_mult;
        let max_yvel = PLAYER_TERMINAL_YVELOCITY
            * (1.0 + (self.player.engine_level as f32 - 1.0) * 0.2)
            * dash_mult;
        self.player.vx = self.player.vx.clamp(-max_xvel, max_xvel);
        self.player.vy = self.player.vy.clamp(-max_yvel, max_yvel);
    }

    fn perform_movement_and_collisions(&mut self, world_manager: &mut WorldManager) {
        self.player.x += self.player.vx;
        self.handle_collisions(world_manager, true);

        self.player.y += self.player.vy;
        self.handle_collisions(world_manager, false);
    }

    fn handle_surface_logic(&mut self) {
        if self.player.y < (SURFACE_Y_LEVEL as f32 * BLOCK_SIZE) + 4.0 {
            // Auto-store natural items
            let mut i = 0;
            while i < self.player.cargo.len() {
                if self.player.cargo[i].is_auto_stored
                    && self.player.storage.len() < self.player.max_storage as usize
                {
                    let item = self.player.cargo.remove(i);
                    self.player.storage.push(item);
                } else {
                    i += 1;
                }
            }

            // Refuel
            if self.player.fuel < self.player.max_fuel {
                self.player.fuel = (self.player.fuel + 1.0).min(self.player.max_fuel);
            }
        }
    }

    fn handle_collisions(&mut self, world_manager: &mut WorldManager, is_x: bool) {
        let player_box = self.player.rect();
        let start_x = (player_box.x / BLOCK_SIZE).floor() as i32 - 1;
        let start_y = (player_box.y / BLOCK_SIZE).floor() as i32 - 1;
        let end_x = ((player_box.x + player_box.w) / BLOCK_SIZE).floor() as i32 + 1;
        let end_y = ((player_box.y + player_box.h) / BLOCK_SIZE).floor() as i32 + 1;

        for x in start_x..=end_x {
            for y in start_y..=end_y {
                self.resolve_block_collision(world_manager, x, y, is_x);
            }
        }
    }

    fn resolve_block_collision(
        &mut self,
        world_manager: &mut WorldManager,
        x: i32,
        y: i32,
        is_x: bool,
    ) {
        let world_x = x as f32 * BLOCK_SIZE;
        let world_y = y as f32 * BLOCK_SIZE;

        if let Some((_, _, _, _, block)) = world_manager.get_block_at_world_coords(world_x, world_y)
            && !block.is_broken
            && block.block_type.is_solid()
        {
            let block_rect = Rect::new(block.x, block.y, BLOCK_SIZE, BLOCK_SIZE);
            let mut current_player_rect = self.player.rect();

            if is_x {
                current_player_rect.y += 0.2;
                current_player_rect.h -= 0.4;
            } else {
                current_player_rect.x += 0.2;
                current_player_rect.w -= 0.4;
            }

            if let Some(intersect) = current_player_rect.intersect(block_rect) {
                self.apply_collision_correction(intersect, is_x, block.x, block.y);
            }
        }
    }

    fn apply_collision_correction(
        &mut self,
        intersect: Rect,
        is_x: bool,
        block_x: f32,
        block_y: f32,
    ) {
        if is_x {
            if self.player.vx > 0.0 {
                self.player.x -= intersect.w;
            } else if self.player.vx < 0.0 {
                self.player.x += intersect.w;
            } else {
                let player_center_x = self.player.x + self.player.width / 2.0;
                let block_center_x = block_x + BLOCK_SIZE / 2.0;
                if player_center_x < block_center_x {
                    self.player.x -= intersect.w;
                } else {
                    self.player.x += intersect.w;
                }
            }
            self.player.vx = 0.0;
        } else {
            if self.player.vy > 0.0 {
                self.player.y -= intersect.h;
            } else if self.player.vy < 0.0 {
                self.player.y += intersect.h;
            } else {
                let player_center_y = self.player.y + self.player.height / 2.0;
                let block_center_y = block_y + BLOCK_SIZE / 2.0;
                if player_center_y < block_center_y {
                    self.player.y -= intersect.h;
                } else {
                    self.player.y += intersect.h;
                }
            }
            self.player.vy = 0.0;
        }
    }
}
