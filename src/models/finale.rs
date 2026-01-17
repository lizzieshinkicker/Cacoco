use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum FinaleType {
    #[default]
    ArtScreen = 0,
    BunnyScroller = 1,
    CastRollCall = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FinaleDefFile {
    pub version: String,
    pub metadata: serde_json::Value,
    pub data: FinaleDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FinaleDefinition {
    #[serde(rename = "type")]
    pub finale_type: FinaleType,
    pub music: String,
    pub background: String,
    #[serde(default)]
    pub donextmap: bool,
    pub bunny: Option<BunnyDef>,
    #[serde(rename = "castrollcall")]
    pub cast_roll_call: Option<CastRollCallDef>,
}

impl FinaleDefFile {
    pub fn new_empty() -> Self {
        Self {
            version: "1.0.0".to_string(),
            metadata: serde_json::json!({}),
            data: FinaleDefinition {
                finale_type: FinaleType::ArtScreen,
                music: "D_VICTO".to_string(),
                background: "INTERPIC".to_string(),
                ..Default::default()
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BunnyDef {
    pub stitchimage: String,
    pub overlay: i32,
    pub overlaycount: i32,
    pub overlaysound: i32,
    pub overlayx: i32,
    pub overlayy: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CastRollCallDef {
    pub castmembers: Vec<CastMember>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CastMember {
    pub name: String, // DeHackEd mnemonic
    pub sound: String,
    pub alive: Vec<CastFrame>,
    pub dead: Vec<CastFrame>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CastFrame {
    pub image: String,
    pub duration: f64,
    pub translation: Option<String>,
    pub tranmap: Option<String>,
    pub sound: Option<String>,
    #[serde(default)]
    pub flip: bool,
}
