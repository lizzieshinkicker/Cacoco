use eframe::egui::Color32;

pub struct DoomPalette {
    pub colors: [Color32; 256],
}

impl Default for DoomPalette {
    fn default() -> Self {
        let mut colors = [Color32::BLACK; 256];
        for i in 0..256 {
            colors[i] = Color32::from_rgb(i as u8, i as u8, i as u8);
        }
        Self { colors }
    }
}

impl DoomPalette {
    pub fn from_raw(data: &[u8]) -> Self {
        let mut colors = [Color32::BLACK; 256];
        for i in 0..256 {
            let r = data[i * 3];
            let g = data[i * 3 + 1];
            let b = data[i * 3 + 2];
            colors[i] = Color32::from_rgb(r, g, b);
        }
        Self { colors }
    }

    pub fn get(&self, index: u8) -> Color32 {
        self.colors[index as usize]
    }
}
