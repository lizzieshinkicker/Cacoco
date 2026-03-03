use crate::render::palette::DoomPalette;
use rand::RngExt;
use std::ops::Sub;

#[derive(Debug, Clone)]
pub struct FireSimulation {
    pub width: u32,
    pub height: u32,
    pub heat_buffer: Vec<u8>,
    pub last_step_time: f64,
}

impl FireSimulation {
    pub fn new(width: u32, height: u32, current_time: f64) -> Self {
        let mut heat = vec![0u8; (width * height) as usize];
        let start = ((height - 1) * width) as usize;
        for i in start..heat.len() {
            heat[i] = 36;
        }
        Self {
            width,
            height,
            heat_buffer: heat,
            last_step_time: current_time,
        }
    }

    pub fn step(&mut self) {
        let mut rng = rand::rng();
        let w = self.width as usize;
        let h = self.height as usize;

        for x in 0..w {
            for y in 1..h {
                let src = y * w + x;
                let pixel = self.heat_buffer[src];

                if pixel == 0 {
                    if src >= w {
                        self.heat_buffer[src - w] = 0;
                    }
                } else {
                    let r: usize = rng.random_range(0..4);
                    let decay = (r & 1) as u8;
                    let target_idx = (src as isize - w as isize - r as isize + 1).max(0) as usize;
                    if target_idx < self.heat_buffer.len() {
                        self.heat_buffer[target_idx] = pixel.saturating_sub(decay);
                    }
                }
            }
        }
    }

    pub fn generate_rgba(&self, palette_indices: &[i32], palette: &DoomPalette) -> Vec<u8> {
        let mut rgba = vec![0u8; (self.width * self.height * 4) as usize];
        let max_intensity = (palette_indices.len() as f32).sub(1.0).max(1.0);

        for (i, &heat) in self.heat_buffer.iter().enumerate() {
            let lookup_idx = (heat as f32 / 36.0 * max_intensity).round() as usize;
            let pal_idx = palette_indices.get(lookup_idx).cloned().unwrap_or(0);

            let color = palette.get(pal_idx as u8);
            let out_idx = i * 4;
            rgba[out_idx] = color.r();
            rgba[out_idx + 1] = color.g();
            rgba[out_idx + 2] = color.b();

            rgba[out_idx + 3] = if pal_idx == 0 { 0 } else { 255 };
        }
        rgba
    }
}
