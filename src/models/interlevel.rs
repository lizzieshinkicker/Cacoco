use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelDefFile {
    pub version: String,
    pub music: String,
    pub backgroundimage: String,
    #[serde(default)]
    pub layers: Vec<InterlevelLayer>,
}

impl InterlevelDefFile {
    pub fn new_empty() -> Self {
        Self {
            version: "1.0.0".to_string(),
            music: "D_INTER".to_string(),
            backgroundimage: "INTERPIC".to_string(),
            layers: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelLayer {
    pub anims: Vec<InterlevelAnim>,
    #[serde(default)]
    pub conditions: Vec<InterlevelCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelAnim {
    pub x: i32,
    pub y: i32,
    pub frames: Vec<InterlevelFrame>,
    #[serde(default)]
    pub conditions: Vec<InterlevelCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelFrame {
    pub image: String,
    #[serde(rename = "type")]
    pub frame_type: i32, // Bitfield: 1-Infinite, 2-Fixed, 4-Random, 0x1000-RandomFirst, 0x8000000-Widescreen
    pub duration: f64,
    #[serde(default)]
    pub maxduration: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelCondition {
    pub condition: i32,
    pub param: i32,
}
