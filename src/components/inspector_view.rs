use std::hash::Hash;

use egui::{CollapsingHeader, CollapsingResponse, Color32, Ui};

use crate::models::Models;
use crate::models::app_model::{ImportState, InspectorTab};
use crate::models::graphics_model::ComputeMethod;
use crate::models::graphics_model::ComputeMethodType;
use crate::models::data_model::{ColorType, ColorRamp, ColorPalette, SizeType};
use crate::utils::file::{path_to_string, pick_folder, system_open_directory};
use crate::widgets::frames::{button_group_style, DEFAULT_BUTTON_PADDING, inspector_frame, inspector_inner_frame};

use super::AppView;

#[derive(Default)]
pub struct InspectorView;

impl AppView for InspectorView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("inspector_view")
            .frame(inspector_frame(ui.style()))
            .default_width(320.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show_inside(ui, |ui| {

                // Render Section
                egui::TopBottomPanel::bottom("render")
                    .frame(inspector_inner_frame(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                        ui.spacing_mut().button_padding = DEFAULT_BUTTON_PADDING;

                        if models.app_model.import_state != ImportState::Success {
                            ui.set_enabled(false);
                        }

                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let folder_open = ui.button("🗁");
                                if folder_open.clicked() {
                                    self.pick_output_folder_and_then(&mut models.app_model.output_folder, |folder| {
                                        system_open_directory(folder);
                                    });
                                }
                                ui.vertical_centered_justified(|ui| {
                                    let render_button = ui.button("Render Image");
                                    if render_button.clicked() {
                                        self.pick_output_folder_and_then(&mut models.app_model.output_folder, |folder| {
                                            models.graphics_model.render_output(String::from(folder));
                                        });
                                    }
                                });
                            },
                        );
                });

                // Main Section
                egui::CentralPanel::default()
                    .frame(inspector_inner_frame(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

                        // Import Section / File Section
                        if matches!(models.app_model.import_state, ImportState::Success) {
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                                    ui.spacing_mut().button_padding = DEFAULT_BUTTON_PADDING;
                                    let remove_data_button = ui.button("🗑");
                                    if remove_data_button.clicked() {
                                        models.clear_data()
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
                                    models.app_model.is_import_visible = true;
                                }
                            });
                        }

                        ui.separator();

                        if models.app_model.import_state != ImportState::Success {
                            ui.set_enabled(false);
                        }

                        // Node Edge Inspector Switch
                        button_group_style(ui.style()).show(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                            ui.columns(5, |columns| {
                                columns[0].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.inspector_tab, InspectorTab::Graph, "Graph");
                                });
                                columns[1].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.inspector_tab, InspectorTab::Node, "Node");
                                });
                                columns[2].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.inspector_tab, InspectorTab::Edge, "Edge");
                                });
                                columns[3].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.inspector_tab, InspectorTab::Camera, "Camera");
                                });
                                columns[4].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.inspector_tab, InspectorTab::Options, "Options");
                                });
                            });
                        });

                        ui.add_space(6.0);

                        // Node Edge Inspector
                        egui::ScrollArea::vertical()
                            // .always_show_scroll(true)
                            .auto_shrink([false, false])
                            .id_source("source")
                            .show(ui, |ui| {
                                match models.app_model.inspector_tab {
                                    InspectorTab::Graph => self.graph_inspector(models, ui),
                                    InspectorTab::Node => self.node_inspector(models, ui),
                                    InspectorTab::Edge => self.edge_inspector(models, ui),
                                    InspectorTab::Camera => self.camera_inspector(models, ui),
                                    InspectorTab::Options => self.options_inspector(models, ui),
                                };

                            });

                        ui.separator();
                });

            });
    }
}

impl InspectorView {
    fn graph_inspector(&mut self, models: &mut Models, ui: &mut Ui) {
        let node_settings = &mut models.data_model.node_settings;

        inspector_section(ui, true, "Transform", |ui| {
            grid_label(ui, "");
            ui.end_row();
        });

        inspector_section(ui, true, "Layout", |ui| {

            grid_label(ui, "Method");
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
                let continuous_button = ui.button(if !models.graphics_model.is_computing { "Start Computing" } else { "Pause Computing" });
                if continuous_button.clicked() {
                    models.graphics_model.switch_computing();
                }
            } else {
                models.graphics_model.set_computing(false);
                let one_step_button = ui.button("⏩ Dispatch");
                if one_step_button.clicked() {
                    models.graphics_model.set_dispatching(true);
                }
            }
            ui.end_row();
        });

    }

    fn node_inspector(&mut self, models: &mut Models, ui: &mut Ui) {
        let node_settings = &mut models.data_model.node_settings;

        inspector_section(ui, true, "Color", |ui| {
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
                    grid_label(ui, "Value");
                    ui.color_edit_button_srgba(&mut node_settings.color_constant);
                    ui.end_row();
                },
                ColorType::Ramp => {
                    let (source, ramp) = &mut node_settings.color_ramp;
                    source_combox("Source", &models.data_model.node_data.headers_index_str, source, ui);
                    grid_label(ui, "Picker");
                    egui::ComboBox::from_id_source("Color Ramp")
                        .selected_text(&ramp.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(ramp, ColorRamp::Ramp1, "Ramp1");
                            ui.selectable_value(ramp, ColorRamp::Ramp2, "Ramp2");
                        });
                    ui.end_row();

                    grid_label(ui, "");
                    let _ = ui.button("Set Color");
                    ui.end_row();
                },
                ColorType::Partition => {
                    let (source, platte) = &mut node_settings.color_partition;
                    source_combox("Color Partition Source", &models.data_model.node_data.headers_index_str, source, ui);
                    grid_label(ui, "Platte");
                    egui::ComboBox::from_id_source("Color Partition")
                        .selected_text(&platte.to_string())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(platte, ColorPalette::Palette1, "Palette1");
                            ui.selectable_value(platte, ColorPalette::Palette2, "Palette2");
                        });
                    ui.end_row();

                    grid_label(ui, "");
                    let _ = ui.button("Set Color");
                    ui.end_row();
                },
            }
        });

        inspector_section(ui, true, "Size", |ui| {
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
                    grid_label(ui, "Value");
                    ui.add(egui::Slider::new(&mut node_settings.size_constant, 0.1..=10.0));
                    ui.end_row();
                },
                SizeType::Ramp => {
                    let (source, _) = &mut node_settings.size_ramp;
                    source_combox("Source", &models.data_model.node_data.headers_index_str, source, ui);
                    grid_label(ui, "Range");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut node_settings.size_ramp.1[0]).speed(0.1));
                        ui.label("—");
                        ui.add(egui::DragValue::new(&mut node_settings.size_ramp.1[1]).speed(0.1));
                    });
                    ui.end_row();

                    grid_label(ui, "");
                    let _ = ui.button("Set Size");
                    ui.end_row();
                }
            }
        });

        inspector_section(ui, false, "Position", |ui| {
            let (source, _) = &mut node_settings.size_ramp;
            source_combox("Source", &models.data_model.node_data.headers_index_str, source, ui);
            grid_label(ui, "");
            let _ = ui.button("Set Position");
            ui.end_row();

        });
    }

    fn edge_inspector(&mut self, _models: &mut Models, ui: &mut Ui) {
        inspector_section(ui, true, "Color", |ui| {
            grid_label(ui, "Value");
            ui.color_edit_button_srgba(&mut Color32::from_rgb(255, 255, 255));
            ui.end_row();
        });

        inspector_section(ui, true, "Width", |ui| {
            grid_label(ui, "Value");
            ui.add(egui::Slider::new(&mut 1.0, 0.1..=10.0));
            ui.end_row();
        });
    }

    fn camera_inspector(&mut self, _models: &mut Models, ui: &mut Ui) {
        inspector_section(ui, true, "Transform", |ui| {
            ui.end_row();
        });
        inspector_section(ui, true, "View", |ui| {
            ui.end_row();
        });
        inspector_section(ui, true, "Composite", |ui| {
            ui.end_row();
        });
    }

    fn options_inspector(&mut self, models: &mut Models, ui: &mut Ui) {

        inspector_section(ui, true, "Output", |ui| {
            grid_label(ui, "Folder");
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                    ui.spacing_mut().button_padding = DEFAULT_BUTTON_PADDING;

                    if ui.button("•••").clicked() {
                        models.app_model.output_folder = path_to_string(&pick_folder()).unwrap_or(models.app_model.output_folder.clone());
                    }

                    ui.vertical_centered_justified(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut models.app_model.output_folder)
                                // .hint_text("未识别的路径")
                                .desired_width(200.)
                        );
                    });

                });
            });
            ui.end_row();

        });

    }
}

impl InspectorView {
    fn pick_output_folder_and_then(&self, output_folder: &mut String, mut then: impl FnMut(&str) -> ()) {
        if output_folder.is_empty() {
            *output_folder = path_to_string(&pick_folder()).unwrap_or(output_folder.clone());
        }
        if !output_folder.is_empty() {
            then(output_folder);
        }
    }
}

fn source_combox(id_source: impl Hash, data_hearders: &Vec<String>, current_value: &mut String, ui: &mut Ui) {
    grid_label(ui, "Source");
    egui::ComboBox::from_id_source(id_source)
        .selected_text(current_value.to_string())
        .show_ui(ui, |ui| {
            ui.selectable_value(current_value, String::from("None"), String::from("None"));
            for value in data_hearders {
                ui.selectable_value(current_value, value.clone(), value);
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
        ui.set_width(100.);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(label)).wrap(true)
            )
        });
    });
}


fn inspector_section<R>(ui: &mut Ui, default_open: bool, title: &str, add_contents: impl FnOnce(&mut Ui) -> R ) -> CollapsingResponse<R> {
    CollapsingHeader::new(title)
        .default_open(default_open)
        .show(ui, |ui| {
            // ui.add_space(6.0);
            let r = inspector_grid(title)
                .show(ui, |ui| {
                    add_contents(ui)
                }).inner;
            // ui.add_space(6.0);

            r
        })
}