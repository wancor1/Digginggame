use crate::constants::{DEFAULT_LANGUAGE, LANG_FOLDER};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct LanguageManager {
    pub current_lang_code: String,
    pub languages: HashMap<String, String>, // code -> display_name
    pub translations: HashMap<String, String>,
}

impl LanguageManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_lang_code: DEFAULT_LANGUAGE.to_string(),
            languages: HashMap::new(),
            translations: HashMap::new(),
        };
        manager.discover_languages();

        if !manager.load_language(DEFAULT_LANGUAGE)
            && let Some(first_key) = manager.languages.keys().next().cloned()
        {
            manager.load_language(&first_key);
        }

        manager
    }

    pub fn discover_languages(&mut self) {
        self.languages.clear();
        let path = Path::new(LANG_FOLDER);
        if !path.exists() {
            // In a real app we might create default files, simplified here
            return;
        }

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let file_stem = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();
                    let display_name = file_stem.clone(); // Fallback
                    // Try to read metadata for display name
                    // Omitted for brevity/perf, or we can read it.
                    // Python code reads it.
                    self.languages.insert(file_stem, display_name);
                }
            }
        }
    }

    pub fn _get_available_languages(&self) -> HashMap<String, String> {
        self.languages.clone()
    }

    pub fn load_language(&mut self, lang_code: &str) -> bool {
        let path = Path::new(LANG_FOLDER).join(format!("{}.json", lang_code));
        if let Ok(content) = fs::read_to_string(path)
            && let Ok(json) = serde_json::from_str::<HashMap<String, Value>>(&content)
        {
            self.translations.clear();
            for (k, v) in json {
                if let Value::String(s) = v {
                    self.translations.insert(k, s);
                }
            }
            self.current_lang_code = lang_code.to_string();
            return true;
        }
        false
    }

    pub fn get_string(&self, key: &str) -> String {
        self.translations
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    // Rust doesn't support **kwargs in the same way. We'll use a simple format replacement helper if needed,
    // or just return the string. The Python code uses .format(**kwargs).
    // Start with basic get_string.
    pub fn _get_string_fmt(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self.get_string(key);
        for (k, v) in args {
            s = s.replace(&format!("{{{}}}", k), v);
        }
        s
    }
}
