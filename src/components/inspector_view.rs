use egui::Ui;

use crate::models::app::{ImportState, NodeEdgeTab};
use crate::{models::compute::ComputeMethod, MainApp};
use crate::models::compute::ComputeMethodType;

use super::AppView;

pub struct InspectorView;

impl Default for InspectorView {
    fn default() -> Self {
        Self {}
    }
}

pub fn panel_style(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

pub fn inner_panel_style(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 8.0),
        rounding: egui::Rounding::none(),
        ..Default::default()
    }
}

impl AppView for InspectorView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        let MainApp { compute_model: model, .. } = ctx;
        egui::SidePanel::right("inspector_view")
            .frame(panel_style(ui.style()))
            .default_width(250.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show_inside(ui, |ui| {

                egui::TopBottomPanel::bottom("render")
                    .frame(inner_panel_style(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        ui.vertical_centered_justified(|ui| {
                            let render_button = ui.button("Render Image");
                            if render_button.clicked() {
                                //
                            }
                        });

                });

                egui::CentralPanel::default()
                    .frame(inner_panel_style(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        if matches!(ctx.app_model.import_state, ImportState::Success) {
                            let node_file_name = ctx.app_model.node_file_name().unwrap();
                            let edge_file_name = ctx.app_model.edge_file_name().unwrap();
                                ui.horizontal(|ui| {
                                    ui.label(format!("{} | {}", node_file_name, edge_file_name));
                                    ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::right_to_left(), |ui| {
                                        let remove_data_button = ui.button("🗑");
                                        if remove_data_button.clicked() {
                                            //
                                        }
                                        let reimport_data_button = ui.button("⟲");
                                        if reimport_data_button.clicked() {
                                            //
                                        }
                                    });
                                });
                        } else {
                            ui.vertical_centered_justified(|ui| {
                                let import_data_button = ui.button("Import Data");
                                if import_data_button.clicked() {
                                    ctx.app_model.import_visible = true;
                                }
                            });
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_source("Compute Method")
                                .selected_text(model.compute_method.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                    ui.separator();
                                    ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                });
                            if model.compute_method.1 == ComputeMethodType::Continuous {
                                let continuous_button = ui.button(if !model.is_computing { "▶" } else { "⏸" });
                                if continuous_button.clicked() {
                                    model.switch_computing();
                                }
                            } else {
                                model.set_computing(false);
                                let one_step_button = ui.button("⏩");
                                if one_step_button.clicked() {
                                    model.set_dispatching(true);
                                }
                            }
                        });

                        ui.separator();

                        ui.columns(2, |columns| {
                            columns[0].vertical_centered_justified(|ui| {
                                ui.selectable_value(&mut ctx.app_model.ne_tab, NodeEdgeTab::Node, "Node");
                            });
                            columns[1].vertical_centered_justified(|ui| {
                                ui.selectable_value(&mut ctx.app_model.ne_tab, NodeEdgeTab::Edge, "Edge");
                            });
                        });

                        egui::ScrollArea::vertical()
                            .always_show_scroll(true)
                            .auto_shrink([false, false])
                            .id_source("source")
                            .show(ui, |ui| {
                                ui.label("Inspector View");
                                // lorem_ipsum(ui);
                            });

                        ui.separator();
                });




            });
    }
}