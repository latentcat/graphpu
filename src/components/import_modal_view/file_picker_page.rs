use std::path::PathBuf;

use egui::Ui;

use crate::models::{Models};
use crate::utils::csv_loader::pick_csv;
use crate::widgets::frames::DEFAULT_BUTTON_MARGIN;

use super::ImportModal;

fn path_to_string(path: &Option<PathBuf>) -> Option<String> {
    path.as_ref().map(|path| path.display().to_string())
}

pub fn show(parent: &mut ImportModal, _: &mut Models, ui: &mut Ui) {

    ui.set_style(ui.ctx().style());
    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

    ui.heading("Import Data");

    ui.horizontal(|ui| {
        ui.selectable_value(&mut 0, 0, "CSV");
        ui.selectable_value(&mut 0, 1, "GraphML");
        ui.selectable_value(&mut 0, 2, "DOT");
    });

    ui.separator();

    egui::Grid::new("my_grid")
        .num_columns(2)
        .spacing([20.0, 8.0])
        .show(ui, |ui| {
            ui.add(egui::Label::new("Node File"));
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(), |ui| {

                    ui.spacing_mut().button_padding = DEFAULT_BUTTON_MARGIN;

                    if ui.button("•••").clicked() {
                        parent.node_file_path = path_to_string(&pick_csv()).unwrap_or("".to_owned());
                    }

                    ui.vertical_centered_justified(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut parent.node_file_path)
                            .hint_text("")
                            .desired_width(200.)
                        );
                    });
                });
            });

            ui.end_row();

            ui.add(egui::Label::new("Edge File*"));
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(), |ui| {

                    ui.spacing_mut().button_padding = DEFAULT_BUTTON_MARGIN;

                    if ui.button("•••").clicked() {
                        parent.edge_file_path = path_to_string(&pick_csv()).unwrap_or("".to_owned());
                    }

                    ui.vertical_centered_justified(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut parent.edge_file_path)
                            .hint_text("")
                            .desired_width(200.),
                        );
                    });
                });
            });

            ui.end_row();
        });
}