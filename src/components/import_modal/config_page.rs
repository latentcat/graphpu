use egui::Ui;

use crate::models::Models;

use super::ImportModal;

pub fn show(parent: &mut ImportModal, models: &mut Models, ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

    ui.heading("Configuration");

    ui.separator();

    egui::Grid::new("my_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .show(ui, |ui| {
            ui.add(egui::Label::new("Edge Source*"));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Edge Source")
                    .selected_text("start_id")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut 0, 0, "start_id");
                        ui.selectable_value(&mut 0, 1, "end_id");
                    })
            });

            ui.end_row();

            ui.add(egui::Label::new("Edge Target*"));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Edge Target")
                    .selected_text("end_id")
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut 1, 0, "start_id");
                        ui.selectable_value(&mut 1, 1, "end_id");
                    })
            });

            ui.end_row();
        });
}
