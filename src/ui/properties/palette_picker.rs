use crate::render::palette::DoomPalette;
use eframe::egui;

/// Renders a 16x16 grid of the Doom palette for color selection.
/// Returns Some(index) if a color was clicked this frame.
pub fn draw_palette_grid(
    ui: &mut egui::Ui,
    palette: &DoomPalette,
    current_selection: u8,
) -> Option<u8> {
    let mut selected_index = None;
    let color_size = egui::vec2(18.0, 18.0);
    let spacing = 0.0;

    egui::Frame::new().inner_margin(4.0).show(ui, |ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(spacing, spacing);

        for row in 0..16 {
            ui.horizontal(|ui| {
                for col in 0..16 {
                    let index = (row * 16 + col) as u8;
                    let color = palette.get(index);

                    let (rect, response) = ui.allocate_at_least(color_size, egui::Sense::click());

                    let is_selected = index == current_selection;

                    ui.painter().rect_filled(rect, 0.0, color);

                    if is_selected {
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(2.0, egui::Color32::WHITE),
                            egui::StrokeKind::Inside,
                        );
                    } else if response.hovered() {
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            egui::Stroke::new(1.0, egui::Color32::LIGHT_GRAY),
                            egui::StrokeKind::Inside,
                        );
                    }

                    if response.clicked() {
                        selected_index = Some(index);
                    }

                    response.on_hover_ui(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(format!("Index: {}", index)).strong());
                            ui.add_space(4.0);
                            let c = color;
                            ui.label(
                                egui::RichText::new(format!(
                                    "#{:02X}{:02X}{:02X}",
                                    c.r(),
                                    c.g(),
                                    c.b()
                                ))
                                .weak(),
                            );
                        });
                    });
                }
            });
        }
    });

    selected_index
}
