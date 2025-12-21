use crate::components::WarpGate;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

pub const SAVE_VERSION: u32 = 1;
pub const SAVE_DIR: &str = "saves";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockSaveData {
    pub i: u32, // index in chunk
    pub t: crate::components::BlockType,
    pub n: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChunkSaveData {
    pub cx: i32,
    pub cy: i32,
    /// Flat RLE encoded blocks: [type_id, count, type_id, count, ...]
    /// This represents the entire chunk state efficiently in a single array.
    pub blocks: Vec<u32>,
    /// Special blocks that need additional data (like names)
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

pub struct PersistenceManager {
    pub is_saving: bool,
    pub is_loading: bool,
    save_result: Arc<Mutex<Option<Result<String, String>>>>,
    load_result: Arc<Mutex<Option<Result<SaveData, String>>>>,
}

impl PersistenceManager {
    pub fn new() -> Self {
        if !Path::new(SAVE_DIR).exists() {
            let _ = fs::create_dir_all(SAVE_DIR);
        }
        Self {
            is_saving: false,
            is_loading: false,
            save_result: Arc::new(Mutex::new(None)),
            load_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn list_save_files() -> Vec<String> {
        let mut files = Vec::new();
        let path = Path::new(SAVE_DIR);
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && ext == "json"
                    && let Ok(content) = fs::read_to_string(&path)
                    && let Ok(data) = serde_json::from_str::<serde_json::Value>(&content)
                    && (data.get("version").is_some() || data.get("is_save_file").is_some())
                    && let Some(file_name) = path.file_name()
                    && let Some(name_str) = file_name.to_str()
                {
                    files.push(name_str.to_string());
                }
            }
        }
        files
    }

    pub fn save_game(&mut self, filename: String, data: SaveData) {
        if self.is_saving {
            return;
        }

        self.is_saving = true;
        let result_clone = self.save_result.clone();

        thread::spawn(move || {
            let path = Path::new(SAVE_DIR).join(&filename);
            let temp_path = path.with_extension("tmp");

            let res = (|| {
                let json_str = serde_json::to_string_pretty(&data)?;
                fs::write(&temp_path, json_str)?;
                fs::rename(&temp_path, &path)?;
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })();

            let res_final = match res {
                Ok(_) => Ok("Save Successful".to_string()),
                Err(e) => Err(e.to_string()),
            };

            let mut lock = result_clone.lock().unwrap();
            *lock = Some(res_final);
        });
    }

    pub fn load_game(&mut self, filename: String) {
        if self.is_loading {
            return;
        }

        self.is_loading = true;
        let result_clone = self.load_result.clone();

        thread::spawn(move || {
            let path = Path::new(SAVE_DIR).join(&filename);
            let res = (|| {
                let content = fs::read_to_string(path)?;
                let data: SaveData = serde_json::from_str(&content)?;
                Ok::<SaveData, Box<dyn std::error::Error + Send + Sync>>(data)
            })();

            let res_final = match res {
                Ok(data) => Ok(data),
                Err(e) => Err(e.to_string()),
            };

            let mut lock = result_clone.lock().unwrap();
            *lock = Some(res_final);
        });
    }

    pub fn check_save_status(&mut self) -> Option<Result<String, String>> {
        let mut lock = self.save_result.lock().unwrap();
        if lock.is_some() {
            self.is_saving = false;
            return lock.take();
        }
        None
    }

    pub fn check_load_status(&mut self) -> Option<Result<SaveData, String>> {
        let mut lock = self.load_result.lock().unwrap();
        if lock.is_some() {
            self.is_loading = false;
            return lock.take();
        }
        None
    }
}
