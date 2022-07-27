use egui::{Ui};
use egui::collapsing_header::HeaderResponse;

use crate::models::Models;
use crate::models::app::{ImportState, NodeEdgeTab};
use crate::{models::compute::ComputeMethod};
use crate::models::compute::ComputeMethodType;
use crate::widgets::frames::{button_group_style, inspector_frame, inspector_inner_frame};

use super::AppView;

pub struct InspectorView {
    test_text: String,
}

impl Default for InspectorView {
    fn default() -> Self {
        Self {
            test_text: String::default(),
        }
    }
}

impl AppView for InspectorView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui) {
        let Models { compute_model: model, .. } = models;
        egui::SidePanel::right("inspector_view")
            .frame(inspector_frame(ui.style()))
            .default_width(280.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.text_edit_singleline(&mut self.test_text);

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
                            let node_file_name = models.app_model.node_file_name().unwrap_or("");
                            let edge_file_name = models.app_model.edge_file_name().unwrap_or("");
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                    let remove_data_button = ui.button("🗑");
                                    if remove_data_button.clicked() {
                                        //
                                    }
                                    let reimport_data_button = ui.button("⟲");
                                    if reimport_data_button.clicked() {
                                        //
                                    }

                                    ui.with_layout(
                                        egui::Layout::top_down(egui::Align::LEFT).with_cross_align(egui::Align::Min),
                                        |ui| {
                                            ui.label(egui::RichText::new(format!("{}\n{}", node_file_name, edge_file_name)).strong());
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
                                    ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Node, "Node");
                                });
                                columns[1].vertical_centered_justified(|ui| {
                                    ui.selectable_value(&mut models.app_model.ne_tab, NodeEdgeTab::Edge, "Edge");
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

                                    /// Node Inspector
                                    NodeEdgeTab::Node => {

                                        let (header, is_header_open) = grid_header(ui, true, "ID", "None");
                                        header.body(|ui| {
                                            inspector_grid("Node Inspector ID")
                                                .show(ui, |ui| {

                                                    grid_label(ui, "Source");
                                                    egui::ComboBox::from_id_source("ID Source")
                                                        .selected_text("None")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "");
                                                    ui.button("Set ID");
                                                    ui.end_row();

                                                });
                                        });

                                        let (header, is_header_open) = grid_header(ui, true, "Position", "Compute");
                                        header.body(|ui| {
                                            inspector_grid("Node Inspector ID")
                                                .show(ui, |ui| {

                                                    grid_label(ui, "Source");
                                                    egui::ComboBox::from_id_source("ID Source 2")
                                                        .selected_text("Compute")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "Method");
                                                    ui.horizontal(|ui| {

                                                        egui::ComboBox::from_id_source("Compute Method 2")
                                                            .selected_text(model.compute_method.0)
                                                            .show_ui(ui, |ui| {
                                                                ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                                ui.separator();
                                                                ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                            })

                                                    });

                                                    ui.end_row();

                                                    grid_label(ui, "");
                                                    if model.compute_method.1 == ComputeMethodType::Continuous {
                                                        let continuous_button = ui.button(if !model.is_computing { "▶ Start Computing" } else { "⏸ Pause Computing" });
                                                        if continuous_button.clicked() {
                                                            model.switch_computing();
                                                        }
                                                    } else {
                                                        model.set_computing(false);
                                                        let one_step_button = ui.button("⏩ Dispatch");
                                                        if one_step_button.clicked() {
                                                            model.set_dispatching(true);
                                                        }
                                                    }

                                                    ui.end_row();

                                                });
                                        });

                                        let (header, is_header_open) = grid_header(ui, false, "Color", "None");
                                        header.body(|ui| {
                                            inspector_grid("N Color")
                                                .show(ui, |ui| {

                                                    grid_label(ui, "Source");
                                                    egui::ComboBox::from_id_source("Color Source")
                                                        .selected_text("None")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "Ramp");
                                                    egui::ComboBox::from_id_source("Ramp")
                                                        .selected_text("None")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "");
                                                    ui.button("Set Color");
                                                    ui.end_row();

                                                });
                                        });


                                        let (header, is_header_open) = grid_header(ui, false, "Size", "None");
                                        header.body(|ui| {
                                            inspector_grid("N Size")
                                                .show(ui, |ui| {

                                                    grid_label(ui, "Source");
                                                    egui::ComboBox::from_id_source("Size Source")
                                                        .selected_text("None")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "");
                                                    ui.button("Set Size");
                                                    ui.end_row();

                                                });
                                        });

                                    },

                                    /// Edge Inspector
                                    NodeEdgeTab::Edge => {

                                        let (header, is_header_open) = grid_header(ui, true, "Node ID", "None");
                                        header.body(|ui| {
                                            inspector_grid("Edge Inspector ID")
                                                .show(ui, |ui| {
                                                    grid_label(ui, "Start");
                                                    egui::ComboBox::from_id_source("Start ID")
                                                        .selected_text("None")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "End");
                                                    egui::ComboBox::from_id_source("End ID")
                                                        .selected_text("None")
                                                        .show_ui(ui, |ui| {
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                                                            ui.separator();
                                                            ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                                                        });

                                                    ui.end_row();

                                                    grid_label(ui, "");
                                                    let _ = ui.button("Build Index");

                                                    ui.end_row();

                                                });
                                        });

                                    },
                                };

                            });

                        ui.separator();
                });

            });


    }

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