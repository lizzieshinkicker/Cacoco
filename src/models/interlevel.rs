use crate::models::deserialize_null_default;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelDefFile {
    #[serde(skip)]
    pub screens: Vec<InterlevelScreen>,
}

impl InterlevelDefFile {
    pub fn new_empty() -> Self {
        Self {
            screens: vec![InterlevelScreen {
                name: "INTERLEV".to_string(),
                version: "1.0.0".to_string(),
                metadata: serde_json::json!({}),
                data: InterlevelDefinition::default(),
            }],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelScreen {
    #[serde(skip)]
    pub name: String,
    pub version: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub metadata: serde_json::Value,
    pub data: InterlevelDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelDefinition {
    pub music: String,
    pub backgroundimage: String,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub layers: Vec<InterlevelLayer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelLayer {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _cacoco_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub anims: Vec<InterlevelAnim>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub conditions: Vec<InterlevelCondition>,
}

impl InterlevelLayer {
    pub fn display_name(&self, default_idx: usize) -> String {
        self._cacoco_name
            .clone()
            .unwrap_or_else(|| format!("Layer {}", default_idx))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelAnim {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _cacoco_name: Option<String>,
    pub x: i32,
    pub y: i32,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub frames: Vec<InterlevelFrame>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub conditions: Vec<InterlevelCondition>,
}

impl InterlevelAnim {
    pub fn display_name(&self, default_idx: usize) -> String {
        self._cacoco_name
            .clone()
            .unwrap_or_else(|| format!("Animation {}", default_idx))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelFrame {
    pub image: String,
    #[serde(rename = "type")]
    pub frame_type: i32,
    pub duration: f64,
    #[serde(default)]
    pub maxduration: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelCondition {
    pub condition: i32,
    pub param: i32,
}
