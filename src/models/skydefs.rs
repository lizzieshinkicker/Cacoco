use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum SkyType {
    #[default]
    Normal = 0,
    Fire = 1,
    WithForeground = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkyDefsFile {
    pub version: String,
    pub metadata: serde_json::Value,
    pub data: SkyDefsDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkyDefsDefinition {
    pub skies: Vec<SkyDef>,
    pub flatmapping: Option<Vec<FlatMap>>,
}

impl SkyDefsFile {
    pub fn new_empty() -> Self {
        Self {
            version: "1.0.0".to_string(),
            metadata: serde_json::json!({}),
            data: SkyDefsDefinition {
                skies: vec![],
                flatmapping: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkyDef {
    #[serde(rename = "type")]
    pub sky_type: SkyType,
    pub name: String,
    pub mid: f32,
    pub scrollx: f32,
    pub scrolly: f32,
    pub scalex: f32,
    pub scaley: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fire: Option<FireSkyDef>,
    pub foregroundtex: Option<ForegroundTexDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FireSkyDef {
    pub palette: Vec<i32>,
    pub updatetime: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ForegroundTexDef {
    pub name: String,
    pub mid: f32,
    pub scrollx: f32,
    pub scrolly: f32,
    pub scalex: f32,
    pub scaley: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FlatMap {
    pub flat: String,
    pub sky: String,
}
