use directories::ProjectDirs;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

/// Represents a configured source port for testing.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SourcePortConfig {
    /// Human-readable name.
    pub name: String,
    /// The executable path or command.
    pub command: String,
}

impl SourcePortConfig {
    /// Infers a default name from a command string.
    pub fn infer_name(command_str: &str) -> String {
        let trimmed = command_str.trim();
        if trimmed.is_empty() {
            return "Unknown Port".to_string();
        }

        if trimmed.to_lowercase().starts_with("flatpak run") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() > 2 {
                let app_id = parts[2];
                if let Some(last_part) = app_id.split('.').last() {
                    return last_part.to_string();
                }
            }
        }

        let mut path_part = if trimmed.starts_with('"') {
            trimmed.split('"').nth(1).unwrap_or(trimmed)
        } else {
            if trimmed.to_lowercase().ends_with(".exe") || trimmed.contains('\\') {
                trimmed
            } else {
                trimmed.split_whitespace().next().unwrap_or(trimmed)
            }
        };

        if let Some(exe_idx) = path_part.to_lowercase().find(".exe") {
            path_part = &path_part[..exe_idx + 4];
        }

        let stem = Path::new(path_part)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(path_part);

        if stem.is_empty() {
            return "Unknown Port".to_string();
        }

        let mut chars = stem.chars();
        match chars.next() {
            None => "Unknown Port".to_string(),
            Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AppConfig {
    pub base_wad_path: Option<String>,
    #[serde(default, deserialize_with = "deserialize_source_ports")]
    pub source_ports: Vec<SourcePortConfig>,
    #[serde(default)]
    pub recent_files: VecDeque<String>,
}

/// Custom deserializer to migrate Vec<String> to Vec<SourcePortConfig>.
fn deserialize_source_ports<'de, D>(deserializer: D) -> Result<Vec<SourcePortConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PortEntry {
        Old(String),
        New(SourcePortConfig),
    }

    let entries: Vec<PortEntry> = Vec::deserialize(deserializer)?;
    Ok(entries
        .into_iter()
        .map(|e| match e {
            PortEntry::Old(cmd) => SourcePortConfig {
                name: SourcePortConfig::infer_name(&cmd),
                command: cmd,
            },
            PortEntry::New(cfg) => cfg,
        })
        .collect())
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
