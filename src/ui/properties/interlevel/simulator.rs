use crate::state::PreviewState;
use eframe::egui;

pub fn draw_simulator_panel(ui: &mut egui::Ui, state: &mut PreviewState) {
    let v = &mut state.viewer;

    ui.vertical_centered(|ui| {
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 170.0).max(0.0) / 2.0);
            ui.label("Screen Mode:");
            ui.radio_value(&mut v.ilvl_is_tally, true, "Tally");
            ui.radio_value(&mut v.ilvl_is_tally, false, "Entering");
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
            ui.label("Current Map:");
            ui.add(egui::DragValue::new(&mut v.ilvl_current_map).range(1..=99));
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 140.0).max(0.0) / 2.0);
            ui.label("Is Secret Map:");
            ui.checkbox(&mut v.ilvl_is_secret_map, "");
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 150.0).max(0.0) / 2.0);
            ui.label("Secret Visited:");
            ui.checkbox(&mut v.ilvl_secret_visited, "");
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 170.0).max(0.0) / 2.0);
            ui.label("Earlier Maps Visited:");
            ui.checkbox(&mut v.ilvl_earlier_visited, "");
        });

        ui.add_space(10.0);
    });
}
