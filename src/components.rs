use crate::constants::*;

use ::rand::Rng;
use macroquad::prelude::*;

pub struct Camera {
    pub x: f32,
    pub y: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

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
    pub fn new(x_start: f32, y_start: f32, color: Color) -> Self {
        let mut rng = ::rand::thread_rng();
        let center_x = x_start + BLOCK_SIZE / 2.0;
        let center_y = y_start + BLOCK_SIZE / 2.0;
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED_MIN..PARTICLE_SPEED_MAX);

        Self {
            x: center_x,
            y: center_y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed - 1.5,
            color: color,
            alive: true,
            time_landed: None,
        }
    }
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
    pub fn new(x: f32, y: f32, max_hp: i32, sprite_rect: Option<Rect>) -> Self {
        Self {
            x,
            y,
            max_hp,
            current_hp: max_hp,
            is_broken: max_hp == 0, // If max_hp is 0, it's considered broken/empty
            is_modified: false,
            sprite_rect,
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
