use eframe::egui;

const MENU_ID_KEY: &str = "cacoco_context_menu_id";
const MENU_POS_KEY: &str = "cacoco_context_menu_pos";

pub struct ContextMenu {
    pub id: egui::Id,
    pub pos: egui::Pos2,
}

impl ContextMenu {
    pub fn check(ui: &egui::Ui, response: &egui::Response) -> bool {
        if response.secondary_clicked() {
            let pos = ui.input(|i| i.pointer.interact_pos().unwrap_or(response.rect.center()));
            ui.data_mut(|d| {
                d.insert_temp(egui::Id::new(MENU_ID_KEY), response.id);
                d.insert_temp(egui::Id::new(MENU_POS_KEY), pos);
            });
            return true;
        }
        false
    }

    pub fn get(ui: &egui::Ui, response_id: egui::Id) -> Option<Self> {
        let open_id: Option<egui::Id> = ui.data(|d| d.get_temp(egui::Id::new(MENU_ID_KEY)));
        if open_id == Some(response_id) {
            let pos: egui::Pos2 = ui.data(|d| d.get_temp(egui::Id::new(MENU_POS_KEY)).unwrap_or_default());
            Some(Self { id: response_id, pos })
        } else {
            None
        }
    }

    pub fn close(ui: &egui::Ui) {
        ui.data_mut(|d| d.remove::<egui::Id>(egui::Id::new(MENU_ID_KEY)));
    }

    pub fn show<F>(ui: &egui::Ui, state: Self, just_opened: bool, add_contents: F)
    where F: FnOnce(&mut egui::Ui)
    {
        let area_res = egui::Area::new(state.id.with("context_menu"))
            .fixed_pos(state.pos)
            .order(egui::Order::Tooltip)
            .constrain(true)
            .show(ui.ctx(), |ui| {
                let frame = egui::Frame::popup(ui.style())
                    .inner_margin(4.0)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(60)));

                frame.show(ui, |ui| {
                    ui.set_min_width(120.0);
                    ui.set_max_width(200.0);
                    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 4.0);
                    add_contents(ui);
                });
            });

        if !just_opened && ui.input(|i| i.pointer.any_click()) {
            if !area_res.response.hovered() {
                Self::close(ui);
            }
        }
    }

    pub fn button(ui: &mut egui::Ui, text: &str, enabled: bool) -> bool {
        let height = 24.0;
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), height),
            egui::Sense::click()
        );

        if enabled && response.hovered() {
            ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(60));
        }

        let text_color = if enabled {
            egui::Color32::from_gray(240)
        } else {
            egui::Color32::from_gray(100)
        };

        ui.painter().text(
            rect.left_center() + egui::vec2(8.0, 0.0),
            egui::Align2::LEFT_CENTER,
            text,
            egui::FontId::proportional(14.0),
            text_color,
        );

        enabled && response.clicked()
    }
}