use serde::{Deserialize, Serialize};

// --- From src/managers/block.rs ---
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockType(pub u32);

// --- From src/components.rs ---
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarpGate {
    pub x: f32,
    pub y: f32,
    pub name: String,
}

// --- From src/managers/persistence.rs ---
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockSaveData {
    pub i: u32, // index in chunk
    pub t: BlockType,
    pub n: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkSaveData {
    pub cx: i32,
    pub cy: i32,
    pub blocks: Vec<u32>,
    pub named_blocks: Vec<BlockSaveData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ItemStack {
    pub item_type: String,
    pub count: u32,
    pub is_natural: bool,
    pub is_auto_stored: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SaveData {
    pub version: u32,
    pub camera_x: f32,
    pub camera_y: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub player_money: i32,
    pub player_fuel: f32,
    pub player_max_fuel: f32,
    pub player_max_cargo: i32,
    pub player_max_storage: i32,
    pub player_drill_level: i32,
    pub player_tank_level: i32,
    pub player_engine_level: i32,
    pub player_cargo_level: i32,
    pub player_warp_gates: Vec<WarpGate>,
    pub player_cargo: Vec<ItemStack>,
    pub player_storage: Vec<ItemStack>,
    pub world_seed_main: u32,
    pub world_seed_ore: u32,
    pub modified_chunks: Vec<ChunkSaveData>,
}
