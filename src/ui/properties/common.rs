use eframe::egui;
use crate::model::{ElementWrapper, Alignment};
use crate::assets::AssetStore;
use crate::ui::layers::thumbnails;
use super::lookups;

pub fn draw_transform_editor(ui: &mut egui::Ui, element: &mut ElementWrapper) -> bool {
    let mut changed = false;
    let common = element.get_common_mut();

    ui.horizontal(|ui| {
        ui.label("X:");
        changed |= ui.add(egui::DragValue::new(&mut common.x)).changed();

        ui.label("Y:");
        changed |= ui.add(egui::DragValue::new(&mut common.y)).changed();
    });

    ui.add_space(4.0);
    ui.label("Alignment:");
    changed |= draw_alignment_selector(ui, &mut common.alignment);

    ui.add_space(4.0);
    changed |= ui.checkbox(&mut common.translucency, "Translucent (Boom Style)").changed();

    changed
}

fn draw_alignment_selector(ui: &mut egui::Ui, align: &mut Alignment) -> bool {
    let mut changed = false;
    let pos_mask = Alignment::H_CENTER | Alignment::RIGHT | Alignment::V_CENTER | Alignment::BOTTOM;
    let current_pos = *align & pos_mask;
    let extras = *align - pos_mask;
    let btn_size = egui::vec2(25.0, 25.0);

    ui.horizontal(|ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
        egui::Grid::new("align_matrix").spacing(egui::vec2(2.0, 2.0)).min_col_width(0.0).show(ui, |ui| {
            let mut toggle = |ui: &mut egui::Ui, target: Alignment| {
                let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
                if response.clicked() {
                    *align = extras | target;
                    changed = true;
                }
                let active = current_pos == target;
                let bg_color = if active { ui.visuals().selection.bg_fill }
                else if response.hovered() { ui.visuals().widgets.hovered.bg_fill }
                else { egui::Color32::from_gray(30) };
                let stroke = if active { ui.visuals().selection.stroke }
                else { egui::Stroke::new(1.0, egui::Color32::from_gray(50)) };
                ui.painter().rect(rect, 2.0, bg_color, stroke, egui::StrokeKind::Middle);
            };
            toggle(ui, Alignment::LEFT | Alignment::TOP);
            toggle(ui, Alignment::H_CENTER | Alignment::TOP);
            toggle(ui, Alignment::RIGHT | Alignment::TOP); ui.end_row();
            
            toggle(ui, Alignment::LEFT | Alignment::V_CENTER);
            toggle(ui, Alignment::H_CENTER | Alignment::V_CENTER);
            toggle(ui, Alignment::RIGHT | Alignment::V_CENTER); ui.end_row();
            
            toggle(ui, Alignment::LEFT | Alignment::BOTTOM); 
            toggle(ui, Alignment::H_CENTER | Alignment::BOTTOM); 
            toggle(ui, Alignment::RIGHT | Alignment::BOTTOM); ui.end_row();
        });
        ui.add_space(8.0);
        ui.vertical(|ui| {
            let mut dyn_left = align.contains(Alignment::DYNAMIC_LEFT);
            if ui.toggle_value(&mut dyn_left, "Dynamic Left").changed() {
                align.set(Alignment::DYNAMIC_LEFT, dyn_left);
                changed = true;
            }
            let mut dyn_right = align.contains(Alignment::DYNAMIC_RIGHT);
            if ui.toggle_value(&mut dyn_right, "Dynamic Right").changed() {
                align.set(Alignment::DYNAMIC_RIGHT, dyn_right);
                changed = true;
            }
        });
    });
    changed
}

pub(super) fn paint_thumb_content(ui: &mut egui::Ui, rect: egui::Rect, tex: Option<&egui::TextureHandle>, fallback_text: Option<&str>) {
    let content_size = rect.width().min(rect.height()) - 4.0;
    if let Some(t) = tex {
        let sz = t.size_vec2();
        if sz.x > 0.0 && sz.y > 0.0 {
            let scale = (content_size / sz.x).min(content_size / sz.y).min(4.0);
            let final_size = sz * if scale >= 1.0 { scale.floor() } else { scale };
            let draw_rect = egui::Rect::from_center_size(rect.center(), final_size);
            ui.painter().image(t.id(), draw_rect, egui::Rect::from_min_max(egui::pos2(0.0,0.0), egui::pos2(1.0,1.0)), egui::Color32::WHITE);
        }
    } else if let Some(text) = fallback_text {
        ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, text, egui::FontId::proportional(14.0), egui::Color32::from_gray(160));
    }
}

pub fn draw_root_statusbar_fields(ui: &mut egui::Ui, bar: &mut crate::model::StatusBarLayout) -> bool {
    let mut changed = false;
    ui.label(egui::RichText::new("Main Settings").strong());

    ui.horizontal(|ui| {
        ui.add_space(2.0);
        ui.label("Bar Height:");
        changed |= ui.add(egui::DragValue::new(&mut bar.height).range(0..=200).speed(1)).changed();
        ui.add_space(2.0);
    });

    ui.horizontal(|ui| {
        ui.add_space(2.0);
        changed |= ui.checkbox(&mut bar.fullscreen_render, "Fullscreen Render").changed();
        ui.add_space(2.0);
    });

    ui.separator();

    ui.label(egui::RichText::new("Widescreen Filling").strong());

    ui.horizontal(|ui| {
        ui.add_space(2.0);
        ui.label("Fill Flat:");

        let mut flat_name = bar.fill_flat.clone().unwrap_or_default();
        let edit_res = ui.add_sized(
            [ui.available_width() - 4.0, 20.0],
            egui::TextEdit::singleline(&mut flat_name)
        );

        if edit_res.changed() {
            bar.fill_flat = if flat_name.is_empty() {
                None
            } else {
                Some(flat_name.to_uppercase())
            };
            changed = true;
        }
        ui.add_space(2.0);
    });

    changed
}

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
    let patch_name = stem.map(|s| assets.resolve_patch_name(s, preview_char, is_number_font));
    let texture = patch_name.and_then(|n| assets.textures.get(&n));
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

pub fn draw_lookup_param_dd(
    ui: &mut egui::Ui,
    salt: &str,
    param: &mut i32,
    items: &[lookups::LookupItem],
    _assets: &AssetStore
) -> bool {
    let mut changed = false;
    let current_name = items.iter().find(|i| i.id == *param).map(|i| i.name).unwrap_or("Unknown");

    egui::ComboBox::from_id_salt(salt)
        .selected_text(current_name)
        .show_ui(ui, |ui| {
            for item in items {
                changed |= ui.selectable_value(param, item.id, item.name).changed();
            }
        });
    changed
}