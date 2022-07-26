use egui::Context;

use crate::models::Models;
use crate::models::app::ImportState;
use crate::models::graphics::read_from_csv;
use crate::widgets::frames::inner_panel_frame;
use crate::widgets::modal::Modal;

pub struct ImportModal;

impl ImportModal {
    pub fn show(ctx: &Context, models: &mut Models) {
        Modal::new(String::from("import_modal")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.set_height(250.0);
            egui::TopBottomPanel::bottom("v")
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
                    ui.set_style(ui.ctx().style());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("{:?}", models.app_model.import_state));
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::right_to_left(),
                            |ui| {
                                let remove_data_button = ui.button("   Import   ");
                                if remove_data_button.clicked() {
                                    let results = [
                                        read_from_csv(&models.app_model.node_file_path).and_then(|data| {
                                            models.graphic_model.node_data = data;
                                            Ok(())
                                        }),
                                        read_from_csv(&models.app_model.edge_file_path).and_then(|data| {
                                            models.graphic_model.edge_data = data;
                                            Ok(())
                                        }),
                                    ];
                                    if results.iter().any(|result| result.is_err()) {
                                        models.app_model.import_state = ImportState::Error;
                                    } else {
                                        models.app_model.import_state = ImportState::Success;
                                        models.app_model.import_visible = false;
                                    }
                                }
                                let reimport_data_button = ui.button("   Cancel   ");
                                if reimport_data_button.clicked() {
                                    models.app_model.import_visible = false;
                                }
                            },
                        );
                    });
                });

            egui::CentralPanel::default()
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
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
                                    if ui.button("•••").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Text File", &["txt", "csv"])
                                            .pick_file()
                                        {
                                            models.app_model.node_file_path =
                                                Some(path)
                                        }
                                    }

                                    ui.vertical_centered_justified(|ui| {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                models
                                                    .app_model
                                                    .node_file_path
                                                    .as_ref()
                                                    .map(|path| path.display().to_string())
                                                    .as_mut()
                                                    .unwrap_or(&mut "".to_string()),
                                            )
                                                .hint_text("")
                                                .desired_width(200.),
                                        );
                                    });
                                });
                            });

                            ui.end_row();

                            ui.add(egui::Label::new("Edge File"));
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                    if ui.button("•••").clicked() {
                                        if let Some(path) = rfd::FileDialog::new()
                                            .add_filter("Text File", &["txt", "csv"])
                                            .pick_file()
                                        {
                                            models.app_model.edge_file_path =
                                                Some(path)
                                        }
                                    }

                                    ui.vertical_centered_justified(|ui| {
                                        ui.add(
                                            egui::TextEdit::singleline(
                                                models
                                                    .app_model
                                                    .edge_file_path
                                                    .as_ref()
                                                    .map(|path| path.display().to_string())
                                                    .as_mut()
                                                    .unwrap_or(&mut "".to_string()),
                                            )
                                                .hint_text("")
                                                .desired_width(200.),
                                        );
                                    });
                                });
                            });

                            ui.end_row();
                        });
                });
        });
    }
}
