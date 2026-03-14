use super::editor::{LayerContext, LumpUI, PropertyContext, ViewportContext};
use crate::assets::AssetStore;
use crate::document::DocumentAction;
use crate::document::actions::UmapAction;
use crate::models::umapinfo::{MapEntry, UmapField, UmapInfoFile};
use crate::state::PreviewState;
use crate::ui::context_menu::ContextMenu;
use crate::ui::properties::common;
use crate::ui::shared;
use eframe::egui;
use std::collections::HashSet;

/// Metadata for a UMAPINFO field type to facilitate UI lookups and creation.
struct FieldMetadata {
    label: &'static str,
    key_name: &'static str,
    group: &'static str,
}

const FIELD_REGISTRY: &[FieldMetadata] = &[
    FieldMetadata {
        label: "Level Name",
        key_name: "levelname",
        group: "Metadata",
    },
    FieldMetadata {
        label: "Author",
        key_name: "author",
        group: "Metadata",
    },
    FieldMetadata {
        label: "Label/Prefix",
        key_name: "label",
        group: "Metadata",
    },
    FieldMetadata {
        label: "Sky Texture",
        key_name: "skytexture",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Level Pic",
        key_name: "levelpic",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Exit Backdrop",
        key_name: "exitpic",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Enter Backdrop",
        key_name: "enterpic",
        group: "Graphics",
    },
    FieldMetadata {
        label: "Music",
        key_name: "music",
        group: "Audio",
    },
    FieldMetadata {
        label: "Inter. Music",
        key_name: "intermusic",
        group: "Audio",
    },
    FieldMetadata {
        label: "Next Map",
        key_name: "next",
        group: "Progression",
    },
    FieldMetadata {
        label: "Secret Exit",
        key_name: "nextsecret",
        group: "Progression",
    },
    FieldMetadata {
        label: "Par Time",
        key_name: "partime",
        group: "Progression",
    },
    FieldMetadata {
        label: "End Game",
        key_name: "endgame",
        group: "Completion",
    },
    FieldMetadata {
        label: "End Bunny",
        key_name: "endbunny",
        group: "Completion",
    },
    FieldMetadata {
        label: "End Cast",
        key_name: "endcast",
        group: "Completion",
    },
    FieldMetadata {
        label: "No Intermission",
        key_name: "nointermission",
        group: "Completion",
    },
    FieldMetadata {
        label: "Inter. Text",
        key_name: "intertext",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Inter. Text (Secret)",
        key_name: "intertextsecret",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Boss Action",
        key_name: "bossaction",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Boss EdNum",
        key_name: "bossactionednum",
        group: "Advanced",
    },
    FieldMetadata {
        label: "Episode Menu",
        key_name: "episode",
        group: "Advanced",
    },
];

/// Renders the high-level editor for UMAPINFO map entries in the Properties Panel.
pub fn draw_umapinfo_editor(
    ui: &mut egui::Ui,
    file: &mut UmapInfoFile,
    selection_path: &[usize],
    _assets: &AssetStore,
    _state: &PreviewState,
) -> bool {
    let mut changed = false;

    if let Some(map) = file.data.maps.get_mut(selection_path[0]) {
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 180.0).max(0.0) / 2.0);
                ui.label("Map ID:");
                if ui
                    .add(egui::TextEdit::singleline(&mut map.mapname).desired_width(100.0))
                    .changed()
                {
                    map.mapname = map.mapname.to_uppercase();
                    changed = true;
                }
            });
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Keys: {}", map.fields.len())).weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let id = ui.make_persistent_id("add_umap_key_menu");
                    let btn = shared::combobox_button(ui, "Add Key...", 110.0);
                    if btn.clicked() {
                        ContextMenu::open(ui, id, btn.rect.left_bottom());
                    }

                    if let Some(menu) = ContextMenu::get(ui, id) {
                        ContextMenu::show(ui, menu, btn.clicked(), |ui| {
                            changed |= draw_add_field_menu(ui, map);
                        });
                    }
                });
            });
            ui.separator();
            ui.add_space(4.0);

            let mut to_remove = None;
            for (idx, field) in map.fields.iter_mut().enumerate() {
                ui.push_id(idx, |ui| {
                    changed |= draw_field_card(ui, field, &mut to_remove, idx);
                });
                ui.add_space(4.0);
            }

            if let Some(idx) = to_remove {
                map.fields.remove(idx);
                changed = true;
            }
        });
    }
    changed
}

/// Renders an individual Field Card using the unified UI style.
fn draw_field_card(
    ui: &mut egui::Ui,
    field: &mut UmapField,
    remove_idx: &mut Option<usize>,
    idx: usize,
) -> bool {
    let mut changed = false;
    let frame = egui::Frame::NONE
        .fill(egui::Color32::from_white_alpha(5))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_white_alpha(15)))
        .inner_margin(6.0)
        .corner_radius(4.0);

    frame.show(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let del_btn = egui::Button::new("X").min_size(egui::vec2(18.0, 18.0));
                    if ui.add(del_btn).on_hover_text("Remove Key").clicked() {
                        *remove_idx = Some(idx);
                    }

                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        let id = ui.make_persistent_id(format!("change_field_type_{}", idx));
                        let label = FIELD_REGISTRY
                            .iter()
                            .find(|m| m.key_name == field.key_name())
                            .map(|m| m.label)
                            .unwrap_or(field.key_name());

                        let btn_res = shared::combobox_button(ui, label, ui.available_width());

                        if btn_res.clicked() {
                            ContextMenu::open(ui, id, btn_res.rect.left_bottom());
                        }

                        if let Some(menu) = ContextMenu::get(ui, id) {
                            ContextMenu::show(ui, menu, btn_res.clicked(), |ui| {
                                changed |= draw_change_field_menu(ui, field);
                            });
                        }
                    });
                });
            });

            shared::draw_separator_line(ui);
            ui.add_space(2.0);

            if let Some(s) = field.as_string_mut() {
                if ui
                    .add(egui::TextEdit::singleline(s).desired_width(ui.available_width()))
                    .changed()
                {
                    changed = true;
                }
            } else {
                match field {
                    UmapField::Label(s) => {
                        ui.horizontal(|ui| {
                            let mut is_clear = s == "clear";
                            if ui.checkbox(&mut is_clear, "Clear").changed() {
                                *s = if is_clear {
                                    "clear".to_string()
                                } else {
                                    String::new()
                                };
                                changed = true;
                            }
                            if !is_clear {
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(s)
                                            .desired_width(ui.available_width()),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                        });
                    }
                    UmapField::InterText(lines) | UmapField::InterTextSecret(lines) => {
                        ui.vertical(|ui| {
                            let mut is_clear = lines.len() == 1 && lines[0] == "clear";
                            if ui.checkbox(&mut is_clear, "Clear / Disable").changed() {
                                if is_clear {
                                    *lines = vec!["clear".to_string()];
                                } else {
                                    *lines = vec!["New intermission text...".to_string()];
                                }
                                changed = true;
                            }

                            if !is_clear {
                                let mut text_buf = lines.join("\n");
                                if ui
                                    .add(
                                        egui::TextEdit::multiline(&mut text_buf)
                                            .desired_width(ui.available_width())
                                            .desired_rows(3),
                                    )
                                    .changed()
                                {
                                    *lines = text_buf.lines().map(|s| s.to_string()).collect();
                                    changed = true;
                                }
                            }
                        });
                    }
                    UmapField::ParTime(v) => {
                        if ui.add(egui::DragValue::new(v).suffix("s")).changed() {
                            changed = true;
                        }
                    }
                    UmapField::EndGame(v)
                    | UmapField::EndBunny(v)
                    | UmapField::EndCast(v)
                    | UmapField::NoIntermission(v) => {
                        if ui.checkbox(v, "Enabled").changed() {
                            changed = true;
                        }
                    }
                    UmapField::BossAction {
                        thing,
                        special,
                        tag,
                    }
                    | UmapField::BossActionEdNum {
                        ednum: thing,
                        special,
                        tag,
                    } => {
                        ui.horizontal(|ui| {
                            ui.label("ID:");
                            if ui
                                .add(egui::TextEdit::singleline(thing).desired_width(70.0))
                                .changed()
                            {
                                changed = true;
                            }
                            ui.label("Spc:");
                            if ui.add(egui::DragValue::new(special)).changed() {
                                changed = true;
                            }
                            ui.label("Tag:");
                            if ui.add(egui::DragValue::new(tag)).changed() {
                                changed = true;
                            }
                        });
                    }
                    UmapField::Episode { patch, name, key } => {
                        ui.horizontal(|ui| {
                            ui.label("Pat:");
                            ui.add(egui::TextEdit::singleline(patch).desired_width(40.0));
                            ui.label("Nam:");
                            ui.add(egui::TextEdit::singleline(name).desired_width(60.0));
                            ui.label("Key:");
                            ui.add(egui::TextEdit::singleline(key).desired_width(20.0));
                        });
                    }
                    _ => {}
                }
            }
        });
    });
    changed
}

/// Renders the categorized dropdown to add new UMAPINFO keys.
fn draw_add_field_menu(ui: &mut egui::Ui, map: &mut MapEntry) -> bool {
    let mut changed = false;
    let groups = [
        "Metadata",
        "Graphics",
        "Audio",
        "Progression",
        "Completion",
        "Advanced",
    ];

    for group in groups {
        ui.label(egui::RichText::new(group).weak().size(10.0));
        for meta in FIELD_REGISTRY.iter().filter(|m| m.group == group) {
            if common::custom_menu_item(ui, meta.label, false) {
                map.fields.push(create_default_field(meta.key_name));
                changed = true;
                ContextMenu::close(ui);
            }
        }
        ui.separator();
    }
    changed
}

/// Renders the scrollable dropdown to change an existing field's type.
fn draw_change_field_menu(ui: &mut egui::Ui, field: &mut UmapField) -> bool {
    let mut changed = false;
    let current_key = field.key_name();

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            for meta in FIELD_REGISTRY {
                if common::custom_menu_item(ui, meta.label, current_key == meta.key_name) {
                    let old_val = field.as_string_mut().cloned();
                    let mut new_field = create_default_field(meta.key_name);
                    if let (Some(val), Some(new_val_ref)) = (old_val, new_field.as_string_mut()) {
                        *new_val_ref = val;
                    }

                    *field = new_field;
                    changed = true;
                    ContextMenu::close(ui);
                }
            }
        });
    changed
}

/// Internal factory to create a field with its standard spec-default value.
fn create_default_field(key: &str) -> UmapField {
    match key {
        "levelname" => UmapField::LevelName(String::new()),
        "author" => UmapField::Author(String::new()),
        "skytexture" => UmapField::SkyTexture(String::new()),
        "music" => UmapField::Music(String::new()),
        "exitpic" => UmapField::ExitPic(String::new()),
        "enterpic" => UmapField::EnterPic(String::new()),
        "levelpic" => UmapField::LevelPic(String::new()),
        "endpic" => UmapField::EndPic(String::new()),
        "interbackdrop" => UmapField::InterBackdrop(String::new()),
        "intermusic" => UmapField::InterMusic(String::new()),
        "next" => UmapField::Next(String::new()),
        "nextsecret" => UmapField::NextSecret(String::new()),
        "label" => UmapField::Label("clear".to_string()),
        "intertextsecret" => UmapField::InterTextSecret(vec!["clear".to_string()]),
        "partime" => UmapField::ParTime(0),
        "endgame" => UmapField::EndGame(true),
        "endbunny" => UmapField::EndBunny(true),
        "endcast" => UmapField::EndCast(true),
        "nointermission" => UmapField::NoIntermission(true),
        "intertext" => UmapField::InterText(vec!["clear".to_string()]),
        "bossaction" => UmapField::BossAction {
            thing: "Cyberdemon".into(),
            special: 0,
            tag: 0,
        },
        "bossactionednum" => UmapField::BossActionEdNum {
            ednum: "7".into(),
            special: 0,
            tag: 0,
        },
        "episode" => UmapField::Episode {
            patch: "M_EPI1".into(),
            name: "Episode 1".into(),
            key: "e".into(),
        },
        _ => UmapField::LevelName(String::new()),
    }
}

impl LumpUI for UmapInfoFile {
    fn draw_properties(&mut self, ui: &mut egui::Ui, ctx: &PropertyContext) -> bool {
        if let Some(path) = ctx.selection.iter().next() {
            return draw_umapinfo_editor(ui, self, path, ctx.assets, ctx.state);
        }
        false
    }

    fn draw_layer_list(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &mut LayerContext,
    ) -> (Vec<DocumentAction>, bool) {
        let mut actions = Vec::new();
        if shared::heading_action_button(ui, "Maps", Some("Add Map"), false).clicked() {
            actions.push(DocumentAction::UndoSnapshot);
            actions.push(DocumentAction::Umap(UmapAction::AddMap { x: 0.0, y: 0.0 }));
        }

        egui::ScrollArea::vertical()
            .id_salt("umapinfo_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                crate::ui::layers::umapinfo::draw_umapinfo_layers_list(
                    ui,
                    self,
                    ctx.selection,
                    ctx.current_item_idx,
                    ctx.assets,
                    &mut actions,
                    ctx.confirmation_modal,
                );
            });
        (actions, false)
    }

    fn header_info(&self, _selection: &HashSet<Vec<usize>>) -> (String, String, egui::Color32) {
        (
            "UMAPINFO".to_string(),
            "Defines the structure, names of maps, and flow of progression through the WAD."
                .to_string(),
            egui::Color32::from_rgb(60, 40, 40),
        )
    }

    fn render_viewport(&self, ui: &mut egui::Ui, ctx: &mut ViewportContext) -> Vec<DocumentAction> {
        if self.metadata.get("node_positions").is_none() && !self.data.maps.is_empty() {
            let bake_id = ui.make_persistent_id("umap_initial_bake_done");
            let already_baked: bool = ui.ctx().data(|d| d.get_temp(bake_id).unwrap_or(false));

            if !already_baked {
                ui.ctx().data_mut(|d| d.insert_temp(bake_id, true));
                return vec![DocumentAction::Umap(UmapAction::ResetLayout)];
            }
        }
        crate::render::umapinfo::draw_umapinfo_viewport(ui, self, ctx)
    }
}
