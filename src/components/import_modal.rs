mod config_page;
mod file_picker_page;

use std::path::PathBuf;

use egui::Context;

use crate::models::app::ImportState;
use crate::models::Models;
use crate::models::graphics::{read_from_csv, ExternalData, read_headers_from_csv};
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
    node_file_path: Option<PathBuf>,
    edge_file_path: Option<PathBuf>,
    edge_source: usize,
    edge_target: usize,
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
                                    ui.add_enabled_ui(self.edge_file_path.is_some(), |ui| {
                                        let next_button = ui.button("   Next   ");
                                        if next_button.clicked() {
                                            self.on_click_next(models);
                                        }
                                    });
                                    if ui.button("   Cancel   ").clicked() {
                                        models.app_model.import_visible = false;
                                    }
                                }
                                Page::Config => {
                                    let remove_data_button = ui.button("   Done   ");
                                    if remove_data_button.clicked() {
                                        self.on_click_done(models);
                                    }
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

    fn on_click_next(&mut self, models: &mut Models) {
        match self.load_edge_headers(models) {
            Ok(_) => {
                let edge_data_headers = &models.graphic_model.edge_data.data_headers;
                self.edge_source = edge_data_headers.iter().position(|s| s.as_ref() == "source").unwrap_or(0);
                self.edge_target = edge_data_headers.iter().position(|s| s.as_ref() == "target").unwrap_or(1);
                self.page_index = Page::Config;
                models.app_model.import_state = ImportState::Initial;
            }
            Err(s) => {
                models.app_model.import_state =
                    ImportState::Error(s);
            }
        }
    }

    fn on_click_done(&mut self, models: &mut Models) {
        if let Err(s) = self.load_data(models) {
            models.app_model.import_state = ImportState::Error(s);
            return;
        }

        let source_key = &models.graphic_model.edge_data.data_headers[self.edge_source];
        let target_key = &models.graphic_model.edge_data.data_headers[self.edge_target];
        let valid = models.graphic_model.edge_data.data.iter()
            .all(|item| {
                item.get(source_key).unwrap().parse::<usize>().is_ok()
                    && item.get(target_key).unwrap().parse::<usize>().is_ok()
            });
        
        if valid {
            models.app_model.node_file_path = self.node_file_path.clone();
            models.app_model.edge_file_path = self.edge_file_path.clone();
            models.app_model.import_state = ImportState::Success;
            models.app_model.import_visible = false;
        } else {
            models.graphic_model.node_data = ExternalData::default();
            models.graphic_model.edge_data = ExternalData {
                data_headers: models.graphic_model.edge_data.data_headers.clone(),
                data: Vec::default(),
            };
            models.app_model.import_state = ImportState::Error("source and target isn't uint".to_owned());
        }
    }

    fn load_edge_headers(&mut self, models: &mut Models) -> Result<(), String> {
        models.graphic_model.edge_data.data_headers = read_headers_from_csv(&self.edge_file_path)?;

        // validate edge data
        if models.graphic_model.edge_data.data_headers.len() < 2 {
            Err("The edge file must contain source and target node IDs".to_owned())
        } else {
            Ok(())
        }
    }

    fn load_data(&mut self, models: &mut Models) -> Result<(), String> {
        models.graphic_model.node_data = read_from_csv(&self.node_file_path).unwrap_or(ExternalData::default());
        models.graphic_model.edge_data = read_from_csv(&self.edge_file_path)?;
        Ok(())
    }
}
