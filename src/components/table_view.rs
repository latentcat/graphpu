use egui_extras::{TableBuilder, Size};

use crate::{
    models::{app::NodeEdgeTab, graphics::ExternalData, Models},
    widgets::frames::button_group_style,
};
use crate::widgets::frames::central_panel_frame;

use super::AppView;

#[derive(Default)]
pub struct TableView;

impl AppView for TableView {
    fn show(&mut self, models: &mut Models, ui: &mut egui::Ui) {
        let style = (*ui.style()).clone();
        egui::CentralPanel::default()
            .frame(central_panel_frame(&style))
            .show_inside(ui, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                ui.horizontal_top(|ui| {
                    button_group_style(ui.style()).show(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Node, "    Node    ");
                        ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Edge, "    Edge    ");
                    });
                });

                ui.separator();

                let ExternalData { data_headers, data } = match models.app_model.ne_tab {
                    NodeEdgeTab::Node => &models.graphic_model.node_data,
                    NodeEdgeTab::Edge => &models.graphic_model.edge_data,
                };

                let text_height = egui::TextStyle::Body.resolve(ui.style()).size + 2.0;

                if data_headers.len() == 0 {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("Import data to display.").weak());
                    });
                    return;
                }

                egui::ScrollArea::horizontal()
                    // .always_show_scroll(true)
                    .auto_shrink([false, false])
                    .id_source("table_scroll")
                    .show(ui, |ui| {
                        TableBuilder::new(ui)
                            .striped(true)
                            .cell_layout(egui::Layout::left_to_right())
                            .columns(Size::initial(100.0).at_least(60.0), if data_headers.len() > 0 { data_headers.len() - 1 } else { 0 })
                            .columns(Size::remainder().at_least(60.0), if data_headers.len() > 0 { 1 } else { 0 })
                            .resizable(true)
                            .header(20.0, |mut header| {
                                for col in data_headers.iter() {
                                    header.col(|ui| {
                                        ui.label(egui::RichText::new(&col[..]).strong());
                                    });
                                }
                            })
                            .body(|mut body| {
                                body.rows(text_height, data.len(), |row_index, mut row| {
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
