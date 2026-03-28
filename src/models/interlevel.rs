use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

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
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub anims: Vec<InterlevelAnim>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub conditions: Vec<InterlevelCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct InterlevelAnim {
    pub x: i32,
    pub y: i32,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub frames: Vec<InterlevelFrame>,
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub conditions: Vec<InterlevelCondition>,
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
