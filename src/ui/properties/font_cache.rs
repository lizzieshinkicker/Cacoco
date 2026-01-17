use crate::models::ProjectData;
use crate::models::sbardef::SBarDefFile;
use std::collections::HashMap;

pub struct FontCache {
    pub number_fonts: HashMap<String, String>,
    pub hud_fonts: HashMap<String, String>,
    pub number_font_names: Vec<String>,
    pub hud_font_names: Vec<String>,
}

impl FontCache {
    pub fn new(file: &SBarDefFile) -> Self {
        let mut number_fonts = HashMap::new();
        let mut number_font_names = Vec::new();
        for f in &file.data.number_fonts {
            number_fonts.insert(f.name.to_lowercase(), f.stem.clone());
            number_font_names.push(f.name.clone());
        }

        let mut hud_fonts = HashMap::new();
        let mut hud_font_names = Vec::new();
        for f in &file.data.hud_fonts {
            hud_fonts.insert(f.name.to_lowercase(), f.stem.clone());
            hud_font_names.push(f.name.clone());
        }

        Self {
            number_fonts,
            hud_fonts,
            number_font_names,
            hud_font_names,
        }
    }

    pub fn new_from_proj(project: &ProjectData) -> Self {
        if let Some(sbar) = project.as_sbar() {
            Self::new(sbar)
        } else {
            Self {
                number_fonts: HashMap::new(),
                hud_fonts: HashMap::new(),
                number_font_names: Vec::new(),
                hud_font_names: Vec::new(),
            }
        }
    }

    pub fn get_number_stem(&self, name: &str) -> Option<String> {
        self.number_fonts.get(&name.to_lowercase()).cloned()
    }

    pub fn get_hud_stem(&self, name: &str) -> Option<String> {
        self.hud_fonts.get(&name.to_lowercase()).cloned()
    }
}
