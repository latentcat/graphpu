mod config_page;
mod file_picker_page;

use egui::Context;

use crate::models::app::{AppModel, ImportState};
use crate::models::graphics::{read_from_csv, ExternalData};
use crate::models::Models;
use crate::widgets::frames::inner_panel_frame;
use crate::widgets::modal::Modal;

#[derive(Default, PartialEq)]
enum Page {
    #[default]
    FilePicker,
    Config,
}

#[derive(Default)]
pub struct ImportModal {
    page_index: Page,
    node_data: ExternalData,
    edge_data: ExternalData,
}

impl ImportModal {
    pub fn show(&mut self, ctx: &Context, models: &mut Models) {
        Modal::new(String::from("import_modal")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.set_height(250.0);

            egui::CentralPanel::default()
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| match self.page_index {
                    Page::FilePicker => file_picker_page::show(self, models, ui),
                    Page::Config => config_page::show(self, models, ui),
                });

            egui::TopBottomPanel::bottom("v")
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
                    ui.set_style(ui.ctx().style());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    ui.horizontal(|ui| {
                        if let ImportState::Error(message) = &models.app_model.import_state {
                            ui.label(message);
                        }
                        ui.allocate_ui_with_layout(
                            ui.available_size(),
                            egui::Layout::right_to_left(),
                            |ui| match self.page_index {
                                Page::FilePicker => {
                                    let next_button = ui.button("   Next   ");
                                    if next_button.clicked() {
                                        let AppModel {
                                            node_file_path,
                                            edge_file_path,
                                            ..
                                        } = &models.app_model;
                                        match models
                                            .graphic_model
                                            .load_data(node_file_path, edge_file_path)
                                        {
                                            Ok(_) => self.page_index = Page::Config,
                                            Err(s) => {
                                                models.app_model.import_state =
                                                    ImportState::Error(s)
                                            }
                                        }
                                    }
                                }
                                Page::Config => {
                                    let remove_data_button = ui.button("   Done   ");
                                    if remove_data_button.clicked() {}
                                    let reimport_data_button = ui.button("   Back   ");
                                    if reimport_data_button.clicked() {
                                        self.page_index = Page::FilePicker;
                                    }
                                }
                            },
                        );
                    });
                });
        });
    }
}
