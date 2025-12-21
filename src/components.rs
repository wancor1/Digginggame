use crate::constants::*;
use crate::utils::get_item_weight;

use ::rand::Rng;
use macroquad::prelude::*;

pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub old_x: f32,
    pub old_y: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            old_x: 0.0,
            old_y: 0.0,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WarpGate {
    pub x: f32,
    pub y: f32,
    pub name: String,
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
        let mut rng = ::rand::rng();
        let center_x = x_start + BLOCK_SIZE / 2.0;
        let center_y = y_start + BLOCK_SIZE / 2.0;
        let angle = rng.random_range(0.0..std::f32::consts::TAU);
        let speed = rng.random_range(PARTICLE_SPEED_MIN..PARTICLE_SPEED_MAX);

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
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum BlockType {
    Dirt,
    Grass,
    Stone,
    Coal,
    Indestructible,
    WarpGate,
    Air,
}

impl BlockType {
    pub fn to_id(&self) -> u8 {
        match self {
            BlockType::Air => 0,
            BlockType::Dirt => 1,
            BlockType::Grass => 2,
            BlockType::Stone => 3,
            BlockType::Coal => 4,
            BlockType::Indestructible => 5,
            BlockType::WarpGate => 6,
        }
    }

    pub fn from_id(id: u8) -> Self {
        match id {
            0 => BlockType::Air,
            1 => BlockType::Dirt,
            2 => BlockType::Grass,
            3 => BlockType::Stone,
            4 => BlockType::Coal,
            5 => BlockType::Indestructible,
            6 => BlockType::WarpGate,
            _ => BlockType::Air,
        }
    }

    pub fn is_solid(&self) -> bool {
        match self {
            BlockType::Dirt
            | BlockType::Grass
            | BlockType::Stone
            | BlockType::Coal
            | BlockType::Indestructible => true,
            BlockType::WarpGate | BlockType::Air => false,
        }
    }

    pub fn is_placeable(&self) -> bool {
        match self {
            BlockType::Dirt
            | BlockType::Grass
            | BlockType::Stone
            | BlockType::Coal
            | BlockType::WarpGate => true,
            _ => false,
        }
    }

    pub fn from_item_type(item_type: &str) -> Option<Self> {
        match item_type {
            "Dirt" => Some(BlockType::Dirt),
            "Grass" => Some(BlockType::Grass),
            "Stone" => Some(BlockType::Stone),
            "Coal" => Some(BlockType::Coal),
            "WarpGate" => Some(BlockType::WarpGate),
            _ => None,
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
    pub block_type: BlockType,
    pub name: Option<String>,
    pub last_damage_time: Option<f64>,
}

impl Block {
    pub fn new(
        x: f32,
        y: f32,
        max_hp: i32,
        sprite_rect: Option<Rect>,
        block_type: BlockType,
    ) -> Self {
        Self {
            x,
            y,
            max_hp,
            current_hp: max_hp,
            is_broken: max_hp == 0 || block_type == BlockType::Air,
            is_modified: false,
            sprite_rect,
            block_type,
            name: None,
            last_damage_time: None,
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

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OwnedItem {
    pub item_type: String,
    pub is_natural: bool,
    pub is_auto_stored: bool,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub old_x: f32,
    pub old_y: f32,
    pub vx: f32,
    pub vy: f32,
    pub fuel: f32,
    pub max_fuel: f32,
    pub money: i32,
    pub cargo: Vec<OwnedItem>, // Changed from Vec<String>
    pub max_cargo: i32,
    pub storage: Vec<OwnedItem>, // Changed from Vec<String>
    pub max_storage: i32,
    pub width: f32,
    pub height: f32,
    pub drill_level: i32,
    pub tank_level: i32,
    pub engine_level: i32,
    pub cargo_level: i32,
    pub warp_gates: Vec<WarpGate>,
}

impl Player {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            old_x: x,
            old_y: y,
            vx: 0.0,
            vy: 0.0,
            fuel: PLAYER_INITIAL_FUEL,
            max_fuel: PLAYER_INITIAL_FUEL,
            money: 0,
            cargo: Vec::new(),
            max_cargo: PLAYER_INITIAL_CARGO,
            storage: Vec::new(),
            max_storage: 2000,
            width: 6.0,
            height: 6.0,
            drill_level: 1,
            tank_level: 1,
            engine_level: 1,
            cargo_level: 1,
            warp_gates: Vec::new(),
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }

    pub fn total_cargo_weight(&self) -> i32 {
        self.cargo
            .iter()
            .map(|it| get_item_weight(&it.item_type))
            .sum()
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub item_type: String,
    pub sprite_rect: Rect,
    pub alive: bool,
    pub weight: i32,
    pub is_natural: bool,
}

impl Item {
    pub fn new(
        x: f32,
        y: f32,
        item_type: String,
        sprite_rect: Rect,
        weight: i32,
        is_natural: bool,
    ) -> Self {
        let mut rng = ::rand::rng();
        Self {
            x,
            y,
            vx: rng.random_range(-0.2..0.2),
            vy: rng.random_range(-0.5..0.0),
            item_type,
            sprite_rect,
            alive: true,
            weight,
            is_natural,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, 4.0, 4.0) // Smaller than blocks
    }
}
