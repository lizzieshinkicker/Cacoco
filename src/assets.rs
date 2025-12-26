use eframe::egui;
use std::collections::HashMap;
use std::path::Path;

pub struct AssetStore {
    pub textures: HashMap<String, egui::TextureHandle>,
    pub raw_files: HashMap<String, Vec<u8>>,
    pub offsets: HashMap<String, (i16, i16)>,
}

impl Default for AssetStore {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
            raw_files: HashMap::new(),
            offsets: HashMap::new(),
        }
    }
}

impl AssetStore {
    /// Standardized method to convert any path or filename into a clean Doom key (Uppercase, no extension)
    pub fn stem(name: &str) -> String {
        Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_uppercase()
    }

    pub fn load_image(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.raw_files.insert(name.to_string(), bytes.to_vec());
        self.load_texture_only(ctx, name, bytes);
    }

    pub fn load_reference_image(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.load_texture_only(ctx, name, bytes);
    }

    fn create_and_store_texture(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        image: egui::ColorImage,
        options: egui::TextureOptions,
        log_label: &str,
    ) {
        let size = image.size;
        let handle = ctx.load_texture(name, image, options);
        let key = Self::stem(name);

        println!(
            "{}: '{}' -> Key: '{}' ({}x{})",
            log_label, name, key, size[0], size[1]
        );
        self.textures.insert(key, handle);
    }

    fn load_image_from_bytes(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        bytes: &[u8],
        options: egui::TextureOptions,
        log_label: &str,
    ) {
        match image::load_from_memory(bytes) {
            Ok(image) => {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                self.create_and_store_texture(ctx, name, color_image, options, log_label);
            }
            Err(e) => {
                eprintln!("!!! FAILED TO LOAD IMAGE '{}': {}", name, e);
            }
        }
    }

    fn load_texture_only(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.load_image_from_bytes(
            ctx,
            name,
            bytes,
            egui::TextureOptions::NEAREST,
            "Loaded Asset",
        );
    }

    pub fn load_smooth_image(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.load_image_from_bytes(
            ctx,
            name,
            bytes,
            egui::TextureOptions::LINEAR,
            "Loaded Smooth Asset",
        );
    }

    pub fn load_rgba(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) {
        if pixels.len() != (width * height * 4) as usize {
            eprintln!("Asset Error: Pixel buffer size mismatch for {}", name);
            return;
        }

        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], pixels);
        self.create_and_store_texture(
            ctx,
            name,
            color_image,
            egui::TextureOptions::NEAREST,
            "Loaded RGBA Asset",
        );
    }

    pub fn load_rgba_with_offset(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        width: u32,
        height: u32,
        left: i16,
        top: i16,
        pixels: &[u8],
    ) {
        let key = Self::stem(name);
        self.offsets.insert(key, (left, top));
        self.load_rgba(ctx, name, width, height, pixels);
    }

    pub fn load_system_assets(&mut self, ctx: &egui::Context) {
        self.load_reference_image(
            ctx,
            "_BADGE_ALLMAP",
            include_bytes!("../assets/badges/Allmap.png"),
        );
        self.load_reference_image(
            ctx,
            "_BADGE_BERSERK",
            include_bytes!("../assets/badges/Berserk.png"),
        );
        self.load_reference_image(
            ctx,
            "_BADGE_BLURSPHERE",
            include_bytes!("../assets/badges/BlurSphere.png"),
        );
        self.load_reference_image(
            ctx,
            "_BADGE_INVULN",
            include_bytes!("../assets/badges/Invuln.png"),
        );
        self.load_reference_image(
            ctx,
            "_BADGE_LITEAMP",
            include_bytes!("../assets/badges/LiteAmp.png"),
        );
        self.load_reference_image(
            ctx,
            "_BADGE_RADSUIT",
            include_bytes!("../assets/badges/Radsuit.png"),
        );
    }

    pub fn resolve_patch_name(&self, stem: &str, c: char, is_number_font: bool) -> String {
        let c_upper = c.to_ascii_uppercase();

        if is_number_font {
            match c_upper {
                '-' => {
                    let p1 = format!("{}MINUS", stem);
                    if self.textures.contains_key(&p1) {
                        return p1;
                    }
                    format!("{}-", stem)
                }
                '%' => {
                    let variants = ["PRCNT", "PRCN", "PCNT", "PERCENT", "%"];
                    for v in variants {
                        let p = format!("{}{}", stem, v);
                        if self.textures.contains_key(&p) {
                            return p;
                        }
                    }
                    format!("{}%", stem)
                }
                '0'..='9' => {
                    let p1 = format!("{}NUM{}", stem, c_upper);
                    if self.textures.contains_key(&p1) {
                        return p1;
                    }
                    format!("{}{}", stem, c_upper)
                }
                _ => format!("{}{}", stem, c_upper),
            }
        } else {
            format!("{}{:03}", stem, c_upper as u32).to_uppercase()
        }
    }
}
