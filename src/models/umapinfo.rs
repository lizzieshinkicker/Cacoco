use serde::{Deserialize, Serialize};

/// Represents the different types of keys available in a UMAPINFO map entry.
/// This modular approach allows the editor to handle any key-value pair
/// defined in the UMAPINFO spec without rigid struct constraints.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum UmapField {
    LevelName(String),
    Author(String),
    /// Compatible with SKYDEFS if that lump exists.
    SkyTexture(String),
    LevelPic(String),
    Music(String),
    ExitPic(String),
    EnterPic(String),
    EndPic(String),
    InterBackdrop(String),
    InterMusic(String),
    Next(String),
    NextSecret(String),
    /// Can be a string or "clear"
    Label(String),
    /// Can be a string or "clear"
    InterTextSecret(String),
    ParTime(i32),
    EndGame(bool),
    EndBunny(bool),
    EndCast(bool),
    NoIntermission(bool),
    /// A list of lines for the intermission screen.
    InterText(Vec<String>),
    /// A menu entry for selecting episodes.
    Episode {
        patch: String,
        name: String,
        key: String,
    },
    /// A death trigger for boss monsters using mnemonics or raw IDs.
    BossAction {
        thing: String,
        special: i32,
        tag: i32,
    },
    /// A death trigger for boss monsters using Editor Numbers.
    BossActionEdNum {
        ednum: String,
        special: i32,
        tag: i32,
    },
}

impl UmapField {
    /// Returns the standard UMAPINFO key name for this field.
    pub fn key_name(&self) -> &'static str {
        match self {
            UmapField::LevelName(_) => "levelname",
            UmapField::Author(_) => "author",
            UmapField::LevelPic(_) => "levelpic",
            UmapField::SkyTexture(_) => "skytexture",
            UmapField::Music(_) => "music",
            UmapField::ExitPic(_) => "exitpic",
            UmapField::EnterPic(_) => "enterpic",
            UmapField::EndPic(_) => "endpic",
            UmapField::InterBackdrop(_) => "interbackdrop",
            UmapField::InterMusic(_) => "intermusic",
            UmapField::Next(_) => "next",
            UmapField::NextSecret(_) => "nextsecret",
            UmapField::Label(_) => "label",
            UmapField::InterTextSecret(_) => "intertextsecret",
            UmapField::ParTime(_) => "partime",
            UmapField::EndGame(_) => "endgame",
            UmapField::EndBunny(_) => "endbunny",
            UmapField::EndCast(_) => "endcast",
            UmapField::NoIntermission(_) => "nointermission",
            UmapField::InterText(_) => "intertext",
            UmapField::Episode { .. } => "episode",
            UmapField::BossAction { .. } => "bossaction",
            UmapField::BossActionEdNum { .. } => "bossactionednum",
        }
    }

    /// Returns a mutable reference to the inner string if the field is a simple text type.
    /// Excludes Label and InterTextSecret which have special "Clear" logic.
    pub fn as_string_mut(&mut self) -> Option<&mut String> {
        match self {
            UmapField::LevelName(s)
            | UmapField::Author(s)
            | UmapField::SkyTexture(s)
            | UmapField::Music(s)
            | UmapField::Next(s)
            | UmapField::NextSecret(s)
            | UmapField::ExitPic(s)
            | UmapField::EnterPic(s)
            | UmapField::LevelPic(s)
            | UmapField::EndPic(s)
            | UmapField::InterBackdrop(s)
            | UmapField::InterMusic(s) => Some(s),
            _ => None,
        }
    }
}

/// A single map definition block in the UMAPINFO lump.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MapEntry {
    /// The map identifier (e.g., MAP01 or E1M1).
    pub mapname: String,
    /// The collection of Bespoke keys defined for this map.
    pub fields: Vec<UmapField>,
}

/// The root structure for a UMAPINFO project lump.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UmapInfoFile {
    pub version: String,
    pub metadata: serde_json::Value,
    pub data: UmapInfoDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct UmapInfoDefinition {
    pub maps: Vec<MapEntry>,
}

impl UmapInfoFile {
    /// Creates a new, empty UMAPINFO file with a default MAP01 entry.
    pub fn new_empty() -> Self {
        Self {
            version: "1.0.0".to_string(),
            metadata: serde_json::json!({}),
            data: UmapInfoDefinition {
                maps: vec![MapEntry {
                    mapname: "MAP01".to_string(),
                    fields: vec![UmapField::LevelName("Entryway".to_string())],
                }],
            },
        }
    }

    /// Serializes the modular data model into the standard UMAPINFO plaintext format.
    pub fn to_umapinfo_text(&self) -> String {
        let mut out = String::new();
        for map in &self.data.maps {
            out.push_str(&format!("map {}\n{{\n", map.mapname));
            for field in &map.fields {
                match field {
                    UmapField::InterText(lines) => {
                        if lines.len() == 1 && lines[0] == "clear" {
                            out.push_str("\tintertext = clear\n");
                        } else {
                            out.push_str("\tintertext = ");
                            for (i, line) in lines.iter().enumerate() {
                                let sep = if i == 0 { "" } else { ",\n\t\t" };
                                out.push_str(&format!("{}\"{}\"", sep, line));
                            }
                            out.push_str("\n");
                        }
                    }
                    UmapField::Episode { patch, name, key } => {
                        if patch == "clear" {
                            out.push_str("\tepisode = clear\n");
                        } else {
                            out.push_str(&format!(
                                "\tepisode = \"{}\", \"{}\", \"{}\"\n",
                                patch, name, key
                            ));
                        }
                    }
                    UmapField::BossAction {
                        thing,
                        special,
                        tag,
                    } => {
                        out.push_str(&format!("\tbossaction = {}, {}, {}\n", thing, special, tag));
                    }
                    UmapField::BossActionEdNum {
                        ednum,
                        special,
                        tag,
                    } => {
                        out.push_str(&format!(
                            "\tbossactionednum = {}, {}, {}\n",
                            ednum, special, tag
                        ));
                    }
                    UmapField::ParTime(v) => {
                        out.push_str(&format!("\t{} = {}\n", field.key_name(), v))
                    }
                    UmapField::EndGame(v)
                    | UmapField::EndBunny(v)
                    | UmapField::EndCast(v)
                    | UmapField::NoIntermission(v) => {
                        let bool_val = if *v { "true" } else { "false" };
                        out.push_str(&format!("\t{} = {}\n", field.key_name(), bool_val));
                    }
                    UmapField::Label(v) | UmapField::InterTextSecret(v) => {
                        if v == "clear" {
                            out.push_str(&format!("\t{} = clear\n", field.key_name()));
                        } else {
                            out.push_str(&format!("\t{} = \"{}\"\n", field.key_name(), v));
                        }
                    }
                    _ => {
                        let val_str = match field {
                            UmapField::LevelName(s)
                            | UmapField::Author(s)
                            | UmapField::SkyTexture(s)
                            | UmapField::Music(s)
                            | UmapField::ExitPic(s)
                            | UmapField::EnterPic(s)
                            | UmapField::LevelPic(s)
                            | UmapField::EndPic(s)
                            | UmapField::InterBackdrop(s)
                            | UmapField::InterMusic(s)
                            | UmapField::Next(s)
                            | UmapField::NextSecret(s) => s,
                            _ => "",
                        };
                        out.push_str(&format!("\t{} = \"{}\"\n", field.key_name(), val_str));
                    }
                }
            }
            out.push_str("}\n\n");
        }
        out
    }

    /// Attempts to parse a plaintext UMAPINFO lump into the internal modular model.
    pub fn from_umapinfo_text(text: &str) -> Self {
        let mut file = Self {
            version: "1.0.0".to_string(),
            ..Default::default()
        };
        let mut current_map: Option<MapEntry> = None;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue;
            }

            if line.to_uppercase().starts_with("MAP") {
                if let Some(m) = current_map.take() {
                    file.data.maps.push(m);
                }
                let name = line[3..].trim().trim_matches('{').trim().to_string();
                current_map = Some(MapEntry {
                    mapname: name,
                    fields: vec![],
                });
                continue;
            }

            if let Some(m) = current_map.as_mut() {
                if line == "}" {
                    file.data.maps.push(current_map.take().unwrap());
                    continue;
                }

                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim().to_lowercase();
                    let val_raw = val.trim();

                    let val_clean = val_raw.trim_matches('"').to_string();

                    let field = match key.as_str() {
                        "levelname" => Some(UmapField::LevelName(val_clean)),
                        "author" => Some(UmapField::Author(val_clean)),
                        "skytexture" => Some(UmapField::SkyTexture(val_clean)),
                        "music" => Some(UmapField::Music(val_clean)),
                        "levelpic" => Some(UmapField::LevelPic(val_clean)),
                        "next" => Some(UmapField::Next(val_clean)),
                        "nextsecret" => Some(UmapField::NextSecret(val_clean)),
                        "label" => Some(UmapField::Label(val_clean)),
                        "intertextsecret" => Some(UmapField::InterTextSecret(val_clean)),
                        "exitpic" => Some(UmapField::ExitPic(val_clean)),
                        "enterpic" => Some(UmapField::EnterPic(val_clean)),
                        "endpic" => Some(UmapField::EndPic(val_clean)),
                        "interbackdrop" => Some(UmapField::InterBackdrop(val_clean)),
                        "intermusic" => Some(UmapField::InterMusic(val_clean)),
                        "partime" => val_clean.parse::<i32>().ok().map(UmapField::ParTime),
                        "endgame" => Some(UmapField::EndGame(val_clean.to_lowercase() == "true")),
                        "endbunny" => Some(UmapField::EndBunny(val_clean.to_lowercase() == "true")),
                        "endcast" => Some(UmapField::EndCast(val_clean.to_lowercase() == "true")),
                        "nointermission" => Some(UmapField::NoIntermission(
                            val_clean.to_lowercase() == "true",
                        )),
                        "episode" => {
                            if val_clean.to_lowercase() == "clear" {
                                Some(UmapField::Episode {
                                    patch: "clear".into(),
                                    name: "".into(),
                                    key: "".into(),
                                })
                            } else {
                                let parts: Vec<String> = val_raw
                                    .split(',')
                                    .map(|s| s.trim().trim_matches('"').to_string())
                                    .collect();
                                if parts.len() >= 3 {
                                    Some(UmapField::Episode {
                                        patch: parts[0].clone(),
                                        name: parts[1].clone(),
                                        key: parts[2].clone(),
                                    })
                                } else {
                                    None
                                }
                            }
                        }
                        _ => None,
                    };

                    if let Some(f) = field {
                        m.fields.push(f);
                    }
                }
            }
        }

        if let Some(m) = current_map {
            file.data.maps.push(m);
        }
        file
    }
}
