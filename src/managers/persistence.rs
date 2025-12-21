use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockSaveData {
    pub x: f32,
    pub y: f32,
    pub current_hp: i32,
    pub sprite_id: String,
    pub name: Option<String>,
}

// We use simplified threading or just sync IO for MVP because Macroquad is single threaded mostly,
// but we can spawn threads for IO.
// The Python version used threads. We can too, but need Arc<Mutex<...>> for shared state.
// To keep things simple and avoid complex RefCell/Arc spaghetti in the port,
// we'll do synchronous IO first, or basic threaded IO that returns a result to a channel/shared var.

pub struct PersistenceManager {
    pub is_saving: bool,
    pub is_loading: bool,
    save_result: Arc<Mutex<Option<(bool, String)>>>,
    load_result: Arc<Mutex<Option<(bool, Value)>>>,
}

impl PersistenceManager {
    pub fn new() -> Self {
        Self {
            is_saving: false,
            is_loading: false,
            save_result: Arc::new(Mutex::new(None)),
            load_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn list_save_files() -> Vec<String> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension()
                    && ext == "json"
                    && let Some(stem) = path.file_stem()
                    && let Some(_str_stem) = stem.to_str()
                {
                    // Filter: Only include files that have is_save_file: true
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(json) = serde_json::from_str::<Value>(&content) {
                            if json.get("is_save_file").and_then(|v| v.as_bool()) == Some(true) {
                                if let Some(file_name) = path.file_name()
                                    && let Some(name_str) = file_name.to_str()
                                {
                                    files.push(name_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        files
    }

    pub fn save_game(&mut self, filename: String, data: Value) {
        if self.is_saving {
            return;
        }

        self.is_saving = true;
        let result_clone = self.save_result.clone();

        thread::spawn(move || {
            let json_str = serde_json::to_string_pretty(&data).unwrap_or_default();
            let res = match fs::write(&filename, json_str) {
                Ok(_) => (true, "Save Successful".to_string()),
                Err(e) => (false, e.to_string()),
            };
            let mut lock = result_clone.lock().unwrap();
            *lock = Some(res);
        });
    }

    pub fn load_game(&mut self, filename: String) {
        if self.is_loading {
            return;
        }

        self.is_loading = true;
        let result_clone = self.load_result.clone();

        thread::spawn(move || {
            let res = match fs::read_to_string(&filename) {
                Ok(content) => match serde_json::from_str::<Value>(&content) {
                    Ok(v) => {
                        if v.get("is_save_file").and_then(|v| v.as_bool()) == Some(true) {
                            (true, v)
                        } else {
                            (false, Value::String("Invalid save file format".to_string()))
                        }
                    }
                    Err(e) => (false, Value::String(format!("Parse error: {}", e))),
                },
                Err(e) => (false, Value::String(format!("Read error: {}", e))),
            };
            let mut lock = result_clone.lock().unwrap();
            *lock = Some(res);
        });
    }

    pub fn check_save_status(&mut self) -> Option<(bool, String)> {
        let mut lock = self.save_result.lock().unwrap();
        if lock.is_some() {
            self.is_saving = false;
            return lock.take();
        }
        None
    }

    pub fn check_load_status(&mut self) -> Option<(bool, Value)> {
        let mut lock = self.load_result.lock().unwrap();
        if lock.is_some() {
            self.is_loading = false;
            return lock.take();
        }
        None
    }
}
