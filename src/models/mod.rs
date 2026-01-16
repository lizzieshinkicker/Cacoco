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

impl ProjectData {
    #[allow(dead_code)]
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
        match self {
            ProjectData::StatusBar(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_sbar_mut(&mut self) -> Option<&mut sbardef::SBarDefFile> {
        match self {
            ProjectData::StatusBar(s) => Some(s),
            _ => None,
        }
    }

    pub fn to_sanitized_json(&self, assets: &crate::assets::AssetStore) -> String {
        match self {
            ProjectData::StatusBar(f) => f.to_sanitized_json(assets),
            _ => "{}".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn get_element(&self, path: &[usize]) -> Option<&sbardef::ElementWrapper> {
        self.as_sbar().and_then(|s| s.get_element(path))
    }

    #[allow(dead_code)]
    pub fn get_element_mut(&mut self, path: &[usize]) -> Option<&mut sbardef::ElementWrapper> {
        self.as_sbar_mut().and_then(|s| s.get_element_mut(path))
    }
}
