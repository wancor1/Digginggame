use crate::constants::*;
use crate::utils::{chunk_coords_to_world_origin};
use ::rand::Rng;
use macroquad::prelude::*;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: Color,
    pub alive: bool,
    pub time_landed: Option<f64>,
}

impl Particle {
    const GRAVITY: f32 = 0.19;
    const MAX_LIFESPAN_ON_GROUND_SEC: f64 = 5.0;
    const PARTICLE_SPEED_MIN: f32 = 20.0 / 60.0;
    const PARTICLE_SPEED_MAX: f32 = 60.0 / 60.0;
    const BOUNCE_DAMPENING_X: f32 = -0.4;
    const FRICTION_ON_GROUND: f32 = 0.85;

    pub fn new(x_start: f32, y_start: f32, block_max_hardness: i32) -> Self {
        let mut rng = ::rand::thread_rng();
        let center_x = x_start + BLOCK_SIZE / 2.0;
        let center_y = y_start + BLOCK_SIZE / 2.0;
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(Self::PARTICLE_SPEED_MIN..Self::PARTICLE_SPEED_MAX);

        let color = if block_max_hardness <= 5 {
            if rng.random::<f32>() < 0.9 {
                LIGHTGRAY
            } else {
                WHITE
            } // 9/13 approximation
        } else if block_max_hardness <= 10 {
            if rng.random::<f32>() < 0.9 {
                LIGHTGRAY
            } else {
                DARKGRAY
            } // 13/6
        } else {
            BLACK // 0
        };

        Self {
            x: center_x,
            y: center_y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed - 1.5,
            color,
            alive: true,
            time_landed: None,
        }
    }

    pub fn update(&mut self, collidable_blocks: &[&Block]) {
        if !self.alive {
            return;
        }

        self.vy += Self::GRAVITY;
        self.x += self.vx;

        for block in collidable_blocks {
            if self.check_collision(block) {
                if self.vx > 0.0 {
                    self.x = block.x - 0.1;
                } else {
                    self.x = block.x + BLOCK_SIZE + 0.1;
                }
                self.vx *= Self::BOUNCE_DAMPENING_X;
                break;
            }
        }

        self.y += self.vy;
        let mut is_on_ground = false;

        for block in collidable_blocks {
            if self.check_collision(block) {
                if self.vy > 0.0 {
                    self.y = block.y - 0.1;
                    self.vy = 0.0;
                    self.vx *= Self::FRICTION_ON_GROUND;
                    is_on_ground = true;
                } else if self.vy < 0.0 {
                    self.y = block.y + BLOCK_SIZE + 0.1;
                    self.vy = 0.0;
                }
                break;
            }
        }

        if is_on_ground {
            let now = get_time();
            if self.time_landed.is_none() {
                self.time_landed = Some(now);
            } else if now - self.time_landed.unwrap() > Self::MAX_LIFESPAN_ON_GROUND_SEC {
                self.alive = false;
            }
        } else {
            self.time_landed = None;
        }

        if self.y > SCREEN_HEIGHT + BLOCK_SIZE * 5.0 {
            self.alive = false;
        }
    }

    fn check_collision(&self, block: &Block) -> bool {
        self.x >= block.x
            && self.x < block.x + BLOCK_SIZE
            && self.y >= block.y
            && self.y < block.y + BLOCK_SIZE
    }

    pub fn draw(&self) {
        if self.alive {
            draw_rectangle(self.x, self.y, 1.0, 1.0, self.color); // Pixel
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockSaveData {
    pub x: f32,
    pub y: f32,
    pub current_hp: i32,
    pub sprite_id: String,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub x: f32,
    pub y: f32,
    pub max_hp: i32,
    pub current_hp: i32,
    pub is_broken: bool,
    pub is_modified: bool,
    pub sprite_rect: Option<Rect>,
}

impl Block {
    const HARDNESS_MIN: i32 = 3;
    const SURFACE_Y_LEVEL: i32 = 7;
    const NOISE_SCALE_HARDNESS: f64 = 0.005;
    const NOISE_SCALE_ORE: f64 = 0.04;
    const ORE_THRESHOLD: f64 = 0.4;
    const HARDNESS_INCREASE_PER_BLOCK: f64 = 0.1;
    const NOISE_HARDNESS_RANGE: f64 = 20.0;

    // Note: Noise generators need to be stateful or instantiated.
    // We will do instantiation inside init for simplicity or pass them.
    // For performance, we should reuse them, but Block init happens once per chunk gen.

    pub fn new(
        x: f32,
        y: f32,
        noise_fn_hardness: &impl NoiseFn<f64, 3>,
        noise_fn_ore: &impl NoiseFn<f64, 3>,
    ) -> Self {
        let y_block = (y / BLOCK_SIZE).floor() as i32;

        if y_block < Self::SURFACE_Y_LEVEL {
            return Self {
                x,
                y,
                max_hp: 0,
                current_hp: 0,
                is_broken: true,
                is_modified: false,
                sprite_rect: None,
            };
        }

        let mut block = Self {
            x,
            y,
            max_hp: 0,
            current_hp: 0,
            is_broken: false,
            is_modified: false,
            sprite_rect: None,
        };

        if y_block == Self::SURFACE_Y_LEVEL {
            block.sprite_rect = Some(SPRITE_BLOCK_GRASS);
            block.max_hp = Self::HARDNESS_MIN;
        } else {
            let depth = (y_block - (Self::SURFACE_Y_LEVEL + 1)) as f64;
            let base_hardness =
                Self::HARDNESS_MIN as f64 + depth * Self::HARDNESS_INCREASE_PER_BLOCK;
            let noise_val = noise_fn_hardness.get([
                x as f64 * Self::NOISE_SCALE_HARDNESS,
                y as f64 * Self::NOISE_SCALE_HARDNESS,
                0.0,
            ]);

            // Simplified noise logic for brevity, omitting complex depth transition for now
            let noise_contribution = noise_val
                * Self::NOISE_HARDNESS_RANGE
                * (if noise_val >= 0.0 { 1.0 } else { 0.25 });
            block.max_hp = (base_hardness + noise_contribution)
                .floor()
                .max(Self::HARDNESS_MIN as f64) as i32;

            if block.max_hp <= 10 {
                block.sprite_rect = Some(SPRITE_BLOCK_DIRT);
            } else {
                let ore_val = noise_fn_ore.get([
                    x as f64 * Self::NOISE_SCALE_ORE,
                    y as f64 * Self::NOISE_SCALE_ORE,
                    256.0,
                ]);
                block.sprite_rect = Some(if ore_val >= Self::ORE_THRESHOLD {
                    SPRITE_BLOCK_COAL
                } else {
                    SPRITE_BLOCK_STONE
                });
            }
        }
        block.current_hp = block.max_hp;
        block
    }

    pub fn handle_click(&mut self) -> Option<Vec<Particle>> {
        if self.is_broken {
            return None;
        }

        if self.current_hp == self.max_hp {
            self.is_modified = true;
        }
        self.current_hp -= 1;

        if self.current_hp <= 0 {
            self.current_hp = 0;
            self.is_broken = true;
            self.is_modified = true;
            let count = ::rand::thread_rng().random_range(5..15);
            let particles = (0..count)
                .map(|_| Particle::new(self.x, self.y, self.max_hp))
                .collect();
            Some(particles)
        } else {
            None
        }
    }

    pub fn to_save_data(&self) -> BlockSaveData {
        let id = if self.sprite_rect == Some(SPRITE_BLOCK_DIRT) {
            "dirt"
        } else if self.sprite_rect == Some(SPRITE_BLOCK_GRASS) {
            "grass"
        } else if self.sprite_rect == Some(SPRITE_BLOCK_STONE) {
            "stone"
        } else if self.sprite_rect == Some(SPRITE_BLOCK_COAL) {
            "coal"
        } else {
            "unknown"
        }; // naive check, ID map better in real app

        BlockSaveData {
            x: self.x,
            y: self.y,
            current_hp: self.current_hp,
            sprite_id: id.to_string(),
        }
    }
}

pub struct Chunk {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub blocks: Vec<Vec<Block>>, // Changed to 2D Vec, easier to instantiate
    pub is_generated: bool,
    pub is_modified_in_session: bool,
}

impl Chunk {
    pub fn new(cx: i32, cy: i32) -> Self {
        Self {
            chunk_x: cx,
            chunk_y: cy,
            blocks: Vec::new(),
            is_generated: false,
            is_modified_in_session: false,
        }
    }

    pub fn generate(&mut self, noise_main: &Perlin, noise_ore: &Perlin) {
        if self.is_generated {
            return;
        }

        let (origin_x, origin_y) = chunk_coords_to_world_origin(self.chunk_x, self.chunk_y);

        // Populate 2D Vec
        for bx in 0..CHUNK_SIZE_X_BLOCKS {
            let mut row = Vec::new();
            for by in 0..CHUNK_SIZE_Y_BLOCKS {
                let wx = origin_x + bx as f32 * BLOCK_SIZE;
                let wy = origin_y + by as f32 * BLOCK_SIZE;
                row.push(Block::new(wx, wy, noise_main, noise_ore));
            }
            self.blocks.push(row);
        }
        self.is_generated = true;
    }

    pub fn get_block(&mut self, rel_x: usize, rel_y: usize) -> Option<&mut Block> {
        if !self.is_generated {
            return None;
        }
        if rel_x < CHUNK_SIZE_X_BLOCKS && rel_y < CHUNK_SIZE_Y_BLOCKS {
            Some(&mut self.blocks[rel_x][rel_y])
        } else {
            None
        }
    }
}
