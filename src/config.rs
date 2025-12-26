use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;

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

    fn get_config_dir() -> Option<std::path::PathBuf> {
        ProjectDirs::from("io.github", "lizzieshinkicker", "Cacoco")
            .map(|proj_dirs| proj_dirs.config_local_dir().to_path_buf())
    }

    pub fn load() -> Self {
        if let Some(config_dir) = Self::get_config_dir()
            && let Ok(content) = fs::read_to_string(config_dir.join(Self::FILENAME))
            && let Ok(cfg) = serde_json::from_str(&content)
        {
            return cfg;
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(config_dir) = Self::get_config_dir() {
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }

            let filename = config_dir.join(Self::FILENAME);
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(&filename, content);
                println!("Config saved to {}", filename.to_string_lossy());
            }
        }
    }
}
