use eframe::egui;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::path::Path;

/// A lightweight, pre-hashed identifier for an asset.
///
/// Using AssetId instead of String keys in the rendering path significantly
/// improves performance by avoiding string allocations and comparisons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetId(u64);

impl AssetId {
    /// Generates a unique identifier from an asset name (case-insensitive).
    pub fn new(name: &str) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        AssetStore::stem(name).hash(&mut hasher);
        Self(hasher.finish())
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl fmt::LowerHex for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

/// A centralized registry for textures, raw data, and Doom-specific offsets.
pub struct AssetStore {
    /// Texture handles indexed by hashed AssetId.
    pub textures: HashMap<AssetId, egui::TextureHandle>,
    /// The original bytes for images, used when building PK3s.
    pub raw_files: HashMap<AssetId, Vec<u8>>,
    /// Horizontal and vertical offsets (Doom patch format).
    pub offsets: HashMap<AssetId, (i16, i16)>,
    /// A reverse-lookup to get the original filename (including extension).
    pub names: HashMap<AssetId, String>,
}

impl Default for AssetStore {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
            raw_files: HashMap::new(),
            offsets: HashMap::new(),
            names: HashMap::new(),
        }
    }
}

impl AssetStore {
    /// Standardized method to convert any path or filename into a clean Doom key.
    ///
    /// Converts to uppercase and removes file extensions.
    pub fn stem(name: &str) -> String {
        Path::new(name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(name)
            .to_uppercase()
    }

    /// Loads a standard image file (PNG/JPG) into the store.
    pub fn load_image(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        let id = AssetId::new(name);
        self.raw_files.insert(id, bytes.to_vec());

        // Preserve the full name (e.g. "graphics/my_patch.png") for PK3 export.
        self.names.insert(id, name.to_string());

        self.load_texture_only(ctx, name, bytes);
    }

    /// Loads an image as a texture handle without storing raw file bytes.
    pub fn load_reference_image(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.load_texture_only(ctx, name, bytes);
    }

    /// Internal: Decodes image bytes and uploads them to the GPU.
    fn create_and_store_texture(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        image: egui::ColorImage,
        options: egui::TextureOptions,
    ) {
        let id = AssetId::new(name);
        let handle = ctx.load_texture(name, image, options);

        self.textures.insert(id, handle);

        self.names.entry(id).or_insert_with(|| name.to_string());
    }

    fn load_image_from_bytes(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        bytes: &[u8],
        options: egui::TextureOptions,
    ) {
        match image::load_from_memory(bytes) {
            Ok(image) => {
                let size = [image.width() as _, image.height() as _];
                let image_buffer = image.to_rgba8();
                let pixels = image_buffer.as_flat_samples();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                self.create_and_store_texture(ctx, name, color_image, options);
            }
            Err(e) => {
                eprintln!("!!! FAILED TO LOAD IMAGE '{}': {}", name, e);
            }
        }
    }

    fn load_texture_only(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.load_image_from_bytes(ctx, name, bytes, egui::TextureOptions::NEAREST);
    }

    /// Loads an image with linear filtering.
    pub fn load_smooth_image(&mut self, ctx: &egui::Context, name: &str, bytes: &[u8]) {
        self.load_image_from_bytes(ctx, name, bytes, egui::TextureOptions::LINEAR);
    }

    /// Directly loads raw RGBA pixels into the texture store.
    pub fn load_rgba(
        &mut self,
        ctx: &egui::Context,
        name: &str,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) {
        if pixels.len() != (width * height * 4) as usize {
            return;
        }

        let color_image =
            egui::ColorImage::from_rgba_unmultiplied([width as _, height as _], pixels);
        self.create_and_store_texture(ctx, name, color_image, egui::TextureOptions::NEAREST);
    }

    /// Loads raw RGBA pixels and registers a Doom patch offset.
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
        let id = AssetId::new(name);
        self.offsets.insert(id, (left, top));
        self.load_rgba(ctx, name, width, height, pixels);
    }

    /// Pre-loads built-in application icons and badges.
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

    /// Resolves a single character into a pre-hashed AssetId.
    pub fn resolve_patch_id(&self, stem: &str, c: char, is_number_font: bool) -> AssetId {
        let c_upper = c.to_ascii_uppercase();

        if is_number_font {
            match c_upper {
                '-' => {
                    let p1 = AssetId::new(&format!("{}MINUS", stem));
                    if self.textures.contains_key(&p1) {
                        return p1;
                    }
                    AssetId::new(&format!("{}-", stem))
                }
                '%' => {
                    let variants = ["PRCNT", "PRCN", "PCNT", "PERCENT", "%"];
                    for v in variants {
                        let p = AssetId::new(&format!("{}{}", stem, v));
                        if self.textures.contains_key(&p) {
                            return p;
                        }
                    }
                    AssetId::new(&format!("{}%", stem))
                }
                '0'..='9' => {
                    let p1 = AssetId::new(&format!("{}NUM{}", stem, c_upper));
                    if self.textures.contains_key(&p1) {
                        return p1;
                    }
                    AssetId::new(&format!("{}{}", stem, c_upper))
                }
                _ => AssetId::new(&format!("{}{}", stem, c_upper)),
            }
        } else {
            AssetId::new(&format!("{}{:03}", stem, c_upper as u32))
        }
    }
}
