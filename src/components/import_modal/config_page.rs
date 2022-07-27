use egui::Ui;

use crate::models::Models;

use super::ImportModal;

pub fn show(parent: &mut ImportModal, _: &mut Models, ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

    ui.heading("Configuration");

    ui.separator();

    egui::Grid::new("my_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .show(ui, |ui| {
            let edge_data_headers = &parent.edge_data.data_headers;
            ui.add(egui::Label::new("Edge Source*"));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Edge Source")
                    .selected_text(edge_data_headers.get(parent.edge_source).unwrap().as_ref())
                    .show_ui(ui, |ui| {
                        for (i, s) in edge_data_headers.iter().enumerate() {
                            ui.selectable_value(&mut parent.edge_source, i, s.as_ref());
                        }
                    });
            });

            ui.end_row();

            ui.add(egui::Label::new("Edge Target*"));
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_source("Edge Target")
                    .selected_text(edge_data_headers.get(parent.edge_target).unwrap().as_ref())
                    .show_ui(ui, |ui| {
                        for (i, s) in edge_data_headers.iter().enumerate() {
                            ui.selectable_value(&mut parent.edge_target, i, s.as_ref());
                        }
                    });
            });

            ui.end_row();
        });
}
