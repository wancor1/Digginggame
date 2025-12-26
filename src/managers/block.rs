use macroquad::prelude::{BLACK, Color, Rect};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockType(pub u32);

impl BlockType {
    pub const AIR: Self = Self(0);
    pub const DIRT: Self = Self(1);
    pub const GRASS: Self = Self(2);
    pub const STONE: Self = Self(3);
    pub const INDESTRUCTIBLE: Self = Self(4);
    pub const COAL: Self = Self(100);
    pub const OIL_SHALE: Self = Self(101);
    pub const LIMESTONE: Self = Self(102);
    pub const WATER: Self = Self(107);
    pub const WARP_GATE: Self = Self(500);

    pub const Air: Self = Self::AIR;
    pub const Dirt: Self = Self::DIRT;
    pub const Grass: Self = Self::GRASS;
    pub const Stone: Self = Self::STONE;
    pub const Coal: Self = Self::COAL;
    pub const OilShale: Self = Self::OIL_SHALE;
    pub const Limestone: Self = Self::LIMESTONE;
    pub const Water: Self = Self::WATER;
    pub const Indestructible: Self = Self::INDESTRUCTIBLE;
    pub const WarpGate: Self = Self::WARP_GATE;

    #[must_use]
    pub const fn to_id(self) -> u32 {
        self.0
    }

    #[must_use]
    pub const fn from_id(id: u32) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn get_data(&self) -> Option<&BlockData> {
        BLOCK_MANAGER.get_data(self)
    }

    #[must_use]
    pub fn is_solid(&self) -> bool {
        BLOCK_MANAGER.is_solid(self)
    }

    #[must_use]
    pub fn is_liquid(&self) -> bool {
        BLOCK_MANAGER.is_liquid(self)
    }

    #[must_use]
    pub fn is_placeable(&self) -> bool {
        BLOCK_MANAGER.is_placeable(self)
    }

    #[must_use]
    pub fn get_sprite(&self) -> Option<Rect> {
        BLOCK_MANAGER.get_sprite(self)
    }

    #[must_use]
    pub fn get_base_hardness(&self) -> i32 {
        BLOCK_MANAGER.get_base_hardness(self)
    }

    #[must_use]
    pub fn get_map_color(&self) -> Color {
        BLOCK_MANAGER.get_map_color(self)
    }

    #[must_use]
    pub fn from_item_type(item_type: &str) -> Option<Self> {
        BLOCK_MANAGER.get_by_item_type(item_type)
    }
}

pub static BLOCK_MANAGER: LazyLock<BlockManager> = LazyLock::new(BlockManager::new);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockData {
    pub id: u32,
    pub key: String,
    pub is_solid: bool,
    pub is_placeable: bool,
    pub base_hardness: i32,
    pub sprite: Option<BlockRect>,
    pub item_type: Option<String>,
    pub weight: i32,
    #[serde(default = "default_tick_interval")]
    pub tick_interval: u32,
    #[serde(default = "default_map_color")]
    pub map_color: [u8; 3],
}

const fn default_tick_interval() -> u32 {
    1
}

const fn default_map_color() -> [u8; 3] {
    [0, 0, 0]
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
        Self::new(br.x, br.y, br.w, br.h)
    }
}

pub struct BlockManager {
    blocks: HashMap<u32, BlockData>,
    item_type_to_id: HashMap<String, u32>,
    liquid_ids: HashSet<u32>,
}

impl Default for BlockManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockManager {
    #[must_use]
    pub fn new() -> Self {
        let mut manager = Self {
            blocks: HashMap::new(),
            item_type_to_id: HashMap::new(),
            liquid_ids: HashSet::new(),
        };
        manager.load_blocks();
        manager
    }

    fn load_blocks(&mut self) {
        let categories = ["solid", "liquid", "gas", "special"];
        for category in categories {
            let path = format!("data/blocks/{category}.json");
            if let Ok(content) = fs::read_to_string(path)
                && let Ok(blocks_vec) = serde_json::from_str::<Vec<BlockData>>(&content)
            {
                for block in blocks_vec {
                    if let Some(ref it) = block.item_type {
                        self.item_type_to_id.insert(it.clone(), block.id);
                    }
                    if category == "liquid" {
                        self.liquid_ids.insert(block.id);
                    }
                    self.blocks.insert(block.id, block);
                }
            }
        }
    }

    #[must_use]
    pub fn get_data(&self, block_type: &BlockType) -> Option<&BlockData> {
        self.blocks.get(&block_type.to_id())
    }

    #[must_use]
    pub fn get_by_item_type(&self, item_type: &str) -> Option<BlockType> {
        self.item_type_to_id
            .get(item_type)
            .map(|&id| BlockType::from_id(id))
    }

    #[must_use]
    pub fn is_solid(&self, block_type: &BlockType) -> bool {
        self.get_data(block_type).is_some_and(|d| d.is_solid)
    }

    #[must_use]
    pub fn is_liquid(&self, block_type: &BlockType) -> bool {
        self.liquid_ids.contains(&block_type.to_id())
    }

    #[must_use]
    pub fn is_placeable(&self, block_type: &BlockType) -> bool {
        self.get_data(block_type).is_some_and(|d| d.is_placeable)
    }

    #[must_use]
    pub fn get_sprite(&self, block_type: &BlockType) -> Option<Rect> {
        self.get_data(block_type)
            .and_then(|d| d.sprite.map(Rect::from))
    }

    #[must_use]
    pub fn get_base_hardness(&self, block_type: &BlockType) -> i32 {
        self.get_data(block_type).map_or(0, |d| d.base_hardness)
    }

    #[must_use]
    pub fn get_map_color(&self, block_type: &BlockType) -> Color {
        self.get_data(block_type).map_or(BLACK, |d| {
            Color::from_rgba(d.map_color[0], d.map_color[1], d.map_color[2], 255)
        })
    }

    #[must_use]
    pub fn get_weight(&self, block_type: &BlockType) -> i32 {
        self.get_data(block_type).map_or(0, |d| d.weight)
    }
}
