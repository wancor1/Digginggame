use lazy_static::lazy_static;
use macroquad::prelude::Rect;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockType(pub u32);

#[allow(non_upper_case_globals)]
impl BlockType {
    pub const AIR: BlockType = BlockType(0);
    pub const DIRT: BlockType = BlockType(1);
    pub const GRASS: BlockType = BlockType(2);
    pub const STONE: BlockType = BlockType(3);
    pub const INDESTRUCTIBLE: BlockType = BlockType(4);
    pub const COAL: BlockType = BlockType(10);
    pub const OIL_SHALE: BlockType = BlockType(11);
    pub const LIMESTONE: BlockType = BlockType(12);
    pub const WARP_GATE: BlockType = BlockType(500);

    pub const Air: BlockType = Self::AIR;
    pub const Dirt: BlockType = Self::DIRT;
    pub const Grass: BlockType = Self::GRASS;
    pub const Stone: BlockType = Self::STONE;
    pub const Coal: BlockType = Self::COAL;
    pub const Indestructible: BlockType = Self::INDESTRUCTIBLE;
    pub const WarpGate: BlockType = Self::WARP_GATE;

    pub fn to_id(&self) -> u32 {
        self.0
    }

    pub fn from_id(id: u32) -> Self {
        BlockType(id)
    }

    pub fn get_data(&self) -> Option<&BlockData> {
        BLOCK_MANAGER.get_data(self)
    }

    pub fn is_solid(&self) -> bool {
        BLOCK_MANAGER.is_solid(self)
    }

    pub fn is_placeable(&self) -> bool {
        BLOCK_MANAGER.is_placeable(self)
    }

    pub fn get_sprite(&self) -> Option<Rect> {
        BLOCK_MANAGER.get_sprite(self)
    }

    pub fn get_base_hardness(&self) -> i32 {
        BLOCK_MANAGER.get_base_hardness(self)
    }

    pub fn from_item_type(item_type: &str) -> Option<Self> {
        BLOCK_MANAGER.from_item_type(item_type)
    }
}

lazy_static! {
    pub static ref BLOCK_MANAGER: BlockManager = BlockManager::new();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub id: u32,
    pub name: String,
    pub is_solid: bool,
    pub is_placeable: bool,
    pub base_hardness: i32,
    pub sprite: Option<BlockRect>,
    pub item_type: Option<String>,
    pub weight: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BlockRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<BlockRect> for Rect {
    fn from(br: BlockRect) -> Self {
        Rect::new(br.x, br.y, br.w, br.h)
    }
}

pub struct BlockManager {
    blocks: HashMap<u32, BlockData>,
    item_type_to_id: HashMap<String, u32>,
}

impl BlockManager {
    pub fn new() -> Self {
        let mut manager = Self {
            blocks: HashMap::new(),
            item_type_to_id: HashMap::new(),
        };
        manager.load_blocks();
        manager
    }

    fn load_blocks(&mut self) {
        let path = "data/blocks.json";
        if let Ok(content) = fs::read_to_string(path)
            && let Ok(blocks_vec) = serde_json::from_str::<Vec<BlockData>>(&content) {
                for block in blocks_vec {
                    if let Some(ref it) = block.item_type {
                        self.item_type_to_id.insert(it.clone(), block.id);
                    }
                    self.blocks.insert(block.id, block);
                }
            }
    }

    pub fn get_data(&self, block_type: &BlockType) -> Option<&BlockData> {
        self.blocks.get(&block_type.to_id())
    }

    pub fn from_item_type(&self, item_type: &str) -> Option<BlockType> {
        self.item_type_to_id
            .get(item_type)
            .map(|&id| BlockType::from_id(id))
    }

    pub fn is_solid(&self, block_type: &BlockType) -> bool {
        self.get_data(block_type)
            .map(|d| d.is_solid)
            .unwrap_or(false)
    }

    pub fn is_placeable(&self, block_type: &BlockType) -> bool {
        self.get_data(block_type)
            .map(|d| d.is_placeable)
            .unwrap_or(false)
    }

    pub fn get_sprite(&self, block_type: &BlockType) -> Option<Rect> {
        self.get_data(block_type)
            .and_then(|d| d.sprite.map(Rect::from))
    }

    pub fn get_base_hardness(&self, block_type: &BlockType) -> i32 {
        self.get_data(block_type)
            .map(|d| d.base_hardness)
            .unwrap_or(0)
    }

    pub fn get_weight(&self, block_type: &BlockType) -> i32 {
        self.get_data(block_type).map(|d| d.weight).unwrap_or(0)
    }
}
