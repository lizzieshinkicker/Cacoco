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
            _ => serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string()),
        }
    }
}
