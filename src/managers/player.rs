use crate::components::Player;
use crate::constants::{
    BLOCK_SIZE, LIQUID_BUOYANCY, LIQUID_RESISTANCE, PLAYER_FRICTION_AIR, PLAYER_FRICTION_GROUND,
    PLAYER_GRAVITY, PLAYER_TERMINAL_XVELOCITY, PLAYER_TERMINAL_YVELOCITY, SURFACE_Y_LEVEL,
};
use crate::managers::world::WorldManager;
use macroquad::prelude::*;
use num_traits::ToPrimitive;

pub struct PlayerManager {
    pub player: Player,
}

impl PlayerManager {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self {
            player: Player::new(x, y),
        }
    }

    pub fn update(&mut self, world_manager: &mut WorldManager) {
        let liquid_level = self.get_liquid_level(world_manager);
        let (move_vec, dash_mult) = self.process_input();
        self.apply_movement(move_vec, dash_mult, liquid_level);
        self.apply_physics(dash_mult, liquid_level);

        self.perform_movement_and_collisions(world_manager);

        self.handle_surface_logic();
    }

    fn get_liquid_level(&self, world_manager: &mut WorldManager) -> u8 {
        let px = self.player.x + self.player.width / 2.0;
        let py = self.player.y + self.player.height / 2.0;
        if let Some((_, _, _, _, block)) = world_manager.get_block_at_world_coords(px, py) {
            if block.block_type.is_liquid() {
                block.liquid_level
            } else {
                0
            }
        } else {
            0
        }
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

    fn apply_movement(&mut self, move_vec: Vec2, dash_mult: f32, liquid_level: u8) {
        let mut base_accel =
            (self.player.engine_level.to_f32().unwrap_or(0.0) - 1.0).mul_add(0.1, 0.2);

        if liquid_level > 0 {
            let effect_ratio = f32::from(liquid_level) / 8.0;
            base_accel *= 0.5f32.mul_add(-effect_ratio, 1.0);
        }

        if move_vec.x == 0.0 {
            self.player.vx *= PLAYER_FRICTION_GROUND;
        } else {
            self.player.vx += move_vec.x * base_accel * dash_mult;
        }
    }

    fn apply_physics(&mut self, dash_mult: f32, liquid_level: u8) {
        let base_thrust =
            (self.player.engine_level.to_f32().unwrap_or(0.0) - 1.0).mul_add(0.08, 0.15);

        // Vertical movement (Thrust)
        if (is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Space))
            && self.player.fuel > 0.0
        {
            self.player.vy -= (base_thrust * 1.5) * dash_mult;
            self.player.fuel -= 0.1 * dash_mult;
        }

        // Gravity
        self.player.vy += PLAYER_GRAVITY;

        // Buoyancy and Resistance
        if liquid_level > 0 {
            let effect_ratio = f32::from(liquid_level) / 8.0;

            // Buoyancy
            self.player.vy -= LIQUID_BUOYANCY * effect_ratio;

            // Friction/Resistance
            let resistance = (1.0 - LIQUID_RESISTANCE).mul_add(-effect_ratio, 1.0);
            self.player.vx *= resistance;
            self.player.vy *= resistance;
        }

        // Friction
        self.player.vx *= PLAYER_FRICTION_AIR;
        self.player.vy *= PLAYER_FRICTION_AIR;

        // Clamp Velocity
        let max_xvel = PLAYER_TERMINAL_XVELOCITY
            * (self.player.engine_level.to_f32().unwrap_or(0.0) - 1.0).mul_add(0.2, 1.0)
            * dash_mult;
        let max_yvel = PLAYER_TERMINAL_YVELOCITY
            * (self.player.engine_level.to_f32().unwrap_or(0.0) - 1.0).mul_add(0.2, 1.0)
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
        if self.player.y < (SURFACE_Y_LEVEL.to_f32().unwrap_or(0.0)).mul_add(BLOCK_SIZE, 4.0) {
            // Auto-store natural items
            let mut i = 0;
            while i < self.player.cargo.len() {
                if self.player.cargo[i].is_auto_stored
                    && self.player.storage.len() < self.player.max_storage.to_usize().unwrap_or(0)
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
        let start_x = (player_box.x / BLOCK_SIZE).floor().to_i32().unwrap_or(0) - 1;
        let start_y = (player_box.y / BLOCK_SIZE).floor().to_i32().unwrap_or(0) - 1;
        let end_x = ((player_box.x + player_box.w) / BLOCK_SIZE)
            .floor()
            .to_i32()
            .unwrap_or(0)
            + 1;
        let end_y = ((player_box.y + player_box.h) / BLOCK_SIZE)
            .floor()
            .to_i32()
            .unwrap_or(0)
            + 1;

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
        let world_x = x.to_f32().unwrap_or(0.0) * BLOCK_SIZE;
        let world_y = y.to_f32().unwrap_or(0.0) * BLOCK_SIZE;

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
