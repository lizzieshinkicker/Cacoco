use crate::app::ConfirmationRequest;
use crate::assets::AssetStore;
use crate::library::{self, FontDefinition, FontSource};
use crate::model::{HudFontDef, NumberFontDef, SBarDefFile};
use crate::ui::context_menu::ContextMenu;
use crate::ui::font_wizard::FontWizardState;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;
use super::thumbnails::{self, ListRow};

const ASSET_SEL_KEY: &str = "cacoco_asset_selection";
const ASSET_PIVOT_KEY: &str = "cacoco_asset_pivot";

struct LibraryGroup {
    title: &'static str,
    prefixes: &'static [&'static str],
    default_open: bool,
}

const LIB_GROUPS: &[LibraryGroup] = &[
    LibraryGroup {
        title: "Status Bar UI",
        prefixes: &["ammo_ov", "arm_", "bmty", "boom", "dsda", "prbm"],
        default_open: true,
    },
    LibraryGroup {
        title: "Progress Bars",
        prefixes: &["progbar"],
        default_open: true,
    },
    LibraryGroup {
        title: "Tags & Labels",
        prefixes: &["tag_"],
        default_open: true,
    },
];

pub fn draw_fonts_content(ui: &mut egui::Ui, file: &mut SBarDefFile, assets: &AssetStore) {
    let mut remove_num = None;
    let mut remove_hud = None;

    if !file.data.hud_fonts.is_empty() {
        ui.label(egui::RichText::new("HUD Fonts").strong());
        for (i, font) in file.data.hud_fonts.iter().enumerate() {
            let preview_char = (b'A' + (i % 26) as u8) as char;
            if draw_registered_font_row(ui, &font.name, &font.stem, false, preview_char, assets) {
                remove_hud = Some(i);
            }
        }
    }

    if !file.data.number_fonts.is_empty() && !file.data.hud_fonts.is_empty() {
        ui.add_space(8.0);
    }

    if !file.data.number_fonts.is_empty() {
        ui.label(egui::RichText::new("Number Fonts").strong());
        for (i, font) in file.data.number_fonts.iter().enumerate() {
            let preview_char = std::char::from_digit((i % 10) as u32, 10).unwrap_or('0');
            if draw_registered_font_row(ui, &font.name, &font.stem, true, preview_char, assets) {
                remove_num = Some(i);
            }
        }
    }

    if let Some(i) = remove_num {
        file.data.number_fonts.remove(i);
    }
    if let Some(i) = remove_hud {
        file.data.hud_fonts.remove(i);
    }
}

fn draw_registered_font_row(
    ui: &mut egui::Ui,
    name: &str,
    stem: &str,
    is_num: bool,
    prev_char: char,
    assets: &AssetStore,
) -> bool {
    let patch = assets.resolve_patch_name(stem, prev_char, is_num);
    let texture = assets.textures.get(&patch);

    let response = ListRow::new(name)
        .subtitle(format!("({})", stem))
        .texture(texture)
        .fallback("?")
        .show(ui);

    let mut remove_clicked = false;
    let just_opened = ContextMenu::check(ui, &response);
    if let Some(menu) = ContextMenu::get(ui, response.id) {
        ContextMenu::show(ui, menu, just_opened, |ui| {
            if ContextMenu::button(ui, "Remove Font", true) {
                remove_clicked = true;
                ContextMenu::close(ui);
            }
        });
    }
    remove_clicked
}

pub fn draw_filtered_browser(
    ui: &mut egui::Ui,
    assets: &mut AssetStore,
    file: &mut Option<SBarDefFile>,
    zoom: f32,
    show_project_assets: bool,
    wizard_state: &mut Option<FontWizardState>,
    confirmation_modal: &mut Option<ConfirmationRequest>,
    show_fonts_toggle: bool,
) {
    if !show_project_assets {
        ui.heading("Base IWAD Browser");
        ui.separator();
        ui.add_space(4.0);

        ui.label(egui::RichText::new("IWAD HUD Fonts").strong());
        ui.add_space(4.0);
        let iwad_hud = library::FONTS
            .iter()
            .filter(|f| f.source == FontSource::Internal && f.is_hud);
        for (i, font) in iwad_hud.enumerate() {
            draw_unified_font_row(ui, assets, file, font, i);
        }

        ui.add_space(8.0);
        ui.label(egui::RichText::new("IWAD Number Fonts").strong());
        ui.add_space(4.0);
        let iwad_num = library::FONTS
            .iter()
            .filter(|f| f.source == FontSource::Internal && !f.is_hud);
        for (i, font) in iwad_num.enumerate() {
            draw_unified_font_row(ui, assets, file, font, i);
        }

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
        ui.label(egui::RichText::new("Loose IWAD Assets").strong());
    }

    let mut project_stems = HashSet::new();
    for path in assets.raw_files.keys() {
        let stem = AssetStore::stem(path);
        project_stems.insert(stem);
    }

    let mut library_stems = HashSet::new();
    for asset in library::ASSETS {
        library_stems.insert(AssetStore::stem(asset.name));
    }

    let mut registered_font_stems = HashSet::new();
    if let Some(f) = file {
        for fd in &f.data.number_fonts {
            registered_font_stems.insert(fd.stem.to_uppercase());
        }
        for fd in &f.data.hud_fonts {
            registered_font_stems.insert(fd.stem.to_uppercase());
        }
    }

    let mut keys: Vec<&String> = assets
        .textures
        .keys()
        .filter(|k| !k.starts_with('_'))
        .filter(|key| {
            if !show_fonts_toggle {
                if registered_font_stems.iter().any(|stem| key.starts_with(stem)) {
                    return false;
                }
                if library::FONTS.iter().any(|f| key.starts_with(&f.stem.to_uppercase())) {
                    return false;
                }
            }

            if !show_project_assets {
                if library_stems.contains(*key) {
                    return false;
                }
            }

            let is_project = project_stems.contains(*key);
            show_project_assets == is_project
        })
        .collect();

    keys.sort();

    if keys.is_empty() && show_project_assets {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.label(egui::RichText::new("No assets found.").weak());
        });
        return;
    }
    draw_asset_grid(
        ui,
        assets,
        &keys,
        zoom,
        wizard_state,
        confirmation_modal,
        show_project_assets,
    );
}

pub fn draw_library_browser(
    ui: &mut egui::Ui,
    assets: &mut AssetStore,
    file: &mut Option<SBarDefFile>,
    zoom: f32,
) {
    ui.heading("Library Browser");
    ui.separator();
    ui.add_space(4.0);

    ui.label(egui::RichText::new("HUD Font Packages").strong());
    ui.add_space(4.0);
    let lib_hud = library::FONTS
        .iter()
        .filter(|f| f.source == FontSource::Package && f.is_hud);
    for (i, font) in lib_hud.enumerate() {
        draw_unified_font_row(ui, assets, file, font, i);
    }

    ui.add_space(8.0);
    ui.label(egui::RichText::new("Number Font Packages").strong());
    ui.add_space(4.0);
    let lib_num = library::FONTS
        .iter()
        .filter(|f| f.source == FontSource::Package && !f.is_hud);
    for (i, font) in lib_num.enumerate() {
        draw_unified_font_row(ui, assets, file, font, i);
    }

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Loose Library Assets").strong());

    let available_w = ui.available_width() - 12.0;
    let target_size = thumbnails::THUMB_SIZE * zoom;
    let cols = ((available_w + 4.0) / (target_size + 4.0)).floor().max(1.0);
    let size = (available_w - ((cols - 1.0) * 4.0)) / cols;

    for group in LIB_GROUPS {
        egui::CollapsingHeader::new(group.title)
            .default_open(group.default_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                    for lib_asset in library::ASSETS {
                        let is_match = group.prefixes.iter().any(|p| lib_asset.name.starts_with(p));
                        let is_not_font =
                            !matches!(lib_asset.category, library::AssetCategory::Font);
                        if is_match && is_not_font {
                            draw_library_item(ui, assets, lib_asset, size);
                        }
                    }
                });
            });
    }
}

fn draw_unified_font_row(
    ui: &mut egui::Ui,
    assets: &mut AssetStore,
    file: &mut Option<SBarDefFile>,
    font: &FontDefinition,
    index: usize,
) {
    let is_installed = file.as_ref().map_or(false, |f| {
        if font.is_hud {
            f.data.hud_fonts.iter().any(|def| def.name == font.name)
        } else {
            f.data.number_fonts.iter().any(|def| def.name == font.name)
        }
    });

    let preview_char = if !font.is_hud {
        std::char::from_digit((index % 10) as u32, 10).unwrap_or('0')
    } else {
        (b'A' + (index % 26) as u8) as char
    };

    let stem_upper = font.stem.to_uppercase();
    let preview_patch = assets.resolve_patch_name(&stem_upper, preview_char, !font.is_hud);
    let texture = assets.textures.get(&preview_patch);

    let response = ListRow::new(font.name)
        .subtitle(font.description)
        .texture(texture)
        .fallback("Aa")
        .active(is_installed)
        .show(ui);

    let btn_rect = egui::Rect::from_center_size(
        egui::pos2(response.rect.right() - 43.0, response.rect.center().y),
        egui::vec2(70.0, 24.0),
    );

    if is_installed {
        ui.put(
            btn_rect,
            egui::Label::new(
                egui::RichText::new("Added!")
                    .color(egui::Color32::GREEN)
                    .strong(),
            ),
        );
    } else if ui.put(btn_rect, egui::Button::new("Add").small()).clicked() {
        if let Some(f) = file {
            if font.source == FontSource::Package {
                for asset in library::ASSETS {
                    let asset_stem = AssetStore::stem(asset.name);
                    if asset_stem.starts_with(&stem_upper) {
                        assets.load_image(ui.ctx(), &asset_stem, asset.bytes);
                    }
                }
            }
            if font.is_hud {
                f.data.hud_fonts.push(HudFontDef {
                    name: font.name.to_string(),
                    type_: 0,
                    stem: stem_upper,
                });
            } else {
                f.data.number_fonts.push(NumberFontDef {
                    name: font.name.to_string(),
                    type_: 0,
                    stem: stem_upper,
                });
            }
        }
    }
    ui.add_space(8.0);
}

fn draw_asset_grid(
    ui: &mut egui::Ui,
    assets: &AssetStore,
    keys: &[&String],
    zoom: f32,
    wizard_state: &mut Option<FontWizardState>,
    confirmation_modal: &mut Option<ConfirmationRequest>,
    is_project_tab: bool,
) {
    let available_w = ui.available_width() - 12.0;
    let target_size = thumbnails::THUMB_SIZE * zoom;

    let mut selection: HashSet<String> = ui.data(|d| {
        d.get_temp(egui::Id::new(ASSET_SEL_KEY))
            .unwrap_or_default()
    });
    let mut pivot: Option<String> =
        ui.data(|d| d.get_temp(egui::Id::new(ASSET_PIVOT_KEY)).unwrap_or_default());

    let cols = ((available_w + 4.0) / (target_size + 4.0)).floor().max(1.0);
    let size = (available_w - ((cols - 1.0) * 4.0)) / cols;

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
        for (idx, key) in keys.iter().enumerate() {
            let texture = assets.textures.get(*key);
            let is_selected = selection.contains(*key);
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click_and_drag());

            if response.clicked() {
                let modifiers = ui.input(|i| i.modifiers);
                if modifiers.ctrl || modifiers.command {
                    if is_selected {
                        selection.remove(*key);
                    } else {
                        selection.insert(key.to_string());
                        pivot = Some(key.to_string());
                    }
                } else if modifiers.shift {
                    if let Some(p_key) = &pivot {
                        let start = keys.iter().position(|k| *k == p_key);
                        if let (Some(s), Some(e)) = (start, Some(idx)) {
                            selection.clear();
                            for i in s.min(e)..=s.max(e) {
                                selection.insert(keys[i].to_string());
                            }
                        }
                    } else {
                        selection.insert(key.to_string());
                        pivot = Some(key.to_string());
                    }
                } else {
                    selection.clear();
                    selection.insert(key.to_string());
                    pivot = Some(key.to_string());
                }
            }

            if is_selected {
                ui.painter()
                    .rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    ui.visuals().selection.stroke,
                    egui::StrokeKind::Inside,
                );
            } else {
                thumbnails::draw_thumb_bg(ui, rect);
            }

            if let Some(tex) = texture {
                let scaled_size = tex.size_vec2() * (size - 4.0)
                    / tex.size_vec2().x.max(tex.size_vec2().y).max(1.0);
                let tint = if is_selected {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::from_gray(200)
                };
                ui.painter().image(
                    tex.id(),
                    egui::Rect::from_center_size(rect.center(), scaled_size),
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    tint,
                );
            }

            let just_opened = ContextMenu::check(ui, &response);
            if let Some(menu) = ContextMenu::get(ui, response.id) {
                if !selection.contains(*key) {
                    selection.clear();
                    selection.insert(key.to_string());
                    pivot = Some(key.to_string());
                }
                ContextMenu::show(ui, menu, just_opened, |ui| {
                    if ContextMenu::button(ui, "Auto-Detect and Create Font", true) {
                        let list: Vec<String> = keys
                            .iter()
                            .filter(|k| selection.contains(**k))
                            .map(|k| k.to_string())
                            .collect();
                        *wizard_state = Some(FontWizardState::new(list));
                        ContextMenu::close(ui);
                    }

                    if is_project_tab {
                        ui.separator();
                        let count = selection.len();
                        let label = if count == 1 {
                            "Delete 1 Asset".to_string()
                        } else {
                            format!("Delete {} Assets", count)
                        };

                        if ContextMenu::button(ui, &label, true) {
                            let list: Vec<String> = keys
                                .iter()
                                .filter(|k| selection.contains(**k))
                                .map(|k| k.to_string())
                                .collect();
                            *confirmation_modal = Some(ConfirmationRequest::DeleteAssets(list));
                            ContextMenu::close(ui);
                        }
                    }
                });
            }

            if response.drag_started() {
                if !is_selected {
                    selection.clear();
                    selection.insert(key.to_string());
                    pivot = Some(key.to_string());
                }
                let list: Vec<String> = keys
                    .iter()
                    .filter(|k| selection.contains(**k))
                    .map(|k| k.to_string())
                    .collect();
                egui::DragAndDrop::set_payload(ui.ctx(), list);
            }

            if response.hovered() {
                response.on_hover_ui(|ui| {
                    ui.label(egui::RichText::new(*key).strong());
                    if let Some(t) = texture {
                        ui.label(format!("{}x{}", t.size()[0], t.size()[1]));
                    }
                });
                if !is_selected {
                    ui.painter().rect_stroke(
                        rect,
                        4.0,
                        egui::Stroke::new(1.0, ui.visuals().widgets.hovered.bg_stroke.color),
                        egui::StrokeKind::Inside,
                    );
                }
            }
        }
    });

    if let Some(asset_keys) = egui::DragAndDrop::payload::<Vec<String>>(ui.ctx()) {
        let count = asset_keys.len();
        let label = if count > 1 {
            format!("{} assets", count)
        } else {
            asset_keys[0].clone()
        };
        let first_key = asset_keys[0].clone();
        let texture = assets.textures.get(&first_key);

        shared::draw_drag_ghost(
            ui.ctx(),
            |ui| {
                thumbnails::draw_thumbnail_widget(ui, texture, Some("?"), false);
            },
            &label,
        );
    }

    ui.data_mut(|d| {
        d.insert_temp(egui::Id::new(ASSET_SEL_KEY), selection);
        d.insert_temp(egui::Id::new(ASSET_PIVOT_KEY), pivot);
    });
}

fn draw_library_item(
    ui: &mut egui::Ui,
    assets: &mut AssetStore,
    lib_asset: &library::LibraryAsset,
    size: f32,
) {
    let stem = AssetStore::stem(lib_asset.name);
    let texture = assets.textures.get(&stem);
    let is_project = assets.raw_files.contains_key(&stem);

    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::click_and_drag());

    if response.hovered() {
        ui.painter()
            .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
    } else {
        thumbnails::draw_thumb_bg(ui, rect);
    }

    if let Some(tex) = texture {
        let scaled_size = tex.size_vec2() * (size - 4.0)
            / tex.size_vec2().x.max(tex.size_vec2().y).max(1.0);
        let tint = if is_project {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_gray(160)
        };
        ui.painter().image(
            tex.id(),
            egui::Rect::from_center_size(rect.center(), scaled_size),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }

    if response.drag_started() {
        if !is_project {
            assets.load_image(ui.ctx(), &stem, lib_asset.bytes);
        }
        egui::DragAndDrop::set_payload(ui.ctx(), vec![stem.clone()]);
    }

    if response.clicked() && !is_project {
        assets.load_image(ui.ctx(), &stem, lib_asset.bytes);
    }

    if response.hovered() {
        response.on_hover_ui(|ui| {
            ui.label(egui::RichText::new(lib_asset.name).strong());
            let msg = if is_project {
                "Saved in Project"
            } else {
                "Built-in Asset (Click to Add)"
            };
            let color = if is_project {
                egui::Color32::GREEN
            } else {
                egui::Color32::GRAY
            };
            ui.label(egui::RichText::new(msg).color(color).size(11.0));
        });
    }
}