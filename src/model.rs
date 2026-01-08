use bitflags::bitflags;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};

fn default_type() -> String {
    "statusbar".to_string()
}
fn default_version() -> String {
    "1.2.0".to_string()
}

fn is_zero(num: &i32) -> bool {
    *num == 0
}

/// A helper for serde to handle null values by falling back to the Default implementation.
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExportTarget {
    #[default]
    Basic, // SBARDEF 1.0.0 (KEX Compatible)
    Extended, // SBARDEF 1.2.0 (Community Ports)
}

bitflags! {
    /// Defines how an element is anchored and offset within its parent container.
    /// Supports widescreen anchoring and offset suppression.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Alignment: u32 {
        const LEFT              = 0x00;
        const H_CENTER          = 0x01;
        const RIGHT             = 0x02;
        const TOP               = 0x00;
        const V_CENTER          = 0x04;
        const BOTTOM            = 0x08;
        /// Ignores the patch's internal X-offset (Doom patch format).
        const NO_LEFT_OFFSET    = 0x10;
        /// Ignores the patch's internal Y-offset (Doom patch format).
        const NO_TOP_OFFSET     = 0x20;
        /// Anchors to the far left edge of a widescreen view.
        const WIDESCREEN_LEFT   = 0x40;
        /// Anchors to the far right edge of a widescreen view.
        const WIDESCREEN_RIGHT  = 0x80;
    }
}

impl Serialize for Alignment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl<'de> Deserialize<'de> for Alignment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Alignment::from_bits_truncate(u32::deserialize(
            deserializer,
        )?))
    }
}

/// Identifiers for specialized engine-driven components like the clock or FPS counter.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    #[default]
    Unknown,
    StatTotals,
    Time,
    Coordinates,
    Speedometer,
    LevelTitle,
    FpsCounter,
    Message,
    AnnounceLevelTitle,
    RenderStats,
    CommandHistory,
    Chat,
}

/// Mapping for numeric values pulled from the player's game state.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum NumberType {
    #[default]
    Health = 0,
    Armor = 1,
    Frags = 2,
    Ammo = 3,
    AmmoSelected = 4,
    MaxAmmo = 5,
    AmmoWeapon = 6,
    MaxAmmoWeapon = 7,
    Kills = 8,
    Items = 9,
    Secrets = 10,
    KillsPercent = 11,
    ItemsPercent = 12,
    SecretsPercent = 13,
    MaxKills = 14,
    MaxItems = 15,
    MaxSecrets = 16,
    PowerupDuration = 17,
}

/// The feature set compatibility for the status bar.
#[derive(
    Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default,
)]
#[repr(i32)]
pub enum FeatureLevel {
    Doom19 = 0,
    LimitRemoving = 1,
    Boom = 2,
    Complevel9 = 3,
    MBF = 4,
    MBF21 = 5,
    #[default]
    ID24 = 6,
}

/// Logic types used in conditions to determine if an element should be rendered.
#[derive(Serialize_repr, Deserialize_repr, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConditionType {
    WeaponOwned = 0,
    WeaponSelected = 1,
    WeaponNotSelected = 2,
    WeaponHasAmmo = 3,
    SelectedWeaponHasAmmo = 4,
    AmmoMatch = 5,
    SlotOwned = 6,
    SlotNotOwned = 7,
    SlotSelected = 8,
    SlotNotSelected = 9,
    ItemOwned = 10,
    ItemNotOwned = 11,
    GameVersionGe = 12,
    GameVersionLt = 13,
    SessionTypeEq = 14,
    SessionTypeNeq = 15,
    GameModeEq = 16,
    GameModeNeq = 17,
    HudModeEq = 18,
    AutomapModeEq = 19,
    WidgetEnabled = 20,
    WidgetDisabled = 21,
    WeaponNotOwned = 22,
    HealthGe = 23,
    HealthLt = 24,
    HealthPercentGe = 25,
    HealthPercentLt = 26,
    ArmorGe = 27,
    ArmorLt = 28,
    ArmorPercentGe = 29,
    ArmorPercentLt = 30,
    SelectedAmmoGe = 31,
    SelectedAmmoLt = 32,
    SelectedAmmoPercentGe = 33,
    SelectedAmmoPercentLt = 34,
    AmmoGe = 35,
    AmmoLt = 36,
    AmmoPercentGe = 37,
    AmmoPercentLt = 38,
    WidescreenModeEq = 39,
    EpisodeEq = 40,
    LevelGe = 41,
    LevelLt = 42,
    PatchEmpty = 43,
    PatchNotEmpty = 44,
    KillsLt = 45,
    KillsGe = 46,
    ItemsLt = 47,
    ItemsGe = 48,
    SecretsLt = 49,
    SecretsGe = 50,
    KillsPercentLt = 51,
    KillsPercentGe = 52,
    ItemsPercentLt = 53,
    ItemsPercentGe = 54,
    SecretsPercentLt = 55,
    SecretsPercentGe = 56,
    PowerupTimeLt = 57,
    PowerupTimeGe = 58,
    PowerupTimePercentLt = 59,
    PowerupTimePercentGe = 60,
}

/// The root structure representing a complete SBARDEF file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SBarDefFile {
    /// The document type (usually "statusbar").
    #[serde(default = "default_type", rename = "type")]
    pub type_: String,
    /// SBARDEF Spec version.
    #[serde(default = "default_version")]
    pub version: String,
    /// The intended target for this project.
    #[serde(skip)]
    pub target: ExportTarget,
    /// The actual definitions for fonts and layouts.
    pub data: StatusBarDefinition,
}

/// Temporary struct to enforce key order during export.
#[derive(Serialize)]
struct KexExport<'a> {
    #[serde(rename = "type")]
    type_: &'a str,
    version: &'a str,
    metadata: Option<serde_json::Value>,
    data: serde_json::Value,
}

/// Container for all font and layout definitions in a project.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatusBarDefinition {
    /// List of fonts intended for numeric stats (Ammo, Health).
    #[serde(default, rename = "numberfonts")]
    pub number_fonts: Vec<NumberFontDef>,
    /// List of fonts intended for alphanumeric HUD text.
    #[serde(default, rename = "hudfonts")]
    pub hud_fonts: Vec<HudFontDef>,
    /// The rendering layouts (can hold multiple versions of the HUD).
    #[serde(default, rename = "statusbars")]
    pub status_bars: Vec<StatusBarLayout>,
}

/// Definition for a number-only font using the 'STT' naming convention.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberFontDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: u8,
    /// The prefix used to find patches (e.g., 'STT' finds 'STTNUM0').
    pub stem: String,
}

/// Definition for a full HUD font using character code suffixes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HudFontDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: u8,
    /// The prefix used to find patches (e.g., 'STCFN' finds 'STCFN033').
    pub stem: String,
}

/// A specific HUD configuration, defining height and visual properties.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusBarLayout {
    /// Optional human-readable name for the layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Height of the status bar area in virtual pixels.
    pub height: i32,
    /// If true, the HUD renders over the world view.
    #[serde(rename = "fullscreenrender", default)]
    pub fullscreen_render: bool,
    /// Name of the flat used to fill the background (e.g., "GRNROCK").
    #[serde(rename = "fillflat")]
    pub fill_flat: Option<String>,
    /// The hierarchy of elements inside this layout.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub children: Vec<ElementWrapper>,
    /// Internal: Mark this bar as a system-managed mandatory slot.
    #[serde(skip)]
    pub _cacoco_system_locked: Option<String>,
}

impl Default for StatusBarLayout {
    fn default() -> Self {
        Self {
            name: None,
            height: 32,
            fullscreen_render: true,
            fill_flat: None,
            children: Vec::new(),
            _cacoco_system_locked: None,
        }
    }
}

impl StatusBarLayout {
    /// Recursively regenerates UIDs for all children. Used during duplication/pasting.
    pub fn reassign_all_uids(&mut self) {
        for child in self.children.iter_mut() {
            child.reassign_uids();
        }
    }

    pub fn normalize(&mut self) {
        for child in &mut self.children {
            child.normalize();
        }
    }
}

/// Generates a new random unique identifier for editor tracking.
pub fn new_uid() -> u64 {
    rand::rng().random()
}

/// Metadata used by Cacoco's "Text String" helper to regenerate children graphics.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TextHelperDef {
    pub text: String,
    pub font: String,
    pub spacing: i32,
}

/// The available types of HUD elements.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Element {
    Canvas(CanvasDef),
    Graphic(GraphicDef),
    Animation(AnimationDef),
    Face(FaceDef),
    #[serde(rename = "facebackground")]
    FaceBackground(FaceDef),
    Number(NumberDef),
    Percent(NumberDef),
    Component(ComponentDef),
    Carousel(CarouselDef),
    List(ListDef),
    String(StringDef),
}

/// A polymorphic wrapper that holds an Element along with editor-specific metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ElementWrapper {
    /// The actual SBARDEF element data.
    #[serde(flatten)]
    pub data: Element,

    /// Internal: If present, this element is treated as a single "Text String".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _cacoco_text: Option<TextHelperDef>,

    /// Internal: User-defined name for organizational purposes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _cacoco_name: Option<String>,

    /// Internal: Runtime-only UID used for UI state and selection tracking.
    #[serde(skip, default = "new_uid")]
    pub uid: u64,
}

impl Default for ElementWrapper {
    fn default() -> Self {
        Self {
            data: Element::Canvas(CanvasDef::default()),
            _cacoco_text: None,
            _cacoco_name: None,
            uid: new_uid(),
        }
    }
}

/// Attributes shared by nearly all HUD elements.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CommonAttrs {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub alignment: Alignment,

    #[serde(default)]
    pub tranmap: Option<String>,
    #[serde(default)]
    pub translation: Option<String>,

    #[serde(default)]
    pub translucency: bool,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub conditions: Vec<ConditionDef>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub children: Vec<ElementWrapper>,
}

impl CommonAttrs {
    /// Utility to generate the required visibility check for ammo-selected elements.
    pub fn selected_ammo_check() -> Self {
        Self {
            conditions: vec![ConditionDef {
                condition: ConditionType::SelectedWeaponHasAmmo,
                param: 0,
                param2: 0,
                param_string: None,
            }],
            ..Default::default()
        }
    }
}

/// A logic rule used to toggle element visibility.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionDef {
    pub condition: ConditionType,
    #[serde(default)]
    pub param: i32,
    #[serde(default)]
    pub param2: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub param_string: Option<String>,
}

/// Defines a rectangular area of a patch to render.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CropDef {
    pub width: i32,
    pub height: i32,
    pub left: i32,
    pub top: i32,
    /// If true, offsets the crop from the center of the patch.
    #[serde(default)]
    pub center: bool,
}

/// A logical grouping element used to offset children.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CanvasDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
}

/// A container that automatically stacks its children horizontally or vertically.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ListDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    #[serde(default)]
    pub horizontal: bool,
    /// Spacing between children in virtual pixels.
    #[serde(default)]
    pub spacing: i32,
}

/// Renders a static image from a WAD patch.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GraphicDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    /// The lump name of the patch to draw.
    pub patch: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub width: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub height: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub topoffset: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub leftoffset: i32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub midoffset: i32,
    /// Optional cropping parameters.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<CropDef>,
}

/// Renders a timed sequence of patches.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnimationDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    /// The sequence of frames.
    pub frames: Vec<FrameDef>,
    /// Target frame rate (standard is 35).
    #[serde(default = "default_fps")]
    pub framerate: f64,
}

fn default_fps() -> f64 {
    10.0
}
fn default_duration() -> f32 {
    4.0
}

/// A single frame within an animation sequence.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FrameDef {
    pub lump: String,
    /// Duration of this frame in seconds.
    pub duration: f64,
}

/// Renders the Doom status bar face (Doomguy).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FaceDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    #[serde(default)]
    pub width: i32,
    #[serde(default)]
    pub height: i32,
    #[serde(default)]
    pub topoffset: i32,
    #[serde(default)]
    pub leftoffset: i32,
    #[serde(default)]
    pub midoffset: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<CropDef>,
}

fn default_maxlength() -> i32 {
    3
}

/// Renders a numeric player statistic.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    /// The name of a registered Number Font.
    pub font: String,
    /// The stat type to display.
    #[serde(rename = "type")]
    pub type_: NumberType,
    /// ID used for ammo types or powerup indices.
    #[serde(default)]
    pub param: i32,
    /// Max digits to display before capping.
    #[serde(default = "default_maxlength")]
    pub maxlength: i32,
}

impl Default for NumberDef {
    fn default() -> Self {
        Self {
            common: CommonAttrs::default(),
            font: String::new(),
            type_: NumberType::Health,
            param: 0,
            maxlength: 3,
        }
    }
}

/// Renders a dynamic alphanumeric string (like Map Titles).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StringDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    #[serde(rename = "type")]
    pub type_: u8,
    /// Hardcoded data for custom strings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// The name of a registered HUD Font.
    pub font: String,
}

/// Renders complex engine-driven components.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ComponentDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    pub font: String,
    #[serde(default)]
    pub vertical: bool,
    #[serde(default = "default_duration")]
    pub duration: f32,
}

impl Default for ComponentDef {
    fn default() -> Self {
        Self {
            common: CommonAttrs::default(),
            type_: ComponentType::Unknown,
            font: String::new(),
            vertical: false,
            duration: 4.0,
        }
    }
}

/// Represents the KEX/Woof style weapon selection carousel.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CarouselDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
}

impl ElementWrapper {
    /// Returns true if the SBARDEF spec allows this element to have children.
    /// In Cacoco, we allow "Child Mode" (Ctrl/Cmd) to target any container-type element.
    pub fn is_spec_container(&self) -> bool {
        matches!(
            self.data,
            Element::Canvas(_) | Element::List(_) | Element::Carousel(_)
        )
    }

    /// Returns true if this is a logical organizational folder (Canvas, List, etc.).
    /// Natural containers allow nesting by default.
    pub fn is_natural_container(&self) -> bool {
        self._cacoco_text.is_none() && self.is_spec_container()
    }

    /// Returns a human-friendly name for use in the layer tree.
    pub fn display_name(&self) -> String {
        if let Some(t) = &self._cacoco_text {
            return format!("\"{}\"", t.text);
        }
        if let Some(n) = &self._cacoco_name {
            return n.clone();
        }

        match &self.data {
            Element::Canvas(_) => "Canvas Group".to_string(),
            Element::List(_) => "List Container".to_string(),
            Element::Graphic(g) => format!("Graphic: {}", g.patch),
            Element::Animation(_) => "Animation".to_string(),
            Element::Face(_) => "Doomguy".to_string(),
            Element::FaceBackground(_) => "Face Background".to_string(),
            Element::Number(n) => format!("Number ({:?})", n.type_),
            Element::Percent(p) => format!("Percent ({:?})", p.type_),
            Element::String(s) => {
                if s.type_ == 0 {
                    if let Some(data) = &s.data {
                        if !data.is_empty() {
                            return format!("\"{}\"", data);
                        }
                    }
                }
                format!("String (Type {})", s.type_)
            }
            Element::Component(c) => format!("Component: {:?}", c.type_),
            Element::Carousel(_) => "Carousel".to_string(),
        }
    }

    /// Accessor for children regardless of the underlying element type.
    pub fn children(&self) -> &[ElementWrapper] {
        match &self.data {
            Element::Canvas(e) => &e.common.children,
            Element::List(e) => &e.common.children,
            Element::Graphic(e) => &e.common.children,
            Element::Animation(e) => &e.common.children,
            Element::Face(e) => &e.common.children,
            Element::FaceBackground(e) => &e.common.children,
            Element::Number(e) => &e.common.children,
            Element::Percent(e) => &e.common.children,
            Element::String(e) => &e.common.children,
            Element::Component(e) => &e.common.children,
            Element::Carousel(e) => &e.common.children,
        }
    }

    /// Accessor for common attributes.
    pub fn get_common_mut(&mut self) -> &mut CommonAttrs {
        match &mut self.data {
            Element::Canvas(e) => &mut e.common,
            Element::List(e) => &mut e.common,
            Element::Graphic(e) => &mut e.common,
            Element::Animation(e) => &mut e.common,
            Element::Face(e) => &mut e.common,
            Element::FaceBackground(e) => &mut e.common,
            Element::Number(e) => &mut e.common,
            Element::Percent(e) => &mut e.common,
            Element::String(e) => &mut e.common,
            Element::Component(e) => &mut e.common,
            Element::Carousel(e) => &mut e.common,
        }
    }

    /// Immutable accessor for common attributes.
    pub fn get_common(&self) -> &CommonAttrs {
        match &self.data {
            Element::Canvas(e) => &e.common,
            Element::List(e) => &e.common,
            Element::Graphic(e) => &e.common,
            Element::Animation(e) => &e.common,
            Element::Face(e) => &e.common,
            Element::FaceBackground(e) => &e.common,
            Element::Number(e) => &e.common,
            Element::Percent(e) => &e.common,
            Element::String(e) => &e.common,
            Element::Component(e) => &e.common,
            Element::Carousel(e) => &e.common,
        }
    }

    /// Recursively regenerates UIDs for this element and all its children.
    pub fn reassign_uids(&mut self) {
        self.uid = new_uid();
        for child in self.get_common_mut().children.iter_mut() {
            child.reassign_uids();
        }
    }

    pub fn normalize(&mut self) {
        match &mut self.data {
            Element::Graphic(g) => {
                g.patch = crate::assets::AssetStore::stem(&g.patch);
            }
            Element::Animation(a) => {
                for frame in &mut a.frames {
                    frame.lump = crate::assets::AssetStore::stem(&frame.lump);
                }
            }
            _ => {}
        }

        for child in &mut self.get_common_mut().children {
            child.normalize();
        }
    }
}

impl SBarDefFile {
    /// Creates a new, empty SBARDEF project targeting Extended features by default.
    pub fn new_empty() -> Self {
        Self {
            type_: "statusbar".to_string(),
            version: "1.2.0".to_string(),
            target: ExportTarget::Extended,
            data: StatusBarDefinition {
                status_bars: vec![StatusBarLayout::default()],
                ..Default::default()
            },
        }
    }

    /// Inspects the content of the file to determine if it uses features beyond the Basic (1.0.0) spec.
    pub fn determine_target(&self) -> ExportTarget {
        if self.version != "1.0.0" {
            return ExportTarget::Extended;
        }
        if !self.data.hud_fonts.is_empty() {
            return ExportTarget::Extended;
        }
        for bar in &self.data.status_bars {
            if Self::is_extended_feature_detected(&bar.children) {
                return ExportTarget::Extended;
            }
        }
        ExportTarget::Basic
    }

    fn is_extended_feature_detected(elements: &[ElementWrapper]) -> bool {
        for el in elements {
            if matches!(
                el.data,
                Element::List(_)
                    | Element::String(_)
                    | Element::Component(_)
                    | Element::Carousel(_)
            ) {
                return true;
            }
            let common = el.get_common();
            if common.translucency {
                return true;
            }
            match &el.data {
                Element::Number(n) | Element::Percent(n) => {
                    if (n.type_ as u8) > 7 {
                        return true;
                    }
                }
                _ => {}
            }
            for cond in &common.conditions {
                if (cond.condition as u8) > 18 {
                    return true;
                }
            }
            match &el.data {
                Element::Graphic(g) if g.crop.is_some() => return true,
                Element::Face(f) if f.crop.is_some() => return true,
                Element::FaceBackground(f) if f.crop.is_some() => return true,
                _ => {}
            }
            if Self::is_extended_feature_detected(&common.children) {
                return true;
            }
        }
        false
    }

    /// Aggressively enforces the KEX status bar "sandwich" for the Basic target.
    pub fn normalize_for_target(&mut self) {
        if self.target != ExportTarget::Basic {
            for bar in &mut self.data.status_bars {
                bar._cacoco_system_locked = None;
            }
            return;
        }

        let mut standards = Vec::new();
        let mut fullscreens = Vec::new();

        let old_bars = self.data.status_bars.drain(..).collect::<Vec<_>>();

        for (i, bar) in old_bars.into_iter().enumerate() {
            if i == 0 && !bar.fullscreen_render {
                standards.push(bar);
                continue;
            }

            if bar._cacoco_system_locked.is_some() && bar.children.is_empty() {
                continue;
            }

            if bar.fullscreen_render {
                fullscreens.push(bar);
            } else {
                standards.push(bar);
            }
        }

        let mut new_bars = Vec::new();

        let mut slot_0 = if !standards.is_empty() {
            standards.remove(0)
        } else {
            StatusBarLayout {
                height: 32,
                fullscreen_render: false,
                ..Default::default()
            }
        };
        slot_0.fullscreen_render = false;
        slot_0._cacoco_system_locked = Some("KEX Mandatory Non-Fullscreen".to_string());
        new_bars.push(slot_0);

        if fullscreens.is_empty() {
            new_bars.push(StatusBarLayout {
                height: 200,
                fullscreen_render: true,
                ..Default::default()
            });
        } else {
            for mut f_bar in fullscreens {
                f_bar.fullscreen_render = true;
                f_bar.height = 200;
                f_bar._cacoco_system_locked = None;
                new_bars.push(f_bar);
            }
        }

        new_bars.push(StatusBarLayout {
            height: 200,
            fullscreen_render: true,
            children: Vec::new(),
            _cacoco_system_locked: Some("KEX Demo Blank Fullscreen".to_string()),
            ..Default::default()
        });

        self.data.status_bars = new_bars;
    }

    pub fn to_sanitized_json(&self, assets: &crate::assets::AssetStore) -> String {
        let mut clone = self.clone();
        clone.normalize_for_target();

        if clone.target == ExportTarget::Basic {
            clone.version = "1.0.0".to_string();
            clone.data.hud_fonts.clear();
            if clone.data.number_fonts.is_empty() {
                clone.data.number_fonts.push(NumberFontDef {
                    name: "BigRed".to_string(),
                    type_: 0,
                    stem: "STT".to_string(),
                });
            }
            let fonts = crate::ui::properties::font_cache::FontCache::new(&clone);
            for bar in &mut clone.data.status_bars {
                bar.name = None;
                if bar.fullscreen_render {
                    bar.height = 200;
                }
                Self::scrub_elements(&mut bar.children, assets, &fonts, ExportTarget::Basic);
            }
        } else {
            clone.version = "1.2.0".to_string();
        }

        if let Ok(mut data_val) = serde_json::to_value(&clone.data) {
            Self::prune_json_value(&mut data_val, clone.target);
            let export = KexExport {
                type_: &clone.type_,
                version: &clone.version,
                metadata: None,
                data: data_val,
            };
            return serde_json::to_string_pretty(&export).unwrap_or_default();
        }
        String::new()
    }

    fn prune_json_value(v: &mut serde_json::Value, target: ExportTarget) {
        if let Some(obj) = v.as_object_mut() {
            obj.retain(|k, _| !k.starts_with("_cacoco_"));
            if target == ExportTarget::Basic {
                obj.remove("hudfonts");
                obj.remove("translucency");
                obj.remove("crop");
                let keys: Vec<String> = obj.keys().cloned().collect();
                for k in keys {
                    let is_empty_array = obj
                        .get(&k)
                        .and_then(|v| v.as_array())
                        .map_or(false, |a| a.is_empty());
                    if is_empty_array {
                        obj.insert(k, serde_json::Value::Null);
                    }
                }
            } else {
                obj.retain(|_, val| !val.is_null());
            }
            for value in obj.values_mut() {
                Self::prune_json_value(value, target);
            }
        } else if let Some(arr) = v.as_array_mut() {
            for value in arr {
                Self::prune_json_value(value, target);
            }
        }
    }

    fn scrub_elements(
        elements: &mut Vec<ElementWrapper>,
        assets: &crate::assets::AssetStore,
        fonts: &crate::ui::properties::font_cache::FontCache,
        target: ExportTarget,
    ) {
        for el in elements.iter_mut() {
            if target == ExportTarget::Basic && el._cacoco_text.is_some() {
                crate::ui::properties::text_helper::rebake_text(el, assets, fonts);
            }
            el._cacoco_name = None;
            el._cacoco_text = None;
            let common = el.get_common_mut();
            if target == ExportTarget::Basic {
                common.translucency = false;
            }
            Self::scrub_elements(&mut common.children, assets, fonts, target);
        }
        if target == ExportTarget::Basic {
            elements.retain(|el| {
                !matches!(
                    el.data,
                    Element::List(_)
                        | Element::String(_)
                        | Element::Component(_)
                        | Element::Carousel(_)
                )
            });
        }
    }

    pub fn get_element_mut(&mut self, path: &[usize]) -> Option<&mut ElementWrapper> {
        if path.is_empty() {
            return None;
        }
        let bar_idx = path[0];
        if bar_idx >= self.data.status_bars.len() {
            return None;
        }
        let bar = &mut self.data.status_bars[bar_idx];
        if path.len() == 1 {
            return None;
        }
        let mut cur = bar.children.get_mut(path[1])?;
        for &idx in &path[2..] {
            cur = cur.get_common_mut().children.get_mut(idx)?;
        }
        Some(cur)
    }

    pub fn get_element(&self, path: &[usize]) -> Option<&ElementWrapper> {
        if path.is_empty() {
            return None;
        }
        let bar_idx = path[0];
        let bar = self.data.status_bars.get(bar_idx)?;
        if path.len() == 1 {
            return None;
        }
        let mut cur = bar.children.get(path[1])?;
        for &idx in &path[2..] {
            cur = cur.get_common().children.get(idx)?;
        }
        Some(cur)
    }

    pub fn normalize_paths(&mut self) {
        for bar in &mut self.data.status_bars {
            bar.normalize();
        }
    }
}

/// Helper to wrap a patch name into a Graphic element with the specified coordinates.
pub fn wrap_graphic(patch: &str, x: i32, y: i32) -> ElementWrapper {
    ElementWrapper {
        data: Element::Graphic(GraphicDef {
            common: CommonAttrs {
                x,
                y,
                ..Default::default()
            },
            patch: crate::assets::AssetStore::stem(patch),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_graphic_preserves_position() {
        let expected_x = 123;
        let expected_y = 456;
        let el = wrap_graphic("STBAR", expected_x, expected_y);

        let common = el.get_common();
        assert_eq!(common.x, expected_x, "Helper failed to set X coordinate!");
        assert_eq!(common.y, expected_y, "Helper failed to set Y coordinate!");
    }
}
