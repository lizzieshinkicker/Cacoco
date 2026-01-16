use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkyType {
    #[default]
    Normal = 0,
    Fire = 1,
    WithForeground = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SkyDefsFile {
    pub version: String,
    pub skies: Vec<SkyDef>,
    #[serde(default)]
    pub flatmapping: Vec<FlatMap>,
}

impl SkyDefsFile {
    pub fn new_empty() -> Self {
        Self {
            version: "1.0.0".to_string(),
            skies: vec![],
            flatmapping: vec![],
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
