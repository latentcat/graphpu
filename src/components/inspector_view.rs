use std::hash::Hash;
use std::rc::Rc;

use egui::{Ui};
use egui::collapsing_header::HeaderResponse;

use crate::models::Models;
use crate::models::app::{ImportState, NodeEdgeTab};
use crate::models::compute::ComputeMethod;
use crate::models::compute::ComputeMethodType;
use crate::models::graphics::{PositionType, ColorType, ColorRamp, ColorPalette, SizeType};
use crate::widgets::frames::{button_group_style, inspector_frame, inspector_inner_frame};

use super::AppView;

#[derive(Default)]
pub struct InspectorView;

impl AppView for InspectorView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::SidePanel::right("inspector_view")
            .frame(inspector_frame(ui.style()))
            .default_width(280.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show_inside(ui, |ui| {

                /// Render Section
                egui::TopBottomPanel::bottom("render")
                    .frame(inspector_inner_frame(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
                            ui.button("⛭");

                            ui.vertical_centered_justified(|ui| {
                                let render_button = ui.button("Render Image");
                                if render_button.clicked() {
                                    //
                                }
                            });
                        });

                });

                /// Main Section
                egui::CentralPanel::default()
                    .frame(inspector_inner_frame(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        /// Import Section / File Section
                        if matches!(models.app_model.import_state, ImportState::Success) {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                    let remove_data_button = ui.button("🗑");
                                    if remove_data_button.clicked() {
                                        models.clear_data();
                                        models.compute_model.reset();
                                    }
                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::LEFT).with_cross_align(egui::Align::Min),
                                        |ui| {
                                            let node_file_name = models.app_model.node_file_name().unwrap_or("");
                                            let edge_file_name = models.app_model.edge_file_name().unwrap_or("");
                                            ui.label(egui::RichText::new(format!("Node File: {}\nEdge File: {}", node_file_name, edge_file_name)).strong());
                                        },
                                    );
                                });
                            });
                        } else {
                            ui.vertical_centered_justified(|ui| {
                                let import_data_button = ui.button("Import Data");
                                if import_data_button.clicked() {
                                    models.app_model.import_visible = true;
                                }
                            });
                        }

                        ui.separator();

                        /// Node Edge Inspector Switch
                        button_group_style(ui.style()).show(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                            ui.columns(2, |columns| {
                                columns[0].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Node, "Node Style");
                                });
                                columns[1].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Edge, "Edge Style");
                                });
                            });
                        });

                        ui.add_space(4.0);

                        /// Node Edge Inspector
                        egui::ScrollArea::vertical()
                            // .always_show_scroll(true)
                            .auto_shrink([false, false])
                            .id_source("source")
                            .show(ui, |ui| {
                                match models.app_model.ne_tab {
                                    NodeEdgeTab::Node => self.node_inspector(models, ui),
                                    NodeEdgeTab::Edge => self.edge_inspector(models, ui),
                                };

                            });

                        ui.separator();
                });

            });
    }
}

impl InspectorView {
    fn node_inspector(&mut self, models: &mut Models, ui: &mut Ui) {
        let node_settings = &mut models.graphic_model.node_settings;
        // TODO: constant editor

        let (header, _) = grid_header(ui, true, "Position", &node_settings.position_type.to_string());
        header.body(|ui| {
            inspector_grid("Node Position")
                .show(ui, |ui| {
                    grid_label(ui, "Type");
                    egui::ComboBox::from_id_source("Position Type")
                        .selected_text(&node_settings.position_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut node_settings.position_type, PositionType::Compute, "Compute");
                            ui.selectable_value(&mut node_settings.position_type, PositionType::Set, "Set");
                        });
                    ui.end_row();

                    match node_settings.position_type {
                        PositionType::Compute => {
                            grid_label(ui, "Compute");
                            egui::ComboBox::from_id_source("Position Compute")
                                .selected_text(node_settings.position_compute.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut node_settings.position_compute, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                    ui.separator();
                                    ui.selectable_value(&mut node_settings.position_compute, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                });
                            ui.end_row();

                            grid_label(ui, "");
                            if node_settings.position_compute.1 == ComputeMethodType::Continuous {
                                let continuous_button = ui.button(if !models.compute_model.is_computing { "▶ Start Computing" } else { "⏸ Pause Computing" });
                                if continuous_button.clicked() {
                                    models.compute_model.switch_computing();
                                }
                            } else {
                                models.compute_model.set_computing(false);
                                let one_step_button = ui.button("⏩ Dispatch");
                                if one_step_button.clicked() {
                                    models.compute_model.set_dispatching(true);
                                }
                            }
                            ui.end_row();
                        },
                        PositionType::Set => {
                            grid_label(ui, "Set");
                            ui.text_edit_singleline(&mut "".to_owned());
                            ui.text_edit_singleline(&mut "".to_owned());
                            ui.text_edit_singleline(&mut "".to_owned());
                            ui.end_row();
                            grid_label(ui, "");
                            if ui.button("Set").clicked() {

                            }
                            ui.end_row();
                        }
                    }
                });
        });

        let (header, _) = grid_header(ui, false, "Color", &node_settings.color_type.to_string());
        header.body(|ui| {
            inspector_grid("Node Color")
                .show(ui, |ui| {
                    grid_label(ui, "Type");
                    egui::ComboBox::from_id_source("Color Type")
                        .selected_text(&node_settings.color_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut node_settings.color_type, ColorType::Constant, "Constant");
                            ui.selectable_value(&mut node_settings.color_type, ColorType::Ramp, "Ramp");
                            ui.selectable_value(&mut node_settings.color_type, ColorType::Partition, "Partition");
                        });
                    ui.end_row();

                    match node_settings.color_type {
                        ColorType::Constant => {
                            grid_label(ui, "Constant");
                            ui.color_edit_button_srgba(&mut node_settings.color_constant);
                            ui.end_row();
                            grid_label(ui, "");
                            if ui.button("Set").clicked() {

                            }
                            ui.end_row();
                        },
                        ColorType::Ramp => {
                            let (source, ramp) = &mut node_settings.color_ramp;
                            source_combox("Color Ramp Source", &models.graphic_model.node_data.data_headers, source, ui);
                            grid_label(ui, "Picker");
                            egui::ComboBox::from_id_source("Color Ramp")
                                .selected_text(&ramp.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(ramp, ColorRamp::Ramp1, "Ramp1");
                                    ui.selectable_value(ramp, ColorRamp::Ramp2, "Ramp2");
                                });
                            ui.end_row();
                        },
                        ColorType::Partition => {
                            let (source, platte) = &mut node_settings.color_partition;
                            source_combox("Color Partition Source", &models.graphic_model.node_data.data_headers, source, ui);
                            grid_label(ui, "Platte");
                            egui::ComboBox::from_id_source("Color Partition")
                                .selected_text(&platte.to_string())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(platte, ColorPalette::Palette1, "Palette1");
                                    ui.selectable_value(platte, ColorPalette::Palette2, "Palette2");
                                });
                            ui.end_row();
                        },
                    }
                });
        });


        let (header, _) = grid_header(ui, false, "Size", &node_settings.size_type.to_string());
        header.body(|ui| {
            inspector_grid("Node Size")
                .show(ui, |ui| {
                    grid_label(ui, "Type");
                    egui::ComboBox::from_id_source("Size Type")
                        .selected_text(&node_settings.size_type.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut node_settings.size_type, SizeType::Constant, "Constant");
                            ui.selectable_value(&mut node_settings.size_type, SizeType::Ramp, "Ramp");
                        });
                    ui.end_row();

                    match node_settings.size_type {
                        SizeType::Constant => {
                            grid_label(ui, "Constant");
                            ui.add(egui::Slider::new(&mut node_settings.size_constant, 0.1..=10.0));
                            ui.end_row();

                            grid_label(ui, "");
                            if ui.button("Set").clicked() {

                            }
                            ui.end_row();
                        },
                        SizeType::Ramp => {
                            let (source, _) = &mut node_settings.size_ramp;
                            source_combox("Size Ramp Source", &models.graphic_model.node_data.data_headers, source, ui);
                            grid_label(ui, "Range");
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut node_settings.size_ramp.1[0]).speed(0.1));
                                ui.label("—");
                                ui.add(egui::DragValue::new(&mut node_settings.size_ramp.1[1]).speed(0.1));
                            });
                            ui.end_row();
                            grid_label(ui, "");
                            if ui.button("Set").clicked() {

                            }
                            ui.end_row();
                        }
                    }
                });
        });
    }

    fn edge_inspector(&mut self, models: &mut Models, ui: &mut Ui) {
    }
}

fn source_combox(id_source: impl Hash, data_hearders: &Vec<Rc<String>>, current_value: &mut Rc<String>, ui: &mut Ui) {
    grid_label(ui, "Source");
    egui::ComboBox::from_id_source(id_source)
        .selected_text(current_value.to_string())
        .show_ui(ui, |ui| {
            for col in data_hearders {
                ui.selectable_value(current_value, col.clone(), &*col.clone());
            }
        });
    ui.end_row();
}

fn inspector_grid(id: &str) -> egui::Grid {
    egui::Grid::new(id)
        .num_columns(2)
        .spacing([10.0, 4.0])
        .min_col_width(65.)
        .min_row_height(10.)
}

fn grid_label(ui: &mut egui::Ui, title: &str) {
    let label = format!("{}", title);
    ui.horizontal(|ui| {
        ui.set_max_width(65.);
        ui.add(
            egui::Label::new(egui::RichText::new(label)).wrap(true)
        )
    });
}

fn grid_category(ui: &mut egui::Ui, title: &str) {
    let label = format!("{}", title);
    ui.horizontal(|ui| {
        ui.set_height(20.);
        ui.add(
            egui::Label::new(egui::RichText::new(label).strong()).wrap(true)
        )
    });
}

fn grid_header<'a>(ui: &'a mut egui::Ui, default_open: bool, title: &str, hint: &str) -> (HeaderResponse<'a, ()>, bool) {
    let id = ui.make_persistent_id(title);
    let header = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, default_open);
    let is_header_open = header.is_open();
    return (header
        .show_header(ui, |ui| {
            if is_header_open {
                ui.strong(title);
            } else {
                ui.horizontal(|ui| {
                    ui.strong(title);
                    ui.with_layout(egui::Layout::right_to_left(), |ui| {
                        ui.weak(hint);
                    });
                });
            }
        }), is_header_open)
}