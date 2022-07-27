mod config_page;
mod file_picker_page;

use egui::Context;

use crate::models::app::ImportState;
use crate::models::graphics::read_from_csv;
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
}

impl ImportModal {
    pub fn show(&mut self, ctx: &Context, models: &mut Models) {
        Modal::new(String::from("import_modal")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.set_height(250.0);

            egui::CentralPanel::default()
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
                  match self.page_index {
                    Page::FilePicker => file_picker_page::show(self, models, ui),
                    Page::Config => config_page::show(self, models, ui),
                  }
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
                            |ui| {
                                match self.page_index {
                                  Page::FilePicker => {
                                    let next_button = ui.button("   Next   ");
                                    if next_button.clicked() {
                                        self.page_index = Page::Config;
                                    }
                                  },
                                  Page::Config => {
                                    let remove_data_button = ui.button("   Done   ");
                                    if remove_data_button.clicked() {
                                        let results = [
                                            read_from_csv(&models.app_model.node_file_path)
                                                .and_then(|data| {
                                                    models.graphic_model.node_data = data;
                                                    Ok(())
                                                }),
                                            read_from_csv(&models.app_model.edge_file_path)
                                                .and_then(|data| {
                                                    models.graphic_model.edge_data = data;
                                                    Ok(())
                                                }),
                                        ];
                                        if results.iter().any(|result| result.is_err()) {
                                            models.app_model.import_state =
                                                ImportState::Error(String::from("Unknown Error"));
                                        } else {
                                            models.app_model.import_state = ImportState::Success;
                                            models.app_model.import_visible = false;
                                        }
                                    }
                                    let reimport_data_button = ui.button("   Back   ");
                                    if reimport_data_button.clicked() {
                                        self.page_index = Page::FilePicker;
                                    }
                                  }
                                }
                            },
                        );
                    });
                });
        });
    }
}
