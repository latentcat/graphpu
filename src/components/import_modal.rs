use egui::Context;

use crate::{widgets::modal::Modal, MainApp};

use super::inspector_view::inner_panel_style;

pub struct ImportModal;

impl ImportModal {
    pub fn show(ctx: &Context, app_ctx: &mut MainApp) {
        Modal::new(String::from("import_modal")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.set_height(250.0);
            egui::TopBottomPanel::bottom("v")
                .frame(inner_panel_style(ui.style()))
                .show_inside(ui, |ui| {
                    ui.set_style(ui.ctx().style());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    ui.horizontal(|ui| {
                        ui.label("");
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::right_to_left(),
                            |ui| {
                                let remove_data_button = ui.button("   Import   ");
                                if remove_data_button.clicked() {

                                }
                                let reimport_data_button = ui.button("   Cancel   ");
                                if reimport_data_button.clicked() {
                                  app_ctx.app_model.import_visible = false;
                                }
                            },
                        );
                    });
                });

            egui::CentralPanel::default()
                .frame(inner_panel_style(ui.style()))
                .show_inside(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    ui.heading("Import Data");

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut 0, 0, "CSV");
                        ui.selectable_value(&mut 0, 1, "GraphML");
                        ui.selectable_value(&mut 0, 2, "DOT");
                    });

                    ui.separator();

                    let mut text = String::from("");

                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            ui.add(egui::Label::new("Node File"));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut text)
                                        .hint_text("")
                                        .desired_width(200.),
                                );
                                ui.button("•••");
                            });

                            ui.end_row();

                            ui.add(egui::Label::new("Edge File"));
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut text)
                                        .hint_text("")
                                        .desired_width(200.),
                                );
                                ui.button("•••");
                            });

                            ui.end_row();
                        });
                });
        });
    }
}