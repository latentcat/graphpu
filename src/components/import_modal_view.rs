mod config_page;
mod file_picker_page;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use egui::{Context, Widget};
use tokio::task::JoinHandle;
use crate::constant::ACCENT_COLOR;

use crate::models::app_model::ImportState;
use crate::models::data_model::ExternalData;
use crate::models::{Models, ImportedData};
use crate::utils::csv_loader::{read_headers_from_csv, load_data};
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
    node_file_path: String,
    edge_file_path: String,
    edge_source: usize,
    edge_target: usize,
    import_promise: Option<Receiver<Result<ImportedData, String>>>,
    import_join_handle: Option<JoinHandle<()>>,
}

impl ImportModal {
    pub fn show(&mut self, ctx: &Context, models: &mut Models) {
        Modal::new(String::from("import_modal_view")).show(ctx, |ui| {
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
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| match self.page_index {
                                Page::FilePicker => {
                                    ui.add_enabled_ui(!self.edge_file_path.is_empty(), |ui| {
                                        if egui::Button::new("   Next   ").fill(ACCENT_COLOR).ui(ui).clicked() {
                                            self.on_click_next(models);
                                        }
                                    });
                                    if ui.button("   Cancel   ").clicked() {
                                        models.app_model.is_import_visible = false;
                                    }
                                }
                                Page::Config => {
                                    if egui::Button::new("   Done   ").fill(ACCENT_COLOR).ui(ui).clicked() {
                                        self.on_click_done();
                                    }
                                    let reimport_data_button = ui.button("   Back   ");
                                    if reimport_data_button.clicked() {
                                        self.reset_import_promise();
                                        self.page_index = Page::FilePicker;
                                    }
                                    if self.check_import_done(models) {
                                        ui.spinner();
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
                let edge_data_headers = &models.data_model.edge_data.headers_index_str;
                self.edge_source = edge_data_headers.iter().position(|s| s == "source").unwrap_or(0);
                self.edge_target = edge_data_headers.iter().position(|s| s == "target").unwrap_or(1);
                self.page_index = Page::Config;
                models.app_model.import_state = ImportState::Initial;
            }
            Err(s) => {
                models.app_model.import_state =
                    ImportState::Error(s);
            }
        }
    }

    #[allow(unused_must_use)]
    fn on_click_done(&mut self) {
        let node_file_path = self.node_file_path.clone();
        let edge_file_path = self.edge_file_path.clone();
        let edge_source = self.edge_source;
        let edge_target = self.edge_target;
        let (sender, recv) = mpsc::channel();
        let join_handle = tokio::task::spawn(async move {
            sender.send(load_data(&node_file_path, &edge_file_path, edge_source, edge_target));
        });
        self.import_promise = Some(recv);
        self.import_join_handle = Some(join_handle);
    }

    fn check_import_done(&mut self, models: &mut Models) -> bool {
        if let Some(promise) = self.import_promise.take() {
            match promise.try_recv() {
                Ok(result) => {
                    match result {
                        Ok(data) => {
                            models.setup_data(data);
                            self.reset_import_promise();
                            self.page_index = Page::FilePicker;
                        },
                        Err(s) => {
                            models.data_model.node_data = ExternalData::default();
                            models.data_model.edge_data = ExternalData {
                                headers_str_index: models.data_model.edge_data.headers_str_index.clone(),
                                headers_index_str: models.data_model.edge_data.headers_index_str.clone(),
                                data: Vec::default(),
                            };
                            models.app_model.import_state = ImportState::Error(s);
                        }
                    }
                },
                Err(_) => {
                    self.import_promise = Some(promise);
                    return true;
                }
            }
        }
        false
    }

    fn load_edge_headers(&mut self, models: &mut Models) -> Result<(), String> {
        let (headers_str_index, headers_index_str) = read_headers_from_csv(&Some(PathBuf::from(self.edge_file_path.clone())))?;
        models.data_model.edge_data.headers_str_index = headers_str_index;
        models.data_model.edge_data.headers_index_str = headers_index_str;

        // validate edge data
        if models.data_model.edge_data.headers_str_index.len() < 2 {
            Err("The edge file must contain source and target node IDs".to_owned())
        } else {
            Ok(())
        }
    }

    fn reset_import_promise(&mut self) {
        if let Some(join_handle) = &self.import_join_handle {
            join_handle.abort();
        }
        self.import_promise = None;
        self.import_join_handle = None;
    }
}
