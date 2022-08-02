use egui_extras::{TableBuilder, Size};

use crate::{
    models::{app::NodeEdgeTab, graphics::ExternalData, Models},
    widgets::frames::button_group_style,
};
use crate::models::app::ImportState;
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
                        ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Node, "  Node Data  ");
                        ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Edge, "  Edge Data  ");
                    });
                });

                ui.separator();

                let ExternalData { data_headers, data } = match models.app_model.ne_tab {
                    NodeEdgeTab::Node => &models.graphic_model.node_data,
                    NodeEdgeTab::Edge => &models.graphic_model.edge_data,
                };

                if models.app_model.import_state != ImportState::Success {
                    ui.centered_and_justified(|ui| {
                        let empty_hint_text = match models.app_model.ne_tab {
                            NodeEdgeTab::Node => "Import node data to display.",
                            NodeEdgeTab::Edge => "Import edge data to display.",
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
                            .column(Size::initial(70.0).at_least(70.0).at_most(100.))
                            .columns(Size::initial(100.0).at_least(60.0), if data_headers.len() > 0 { data_headers.len() - 1 } else { 0 })
                            .columns(Size::remainder().at_least(60.0), if data_headers.len() > 0 { 1 } else { 0 })
                            .resizable(true)
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.label(egui::RichText::new("Index").weak());
                                });
                                for col in data_headers.iter() {
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new(&col[..]).strong());
                                    });
                                }
                            })
                            .body(|body| {
                                body.rows(text_height, data.len(), |row_index, mut row| {
                                    row.col(|ui| {
                                        ui.label(egui::RichText::new(row_index.to_string()).weak());
                                        // ui.with_layout(egui::Layout::right_to_left(), |ui| {});
                                    });
                                    let data_row = &data[row_index];
                                    for data_col in data_headers {
                                        row.col(|ui| {
                                            ui.label(data_row.get(data_col).unwrap_or(&"".to_string()));
                                        });
                                    }
                                })
                            });
                    });

            });
    }
}
