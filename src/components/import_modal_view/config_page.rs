use egui::Ui;

use crate::models::Models;

use super::ImportModal;

pub fn show(parent: &mut ImportModal, models: &mut Models, ui: &mut Ui) {

    ui.set_style(ui.ctx().style());
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
                    .selected_text(&models.data_model.edge_data.headers_index_str[parent.edge_source])
                    .show_ui(ui, |ui| {
                        for (i, s) in models.data_model.edge_data.headers_index_str.iter().enumerate() {
                            ui.selectable_value(&mut parent.edge_source, i, s);
                        }
                    });
            });

            ui.end_row();

            ui.add(egui::Label::new("Edge Target*"));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Edge Target")
                    .selected_text(&models.data_model.edge_data.headers_index_str[parent.edge_target])
                    .show_ui(ui, |ui| {
                        for (i, s) in models.data_model.edge_data.headers_index_str.iter().enumerate() {
                            ui.selectable_value(&mut parent.edge_target, i, s);
                        }
                    });
            });

            ui.end_row();
        });
}
