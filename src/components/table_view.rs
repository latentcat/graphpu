use egui_extras::{TableBuilder, Size};

use crate::{
    models::{app_model::TableTab, data_model::ExternalData, Models},
    widgets::frames::button_group_style,
};
use crate::models::app_model::ImportState;
use crate::widgets::frames::central_panel_frame;

use super::AppView;

#[derive(Default)]
pub struct TableView;

impl AppView for TableView {
    fn show(&mut self, models: &mut Models, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let style = (*ui.style()).clone();
        egui::CentralPanel::default()
            .frame(central_panel_frame(&style))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                ui.horizontal_top(|ui| {
                    button_group_style(ui.style()).show(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ui.selectable_value(&mut models.app_model.table_tab, TableTab::Node, "  Node Data  ");
                        ui.selectable_value(&mut models.app_model.table_tab, TableTab::Edge, "  Edge Data  ");
                    });
                });

                ui.separator();

                let ExternalData { headers_index_str: data_headers, data, .. } = match models.app_model.table_tab {
                    TableTab::Node => &models.data_model.node_data,
                    TableTab::Edge => &models.data_model.edge_data,
                };

                if models.app_model.import_state != ImportState::Success {
                    ui.centered_and_justified(|ui| {
                        let empty_hint_text = match models.app_model.table_tab {
                            TableTab::Node => "Import node data to display.",
                            TableTab::Edge => "Import edge data to display.",
                        };
                        ui.label(egui::RichText::new(empty_hint_text).weak());
                    });
                    return;
                }

                let text_height = egui::TextStyle::Body.resolve(ui.style()).size + 2.0;

                egui::ScrollArea::horizontal()
                    // .always_show_scroll(true)
                    .auto_shrink([false, false])
                    .id_source("table_scroll")
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .striped(true)
                            .cell_layout(egui::Layout::left_to_right())
                            .columns(Size::initial(100.0).at_least(60.0), if data_headers.len() > 0 { data_headers.len() } else { 0 })
                            .columns(Size::remainder().at_least(60.0), 1)
                            .resizable(true)
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.label(egui::RichText::new("Index").weak());
                                });
                                for col in data_headers.iter() {
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new(col).strong());
                                    });
                                }
                            })
                            .body(|body| {
                                body.rows(text_height, models.data_model.status.node_count, |row_index, mut row| {
                                    row.col(|ui| {
                                        ui.label(egui::RichText::new(row_index.to_string()).weak());
                                    });
                                    if row_index >= data.len() {
                                        for _data_col in data_headers {
                                            row.col(|ui| {
                                                ui.label(egui::RichText::new("N/A").weak());
                                            });
                                        }
                                    } else {
                                        let data_row = &data[row_index];
                                        for data_col in data_row {
                                            row.col(|ui| {
                                                ui.label(data_col);
                                            });
                                        }
                                    }

                                })
                            });
                    });

            });
    }
}
