use crate::ui::layers::thumbnails;
use eframe::egui;

/// Unique key for storing the viewport bounds in egui temp data.
pub const VIEWPORT_RECT_ID: &str = "cacoco_viewport_rect";

/// Draws an image scaled and centered within a given rect, respecting aspect ratio.
pub fn draw_scaled_image(
    ui: &egui::Ui,
    rect: egui::Rect,
    tex: &egui::TextureHandle,
    tint: egui::Color32,
    max_scale: f32,
) {
    let tex_size = tex.size_vec2();
    if tex_size.x > 0.0 && tex_size.y > 0.0 {
        let raw_scale = (rect.width() / tex_size.x)
            .min(rect.height() / tex_size.y)
            .min(max_scale);

        let scale = if raw_scale >= 1.0 {
            raw_scale.floor()
        } else {
            raw_scale
        };
        let final_size = tex_size * scale;

        let left = (rect.left() + (rect.width() - final_size.x) / 2.0).floor();
        let top = (rect.top() + (rect.height() - final_size.y) / 2.0).floor();
        let draw_rect = egui::Rect::from_min_size(egui::pos2(left, top), final_size);

        ui.painter().image(
            tex.id(),
            draw_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );
    }
}

/// Standard frame style for drag-and-drop "ghost" elements.
pub fn drag_ghost_frame() -> egui::Frame {
    egui::Frame::default()
        .inner_margin(6.0)
        .corner_radius(4.0)
        .fill(egui::Color32::from_black_alpha(200))
        .stroke(egui::Stroke::new(1.0, egui::Color32::WHITE))
}

/// Draws a standard Cacoco-style drag ghost that follows the pointer.
/// Will automatically suppress itself if the pointer is inside the Viewport.
pub fn draw_drag_ghost(ctx: &egui::Context, icon_content: impl FnOnce(&mut egui::Ui), label: &str) {
    let pointer_pos = ctx.input(|i| i.pointer.latest_pos());

    let viewport_rect: Option<egui::Rect> =
        ctx.data(|d| d.get_temp(egui::Id::new(VIEWPORT_RECT_ID)));
    if let (Some(vr), Some(pp)) = (viewport_rect, pointer_pos) {
        if vr.contains(pp) {
            return;
        }
    }

    if let Some(pos) = pointer_pos {
        egui::Area::new(egui::Id::new("cacoco_drag_ghost"))
            .interactable(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_pos(pos)
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                drag_ghost_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.allocate_ui(
                            egui::vec2(thumbnails::THUMB_SIZE, thumbnails::THUMB_SIZE),
                            |ui| {
                                icon_content(ui);
                            },
                        );
                        ui.label(
                            egui::RichText::new(label)
                                .strong()
                                .color(egui::Color32::WHITE),
                        );
                    });
                });
            });
    }
}

/// Draws a yellow horizontal line across a rect at height 'y' to indicate a drop target.
pub fn draw_yellow_line(ui: &egui::Ui, rect: egui::Rect, y: f32) {
    ui.painter().line_segment(
        [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
        egui::Stroke::new(2.0, egui::Color32::YELLOW),
    );
}

/// Draws a centered placeholder text indicating no file is currently active.
pub fn draw_no_file_placeholder(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(10.0);
        ui.label(egui::RichText::new("No file loaded").weak());
    });
}

/// A stylized header button used for section switching or expansion.
pub fn section_header_button(
    ui: &mut egui::Ui,
    label: &str,
    subheading: Option<&str>,
    active: bool,
) -> egui::Response {
    section_header_button_impl(ui, label, subheading, active, 28.0, 14.0, false)
}

/// A smaller version for tight spaces like the browser tabs.
pub fn compact_header_button(
    ui: &mut egui::Ui,
    label: &str,
    subheading: Option<&str>,
    active: bool,
) -> egui::Response {
    section_header_button_impl(ui, label, subheading, active, 24.0, 11.0, false)
}

/// A large header button with heading-style text and optional right-aligned action text.
pub fn heading_action_button(
    ui: &mut egui::Ui,
    label: &str,
    subheading: Option<&str>,
    active: bool,
) -> egui::Response {
    section_header_button_impl(ui, label, subheading, active, 32.0, 18.0, true)
}

fn section_header_button_impl(
    ui: &mut egui::Ui,
    label: &str,
    subheading: Option<&str>,
    active: bool,
    h: f32,
    font_sz: f32,
    is_heading: bool,
) -> egui::Response {
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), h), egui::Sense::click());

    let mut bg_color = ui.visuals().widgets.noninteractive.bg_fill;
    if active {
        bg_color = egui::Color32::from_rgba_unmultiplied(60, 130, 255, 15);
    }
    if response.hovered() {
        bg_color = ui.visuals().widgets.hovered.bg_fill;
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    ui.painter().rect(
        rect,
        4.0,
        bg_color,
        egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)),
        egui::StrokeKind::Inside,
    );

    let text_color = if active || is_heading {
        ui.visuals().text_color()
    } else {
        ui.visuals().weak_text_color()
    };

    let font_id = egui::FontId::proportional(font_sz);

    if let Some(sub) = subheading {
        ui.painter().text(
            rect.left_center() + egui::vec2(8.0, -1.0),
            egui::Align2::LEFT_CENTER,
            label,
            font_id,
            text_color,
        );

        ui.painter().text(
            rect.right_center() + egui::vec2(-8.0, -1.0),
            egui::Align2::RIGHT_CENTER,
            sub,
            egui::FontId::proportional((font_sz * 0.75).max(10.0)),
            ui.visuals().weak_text_color(),
        );
    } else {
        ui.painter().text(
            rect.center() + egui::vec2(0.0, -1.0),
            egui::Align2::CENTER_CENTER,
            label,
            font_id,
            text_color,
        );
    }

    response
}

/// Draws a button styled exactly like a standard egui::ComboBox button.
pub fn combobox_button(ui: &mut egui::Ui, text: &str, width: f32) -> egui::Response {
    let height = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        ui.painter().rect(
            rect,
            visuals.corner_radius,
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );

        let text_rect = rect.shrink(ui.spacing().button_padding.x);
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        ui.painter().text(
            text_rect.left_center(),
            egui::Align2::LEFT_CENTER,
            text,
            font_id,
            visuals.text_color(),
        );

        let arrow_size = 4.0;
        let arrow_pos = rect.right_center() - egui::vec2(ui.spacing().item_spacing.x + 4.0, 0.0);

        let points = vec![
            arrow_pos + egui::vec2(-arrow_size, -arrow_size * 0.6),
            arrow_pos + egui::vec2(arrow_size, -arrow_size * 0.6),
            arrow_pos + egui::vec2(0.0, arrow_size * 0.6),
        ];

        ui.painter().add(egui::Shape::convex_polygon(
            points,
            visuals.fg_stroke.color,
            egui::Stroke::NONE,
        ));
    }

    response
}

pub fn truncate_path(path: &str, max_chars: usize) -> String {
    if path.len() <= max_chars {
        return path.to_string();
    }
    let half = (max_chars - 3) / 2;
    format!("{}...{}", &path[..half], &path[path.len() - half..])
}
