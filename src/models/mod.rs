pub mod finale;
pub mod interlevel;
pub mod sbardef;
pub mod skydefs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProjectData {
    #[serde(rename = "statusbar")]
    StatusBar(sbardef::SBarDefFile),
    #[serde(rename = "finale")]
    Finale(finale::FinaleDefFile),
    #[serde(rename = "skydefs")]
    Sky(skydefs::SkyDefsFile),
    #[serde(rename = "interlevel")]
    Interlevel(interlevel::InterlevelDefFile),
}

#[allow(dead_code)]
impl ProjectData {
    pub fn standard_lump_name(&self) -> &str {
        match self {
            ProjectData::StatusBar(_) => "SBARDEF",
            ProjectData::Sky(_) => "SKYDEFS",
            ProjectData::Interlevel(_) => "INTERLEVEL",
            ProjectData::Finale(_) => "FINALE",
        }
    }

    pub fn version(&self) -> &str {
        match self {
            ProjectData::StatusBar(f) => &f.version,
            ProjectData::Finale(f) => &f.version,
            ProjectData::Sky(f) => &f.version,
            ProjectData::Interlevel(f) => &f.version,
        }
    }

    pub fn target(&self) -> sbardef::ExportTarget {
        match self {
            ProjectData::StatusBar(f) => f.target,
            _ => sbardef::ExportTarget::Extended,
        }
    }

    pub fn set_target(&mut self, target: sbardef::ExportTarget) {
        if let ProjectData::StatusBar(f) = self {
            f.target = target;
        }
    }

    pub fn determine_target(&self) -> sbardef::ExportTarget {
        match self {
            ProjectData::StatusBar(f) => f.determine_target(),
            _ => sbardef::ExportTarget::Extended,
        }
    }

    pub fn normalize_for_target(&mut self) {
        if let ProjectData::StatusBar(f) = self {
            f.normalize_for_target();
        }
    }

    pub fn as_sbar(&self) -> Option<&sbardef::SBarDefFile> {
        if let ProjectData::StatusBar(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_sbar_mut(&mut self) -> Option<&mut sbardef::SBarDefFile> {
        if let ProjectData::StatusBar(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_sky(&self) -> Option<&skydefs::SkyDefsFile> {
        if let ProjectData::Sky(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_sky_mut(&mut self) -> Option<&mut skydefs::SkyDefsFile> {
        if let ProjectData::Sky(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_interlevel(&self) -> Option<&interlevel::InterlevelDefFile> {
        if let ProjectData::Interlevel(i) = self {
            Some(i)
        } else {
            None
        }
    }

    pub fn as_finale(&self) -> Option<&finale::FinaleDefFile> {
        if let ProjectData::Finale(f) = self {
            Some(f)
        } else {
            None
        }
    }

    pub fn to_sanitized_json(&self, assets: &crate::assets::AssetStore) -> String {
        match self {
            ProjectData::StatusBar(f) => f.to_sanitized_json(assets),
            ProjectData::Sky(f) => self.wrap_lump("skydefs", &f.version, &f.metadata, &f.data),
            ProjectData::Interlevel(f) => {
                self.wrap_lump("interlevel", &f.version, &f.metadata, &f.data)
            }
            ProjectData::Finale(f) => self.wrap_lump("finale", &f.version, &f.metadata, &f.data),
        }
    }

    fn wrap_lump(
        &self,
        lump_type: &str,
        version: &str,
        metadata: &serde_json::Value,
        data: &impl serde::Serialize,
    ) -> String {
        let mut data_val = serde_json::to_value(data).unwrap_or_default();
        sanitize_json_value(&mut data_val);

        let mut root = serde_json::Map::new();
        root.insert("type".to_string(), serde_json::json!(lump_type));
        root.insert("version".to_string(), serde_json::json!(version));

        if metadata.is_null() {
            root.insert("metadata".to_string(), serde_json::json!({}));
        } else {
            root.insert("metadata".to_string(), metadata.clone());
        }

        root.insert("data".to_string(), data_val);

        serde_json::to_string_pretty(&root).unwrap_or_default()
    }
}

fn sanitize_json_value(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.retain(|_, val| !val.is_null());
        for value in obj.values_mut() {
            sanitize_json_value(value);
        }
    } else if let Some(arr) = v.as_array_mut() {
        for value in arr {
            sanitize_json_value(value);
        }
    } else if let Some(n) = v.as_f64() {
        if n.fract() == 0.0 {
            *v = serde_json::json!(n as i64);
        }
    }
}
