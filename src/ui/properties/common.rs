use super::lookups;
use crate::assets::AssetStore;
use crate::model::{Alignment, ElementWrapper};
use crate::ui::context_menu::ContextMenu;
use crate::ui::layers::thumbnails;
use eframe::egui;

/// Renders the transformation widgets (X, Y, Alignment) shared by all SBARDEF elements.
pub fn draw_transform_editor(ui: &mut egui::Ui, element: &mut ElementWrapper) -> bool {
    let mut changed = false;
    let common = element.get_common_mut();

    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
            ui.label("X:");
            changed |= ui.add(egui::DragValue::new(&mut common.x)).changed();

            ui.add_space(8.0);
            ui.label("Y:");
            changed |= ui.add(egui::DragValue::new(&mut common.y)).changed();
        });

        ui.add_space(4.0);
        ui.label("Alignment:");
        changed |= draw_alignment_selector(ui, &mut common.alignment);

        ui.add_space(4.0);
        changed |= ui
            .checkbox(&mut common.translucency, "Translucent (Boom Style)")
            .changed();
    });

    changed
}

/// Renders the grid-based anchor selector for element alignment.
fn draw_alignment_selector(ui: &mut egui::Ui, align: &mut Alignment) -> bool {
    let mut changed = false;
    let pos_mask = Alignment::H_CENTER | Alignment::RIGHT | Alignment::V_CENTER | Alignment::BOTTOM;
    let current_pos = *align & pos_mask;
    let extras = *align - pos_mask;
    let btn_size = egui::vec2(25.0, 25.0);

    ui.horizontal(|ui| {
        ui.add_space((ui.available_width() - 210.0).max(0.0) / 2.0);
        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

        egui::Grid::new("align_matrix")
            .spacing(egui::vec2(2.0, 2.0))
            .min_col_width(0.0)
            .show(ui, |ui| {
                let mut toggle = |ui: &mut egui::Ui, target: Alignment| {
                    let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
                    if response.clicked() {
                        *align = extras | target;
                        changed = true;
                    }
                    let active = current_pos == target;
                    let bg_color = if active {
                        ui.visuals().selection.bg_fill
                    } else if response.hovered() {
                        ui.visuals().widgets.hovered.bg_fill
                    } else {
                        egui::Color32::from_gray(30)
                    };
                    let stroke = if active {
                        ui.visuals().selection.stroke
                    } else {
                        egui::Stroke::new(1.0, egui::Color32::from_gray(50))
                    };
                    ui.painter()
                        .rect(rect, 2.0, bg_color, stroke, egui::StrokeKind::Middle);
                };

                toggle(ui, Alignment::LEFT | Alignment::TOP);
                toggle(ui, Alignment::H_CENTER | Alignment::TOP);
                toggle(ui, Alignment::RIGHT | Alignment::TOP);
                ui.end_row();

                toggle(ui, Alignment::LEFT | Alignment::V_CENTER);
                toggle(ui, Alignment::H_CENTER | Alignment::V_CENTER);
                toggle(ui, Alignment::RIGHT | Alignment::V_CENTER);
                ui.end_row();

                toggle(ui, Alignment::LEFT | Alignment::BOTTOM);
                toggle(ui, Alignment::H_CENTER | Alignment::BOTTOM);
                toggle(ui, Alignment::RIGHT | Alignment::BOTTOM);
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.vertical(|ui| {
            ui.style_mut().spacing.item_spacing.y = 4.0;
            let mut wl = align.contains(Alignment::WIDESCREEN_LEFT);
            if ui.toggle_value(&mut wl, "Widescreen L").changed() {
                align.set(Alignment::WIDESCREEN_LEFT, wl);
                changed = true;
            }
            let mut wr = align.contains(Alignment::WIDESCREEN_RIGHT);
            if ui.toggle_value(&mut wr, "Widescreen R").changed() {
                align.set(Alignment::WIDESCREEN_RIGHT, wr);
                changed = true;
            }

            let mut nl = align.contains(Alignment::NO_LEFT_OFFSET);
            if ui.toggle_value(&mut nl, "Ignore Left Offset").changed() {
                align.set(Alignment::NO_LEFT_OFFSET, nl);
                changed = true;
            }
            let mut nt = align.contains(Alignment::NO_TOP_OFFSET);
            if ui.toggle_value(&mut nt, "Ignore Top Offset").changed() {
                align.set(Alignment::NO_TOP_OFFSET, nt);
                changed = true;
            }
        });
    });
    changed
}

/// Renders the root-level Status Bar configuration fields.
pub fn draw_root_statusbar_fields(
    ui: &mut egui::Ui,
    bar: &mut crate::model::StatusBarLayout,
) -> bool {
    let mut changed = false;

    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 210.0).max(0.0) / 2.0);
            ui.label("Layout Name:");
            let mut name_buf = bar.name.clone().unwrap_or_default();
            if ui
                .add_sized([110.0, 18.0], egui::TextEdit::singleline(&mut name_buf))
                .changed()
            {
                bar.name = if name_buf.is_empty() {
                    None
                } else {
                    Some(name_buf)
                };
                changed = true;
            }
        });
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
            ui.label("Bar Height:");
            changed |= ui
                .add_enabled(
                    !bar.fullscreen_render,
                    egui::DragValue::new(&mut bar.height)
                        .range(0..=200)
                        .speed(1),
                )
                .changed();
        });

        ui.checkbox(&mut bar.fullscreen_render, "Fullscreen Render");

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 180.0).max(0.0) / 2.0);
            ui.label("Fill Flat:");

            let mut flat_name = bar.fill_flat.clone().unwrap_or_default();
            let edit_res = ui.add_sized([100.0, 18.0], egui::TextEdit::singleline(&mut flat_name));

            if edit_res.changed() {
                bar.fill_flat = if flat_name.is_empty() {
                    None
                } else {
                    Some(flat_name.to_uppercase())
                };
                changed = true;
            }
        });
    });

    changed
}

/// Renders a list of HUD fonts for selection within a dropdown.
pub fn draw_hud_font_selectors(
    ui: &mut egui::Ui,
    current_font: &mut String,
    fonts: &super::font_cache::FontCache,
    assets: &AssetStore,
) -> bool {
    let mut changed = false;
    for (i, name) in fonts.hud_font_names.iter().enumerate() {
        let stem = fonts.get_hud_stem(name);
        changed |= draw_font_selection_row(ui, current_font, name, stem.as_ref(), assets, false, i);
    }
    changed
}

/// Renders a list of number fonts for selection within a dropdown.
pub fn draw_number_font_selectors(
    ui: &mut egui::Ui,
    current_font: &mut String,
    fonts: &super::font_cache::FontCache,
    assets: &AssetStore,
) -> bool {
    let mut changed = false;
    for (i, name) in fonts.number_font_names.iter().enumerate() {
        let stem = fonts.get_number_stem(name);
        changed |= draw_font_selection_row(ui, current_font, name, stem.as_ref(), assets, true, i);
    }
    changed
}

/// Renders a single row in a font selection dropdown, including a live character preview.
pub fn draw_font_selection_row(
    ui: &mut egui::Ui,
    current_val: &mut String,
    target_name: &str,
    stem: Option<&String>,
    assets: &AssetStore,
    is_number_font: bool,
    index: usize,
) -> bool {
    let mut changed = false;

    let preview_char = if is_number_font {
        std::char::from_digit((index % 10) as u32, 10).unwrap_or('0')
    } else {
        (b'A' + (index % 26) as u8) as char
    };

    let patch_id = stem.map(|s| assets.resolve_patch_id(s, preview_char, is_number_font));
    let texture = patch_id.and_then(|id| assets.textures.get(&id));

    let response = thumbnails::ListRow::new(target_name)
        .subtitle(format!("({})", stem.unwrap_or(&"???".to_string())))
        .texture(texture)
        .fallback("?")
        .selected(*current_val == target_name)
        .show(ui);

    if response.clicked() {
        *current_val = target_name.to_string();
        changed = true;
        ui.close();
    }
    changed
}

/// Helper to render a specialized dropdown menu based on a lookup table.
pub fn draw_lookup_param_dd(
    ui: &mut egui::Ui,
    salt: &str,
    param: &mut i32,
    items: &[lookups::LookupItem],
    _assets: &AssetStore,
) -> bool {
    let mut changed = false;
    let current_name = items
        .iter()
        .find(|i| i.id == *param)
        .map(|i| i.name)
        .unwrap_or("Unknown");

    let id = ui.make_persistent_id(salt);
    let button_res = ui.add(egui::Button::new(current_name).min_size(egui::vec2(0.0, 18.0)));

    if button_res.clicked() {
        ContextMenu::open(ui, id, button_res.rect.left_bottom());
    }

    if let Some(menu) = ContextMenu::get(ui, id) {
        ContextMenu::show(ui, menu, button_res.clicked(), |ui| {
            for item in items {
                if ContextMenu::button(ui, item.name, true) {
                    *param = item.id;
                    changed = true;
                    ContextMenu::close(ui);
                }
            }
        });
    }

    changed
}

/// Helper to draw a selectable list item for use in properties dropdowns.
pub fn custom_menu_item(ui: &mut egui::Ui, text: &str, selected: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width().max(100.0), 20.0),
        egui::Sense::click(),
    );

    if response.hovered() || selected {
        ui.painter().rect_filled(
            rect,
            4.0,
            if selected {
                ui.visuals().selection.bg_fill
            } else {
                egui::Color32::from_gray(60)
            },
        );
    }

    ui.painter().text(
        rect.left_center() + egui::vec2(8.0, 0.0),
        egui::Align2::LEFT_CENTER,
        text,
        egui::FontId::proportional(14.0),
        if selected {
            ui.visuals().selection.stroke.color
        } else {
            egui::Color32::from_gray(240)
        },
    );

    response.clicked()
}

/// Specialized helper for painting static thumbnails into a rect.
pub fn paint_thumb_content(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    tex: Option<&egui::TextureHandle>,
    fallback_text: Option<&str>,
) {
    let content_size = rect.width().min(rect.height()) - 4.0;
    if let Some(t) = tex {
        let sz = t.size_vec2();
        if sz.x > 0.0 && sz.y > 0.0 {
            let scale = (content_size / sz.x).min(content_size / sz.y).min(4.0);
            let final_size = sz * if scale >= 1.0 { scale.floor() } else { scale };
            let draw_rect = egui::Rect::from_center_size(rect.center(), final_size);
            ui.painter().image(
                t.id(),
                draw_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    } else if let Some(text) = fallback_text {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(14.0),
            egui::Color32::from_gray(160),
        );
    }
}
