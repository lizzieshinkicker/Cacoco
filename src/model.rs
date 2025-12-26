use crate::assets::AssetStore;
use crate::state::PreviewState;
use crate::ui::properties::editor::PropertiesUI;
use crate::ui::properties::font_cache::FontCache;
use crate::ui::properties::preview::PreviewContent;
use bitflags::bitflags;
use eframe::egui;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_repr::{Deserialize_repr, Serialize_repr};

fn default_type() -> String {
    "statusbar".to_string()
}
fn default_version() -> String {
    "1.0.0".to_string()
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Alignment: u32 {
        const LEFT          = 0x00;
        const H_CENTER      = 0x01;
        const RIGHT         = 0x02;
        const TOP           = 0x00;
        const V_CENTER      = 0x04;
        const BOTTOM        = 0x08;
        const DYNAMIC_LEFT  = 0x10;
        const DYNAMIC_RIGHT = 0x20;
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
}

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SBarDefFile {
    #[serde(default = "default_type", rename = "type")]
    pub type_: String,

    #[serde(default = "default_version")]
    pub version: String,

    pub data: StatusBarDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StatusBarDefinition {
    #[serde(default, rename = "numberfonts")]
    pub number_fonts: Vec<NumberFontDef>,
    #[serde(default, rename = "hudfonts")]
    pub hud_fonts: Vec<HudFontDef>,
    #[serde(default, rename = "statusbars")]
    pub status_bars: Vec<StatusBarLayout>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberFontDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: u8,
    pub stem: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HudFontDef {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: u8,
    pub stem: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusBarLayout {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub height: i32,
    #[serde(rename = "fullscreenrender", default)]
    pub fullscreen_render: bool,
    #[serde(rename = "fillflat")]
    pub fill_flat: Option<String>,
    #[serde(default)]
    pub children: Vec<ElementWrapper>,
}

impl Default for StatusBarLayout {
    fn default() -> Self {
        Self {
            name: None,
            height: 32,
            fullscreen_render: true,
            fill_flat: None,
            children: Vec::new(),
        }
    }
}

impl StatusBarLayout {
    /// Recursively reassigns all UIDs for every element within this layout.
    pub fn reassign_all_uids(&mut self) {
        for child in self.children.iter_mut() {
            child.reassign_uids();
        }
    }
}

pub fn new_uid() -> u64 {
    rand::rng().random()
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TextHelperDef {
    pub text: String,
    pub font: String,
    pub spacing: i32,
}

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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ElementWrapper {
    #[serde(flatten)]
    pub data: Element,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _cacoco_text: Option<TextHelperDef>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub _cacoco_name: Option<String>,

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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CommonAttrs {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default)]
    pub alignment: Alignment,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tranmap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
    #[serde(default)]
    pub translucency: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<ConditionDef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ElementWrapper>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionDef {
    pub condition: ConditionType,
    #[serde(default)]
    pub param: i32,
    #[serde(default)]
    pub param2: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CanvasDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GraphicDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    pub patch: String,
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
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AnimationDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    pub frames: Vec<FrameDef>,
    #[serde(default = "default_fps")]
    pub framerate: f64,
}

fn default_fps() -> f64 {
    10.0
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct FrameDef {
    pub lump: String,
    pub duration: f64,
}

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
}

fn default_maxlength() -> i32 {
    3
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    pub font: String,
    #[serde(rename = "type")]
    pub type_: NumberType,
    #[serde(default)]
    pub param: i32,
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

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ComponentDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
    #[serde(rename = "type")]
    pub type_: ComponentType,
    pub font: String,
    #[serde(default)]
    pub vertical: bool,
    #[serde(default)]
    pub duration: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CarouselDef {
    #[serde(flatten)]
    pub common: CommonAttrs,
}

impl ElementWrapper {
    pub fn display_name(&self) -> String {
        if let Some(t) = &self._cacoco_text {
            return format!("\"{}\"", t.text);
        }
        if let Some(n) = &self._cacoco_name {
            return n.clone();
        }

        match &self.data {
            Element::Canvas(_) => "Canvas Group".to_string(),
            Element::Graphic(g) => format!("Graphic: {}", g.patch),
            Element::Animation(_) => "Animation".to_string(),
            Element::Face(_) => "Doomguy".to_string(),
            Element::FaceBackground(_) => "Face Background".to_string(),
            Element::Number(n) => format!("Number ({:?})", n.type_),
            Element::Percent(p) => format!("Percent ({:?})", p.type_),
            Element::Component(c) => format!("Component: {:?}", c.type_),
            Element::Carousel(_) => "Carousel".to_string(),
        }
    }

    pub fn children(&self) -> &[ElementWrapper] {
        match &self.data {
            Element::Canvas(e) => &e.common.children,
            Element::Graphic(e) => &e.common.children,
            Element::Animation(e) => &e.common.children,
            Element::Face(e) => &e.common.children,
            Element::FaceBackground(e) => &e.common.children,
            Element::Number(e) => &e.common.children,
            Element::Percent(e) => &e.common.children,
            Element::Component(e) => &e.common.children,
            Element::Carousel(e) => &e.common.children,
        }
    }

    pub fn get_common_mut(&mut self) -> &mut CommonAttrs {
        match &mut self.data {
            Element::Canvas(e) => &mut e.common,
            Element::Graphic(e) => &mut e.common,
            Element::Animation(e) => &mut e.common,
            Element::Face(e) => &mut e.common,
            Element::FaceBackground(e) => &mut e.common,
            Element::Number(e) => &mut e.common,
            Element::Percent(e) => &mut e.common,
            Element::Component(e) => &mut e.common,
            Element::Carousel(e) => &mut e.common,
        }
    }

    pub fn get_common(&self) -> &CommonAttrs {
        match &self.data {
            Element::Canvas(e) => &e.common,
            Element::Graphic(e) => &e.common,
            Element::Animation(e) => &e.common,
            Element::Face(e) => &e.common,
            Element::FaceBackground(e) => &e.common,
            Element::Number(e) => &e.common,
            Element::Percent(e) => &e.common,
            Element::Component(e) => &e.common,
            Element::Carousel(e) => &e.common,
        }
    }

    /// Recursively reassigns UIDs for this element and all its children.
    pub fn reassign_uids(&mut self) {
        self.uid = new_uid();
        for child in self.get_common_mut().children.iter_mut() {
            child.reassign_uids();
        }
    }
}

impl SBarDefFile {
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

        let mut current_element = bar.children.get_mut(path[1])?;

        for &child_idx in &path[2..] {
            current_element = current_element
                .get_common_mut()
                .children
                .get_mut(child_idx)?;
        }

        Some(current_element)
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

        let mut current_element = bar.children.get(path[1])?;
        for &child_idx in &path[2..] {
            current_element = current_element.get_common().children.get(child_idx)?;
        }

        Some(current_element)
    }
}

impl PropertiesUI for ElementWrapper {
    fn draw_specific_fields(
        &mut self,
        ui: &mut egui::Ui,
        fonts: &FontCache,
        assets: &AssetStore,
        state: &PreviewState,
    ) -> bool {
        match &mut self.data {
            Element::Canvas(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Graphic(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Animation(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Face(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::FaceBackground(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Number(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Percent(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Component(e) => e.draw_specific_fields(ui, fonts, assets, state),
            Element::Carousel(e) => e.draw_specific_fields(ui, fonts, assets, state),
        }
    }

    fn get_preview_content(
        &self,
        ui: &egui::Ui,
        fonts: &FontCache,
        state: &PreviewState,
    ) -> Option<PreviewContent> {
        match &self.data {
            Element::Canvas(e) => e.get_preview_content(ui, fonts, state),
            Element::Graphic(e) => e.get_preview_content(ui, fonts, state),
            Element::Animation(e) => e.get_preview_content(ui, fonts, state),
            Element::Face(e) => e.get_preview_content(ui, fonts, state),
            Element::FaceBackground(_) => Some(PreviewContent::Image("STFB0".to_string())),
            Element::Number(e) => e.get_preview_content(ui, fonts, state),
            Element::Percent(e) => {
                let mut content = e.get_preview_content(ui, fonts, state)?;
                if let PreviewContent::Text { text, .. } = &mut content {
                    *text = format!("{}%", text);
                }
                Some(content)
            }
            Element::Component(e) => e.get_preview_content(ui, fonts, state),
            Element::Carousel(e) => e.get_preview_content(ui, fonts, state),
        }
    }

    fn has_specific_fields(&self) -> bool {
        match &self.data {
            Element::Canvas(e) => e.has_specific_fields(),
            Element::Graphic(e) => e.has_specific_fields(),
            Element::Animation(e) => e.has_specific_fields(),
            Element::Face(e) => e.has_specific_fields(),
            Element::FaceBackground(e) => e.has_specific_fields(),
            Element::Number(e) => e.has_specific_fields(),
            Element::Percent(e) => e.has_specific_fields(),
            Element::Component(e) => e.has_specific_fields(),
            Element::Carousel(e) => e.has_specific_fields(),
        }
    }
}
