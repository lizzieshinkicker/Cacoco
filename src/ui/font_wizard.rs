use crate::assets::AssetStore;
use crate::model::{HudFontDef, NumberFontDef, SBarDefFile};
use eframe::egui;

pub struct FontWizardState {
    pub font_name: String,
    pub font_type: FontTypeWrapper,
    pub detected_stem: String,
}

#[derive(PartialEq, Clone, Copy)]
pub enum FontTypeWrapper {
    Number,
    Hud,
}

impl FontWizardState {
    pub fn new(selected: Vec<String>) -> Self {
        let (stem, font_type) = analyze_selection(&selected);
        let name = if stem.is_empty() {
            "New Font".to_string()
        } else {
            stem.clone()
        };

        Self {
            font_name: name,
            font_type,
            detected_stem: stem,
        }
    }
}

pub fn draw_font_wizard(
    ctx: &egui::Context,
    state: &mut Option<FontWizardState>,
    file: &mut SBarDefFile,
    assets: &AssetStore,
) -> bool {
    let mut changed = false;
    let mut is_open = state.is_some();
    if !is_open {
        return false;
    }

    let mut close = false;
    let mut register = false;

    egui::Window::new("Register New Font")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            if let Some(data) = state {
                ui.set_width(304.0);

                ui.horizontal(|ui| {
                    ui.label("Font Name:");
                    ui.add(
                        egui::TextEdit::singleline(&mut data.font_name)
                            .desired_width(ui.available_width()),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.radio_value(&mut data.font_type, FontTypeWrapper::Number, "Number");
                    ui.radio_value(&mut data.font_type, FontTypeWrapper::Hud, "HUD");
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Stem Prefix:");
                    ui.add(
                        egui::TextEdit::singleline(&mut data.detected_stem)
                            .desired_width(ui.available_width()),
                    );
                });

                if data.font_type == FontTypeWrapper::Number {
                    ui.label(
                        egui::RichText::new("e.g. Stem 'STT' -> Finds 'STTNUM0'...")
                            .italics()
                            .weak()
                            .size(11.0),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("e.g. Stem 'STCFN' -> Finds 'STCFN033'...")
                            .italics()
                            .weak()
                            .size(11.0),
                    );
                }

                ui.add_space(8.0);
                ui.label(egui::RichText::new("Character Coverage:").strong());

                let scroll_height = 200.0;
                egui::ScrollArea::vertical()
                    .max_height(scroll_height)
                    .min_scrolled_height(scroll_height)
                    .show(ui, |ui| {
                        draw_coverage_tiles(ui, data, assets);
                    });

                ui.add_space(10.0);
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let can_register =
                            !data.font_name.is_empty() && !data.detected_stem.is_empty();
                        if ui
                            .add_enabled(can_register, egui::Button::new("Register Font"))
                            .clicked()
                        {
                            register = true;
                        }
                    });
                });
            }
        });

    if register {
        if let Some(data) = state {
            changed = true;
            match data.font_type {
                FontTypeWrapper::Number => {
                    file.data.number_fonts.push(NumberFontDef {
                        name: data.font_name.clone(),
                        type_: 0,
                        stem: data.detected_stem.clone(),
                    });
                }
                FontTypeWrapper::Hud => {
                    file.data.hud_fonts.push(HudFontDef {
                        name: data.font_name.clone(),
                        type_: 0,
                        stem: data.detected_stem.clone(),
                    });
                }
            }
        }
        close = true;
    }

    if close || !is_open {
        *state = None;
    }

    changed
}

fn analyze_selection(patches: &[String]) -> (String, FontTypeWrapper) {
    if patches.is_empty() {
        return ("".to_string(), FontTypeWrapper::Number);
    }

    let mut common = patches[0].clone();
    for p in &patches[1..] {
        while !p.starts_with(&common) {
            common.pop();
            if common.is_empty() {
                break;
            }
        }
    }

    let mut looks_like_hud = false;
    for p in patches {
        if p.len() >= 3 {
            let suffix = &p[p.len() - 3..];
            if suffix.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(val) = suffix.parse::<u32>() {
                    if val >= 33 && val <= 126 {
                        looks_like_hud = true;
                        break;
                    }
                }
            }
        }
    }

    let mut stem = common.trim_end_matches(|c: char| c.is_ascii_digit()).to_string();

    if looks_like_hud {
        (stem, FontTypeWrapper::Hud)
    } else {
        let suffixes = ["NUM", "PRCNT", "PRCN", "PCNT", "PERCENT", "MINUS"];
        for s in suffixes {
            if stem.ends_with(s) {
                stem = stem[..stem.len() - s.len()].to_string();
                break;
            }
        }

        (stem, FontTypeWrapper::Number)
    }
}

fn draw_coverage_tiles(ui: &mut egui::Ui, data: &FontWizardState, assets: &AssetStore) {
    let chars_to_check: Vec<(char, String)> = match data.font_type {
        FontTypeWrapper::Number => {
            vec![
                ('0', "0".to_string()),
                ('1', "1".to_string()),
                ('2', "2".to_string()),
                ('3', "3".to_string()),
                ('4', "4".to_string()),
                ('5', "5".to_string()),
                ('6', "6".to_string()),
                ('7', "7".to_string()),
                ('8', "8".to_string()),
                ('9', "9".to_string()),
                ('%', "%".to_string()),
                ('-', "-".to_string()),
            ]
        }
        FontTypeWrapper::Hud => (33..=95)
            .map(|b| {
                let c = b as u8 as char;
                let label = match c {
                    ' ' => "SPC".to_string(),
                    _ => format!("{}", c),
                };
                (c, label)
            })
            .collect(),
    };

    let tile_size = 30.0;

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

        for (c, label) in chars_to_check.iter() {
            let is_num = data.font_type == FontTypeWrapper::Number;

            let resolved = assets.resolve_patch_name(&data.detected_stem, *c, is_num);
            let texture = assets.textures.get(&resolved);
            let found = texture.is_some();

            let display_name = if found {
                resolved
            } else if is_num {
                match *c {
                    '0'..='9' => format!("{}NUM{}", data.detected_stem, c),
                    '%' => format!("{}PRCNT", data.detected_stem),
                    '-' => format!("{}MINUS", data.detected_stem),
                    _ => resolved,
                }
            } else {
                resolved
            };

            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(tile_size, tile_size), egui::Sense::hover());

            if found {
                ui.painter()
                    .rect_filled(rect, 3.0, egui::Color32::from_rgb(20, 50, 20));
                ui.painter().rect_stroke(
                    rect,
                    3.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(20, 255, 20, 20)),
                    egui::StrokeKind::Inside,
                );
                if let Some(tex) = texture {
                    let tex_size = tex.size_vec2();
                    if tex_size.x > 0.0 && tex_size.y > 0.0 {
                        let content_size = tile_size - 4.0;
                        let scale = (content_size / tex_size.x)
                            .min(content_size / tex_size.y)
                            .min(2.0);
                        let final_size = tex_size * scale;
                        let draw_rect = egui::Rect::from_center_size(rect.center(), final_size);
                        ui.painter().image(
                            tex.id(),
                            draw_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    }
                }
            } else {
                ui.painter()
                    .rect_filled(rect, 3.0, egui::Color32::from_rgb(60, 20, 20));
                ui.painter().rect_stroke(
                    rect,
                    3.0,
                    egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 20, 20, 20)),
                    egui::StrokeKind::Inside,
                );
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    label,
                    egui::FontId::proportional(16.0),
                    egui::Color32::from_rgb(255, 100, 100),
                );
            }

            if response.hovered() {
                response.on_hover_ui(|ui| {
                    if found {
                        ui.label(
                            egui::RichText::new("✔ Found")
                                .color(egui::Color32::GREEN)
                                .strong(),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("✖ Missing")
                                .color(egui::Color32::RED)
                                .strong(),
                        );
                    }
                    ui.label(format!("Looking for: {}", display_name));
                });
            }
        }
    });
}
