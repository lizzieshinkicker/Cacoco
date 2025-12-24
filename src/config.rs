use std::fs;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    pub base_wad_path: Option<String>,
    #[serde(default)]
    pub source_ports: Vec<String>,
    #[serde(default)]
    pub recent_files: VecDeque<String>,
}

impl AppConfig {
    const FILENAME: &'static str = "cacoco_config.json";

    pub fn load() -> Self {
        if let Ok(content) = fs::read_to_string(Self::FILENAME) {
            if let Ok(cfg) = serde_json::from_str(&content) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(Self::FILENAME, content);
            println!("Config saved to {}", Self::FILENAME);
        }
    }
}