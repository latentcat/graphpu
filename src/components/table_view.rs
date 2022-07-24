use egui_extras::{TableBuilder, Size};

use crate::{
    models::{app::NodeEdgeTab, graphics::ExternalData},
    widgets::frames::button_group_style,
};

use super::AppView;

#[derive(Default)]
pub struct TableView;

impl AppView for TableView {
    fn show(self, ctx: &mut crate::MainApp, ui: &mut egui::Ui) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.horizontal_top(|ui| {
                    button_group_style(ui.style()).show(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ui.selectable_value(&mut ctx.app_model.ne_tab, NodeEdgeTab::Node, "Node");
                        ui.selectable_value(&mut ctx.app_model.ne_tab, NodeEdgeTab::Edge, "Edge");
                    });
                });

                let ExternalData { data_headers, data } = match ctx.app_model.ne_tab {
                    NodeEdgeTab::Node => &ctx.graphic_model.node_data,
                    NodeEdgeTab::Edge => &ctx.graphic_model.edge_data,
                };

                let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

                TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::left_to_right())
                    .columns(Size::remainder().at_least(40.0), data_headers.len())
                    .header(20.0, |mut header| {
                        for col in data_headers.iter() {
                            header.col(|ui| {
                                ui.heading(&col[..]);
                            });
                        }
                    })
                    .body(|mut body| {
                        for data_row in data.iter() {
                            body.row(text_height, |mut row| {
                                for data_col in data_headers {
                                    row.col(|ui| {
                                        ui.label(data_row.get(data_col).unwrap_or(&"".to_string()));
                                    });
                                }
                            })
                        }
                    });
            });
    }
}
