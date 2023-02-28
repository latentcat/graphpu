use std::{fs::File, io::Write};
use std::default::Default;

use egui::Context;
use rayon::prelude::*;

use crate::{
    models::Models,
    utils::file::{path_to_string, pick_folder},
    widgets::{
        frames::{inner_panel_frame, DEFAULT_BUTTON_PADDING},
        modal::Modal,
    },
};
use crate::utils::message::message_info;

pub struct ExportModal {
    directory_path: String,
    file_name: String,
    is_cast_to_float: bool,
}

const DEFAULT_FILE_NAME: &'static str = "graph";

impl Default for ExportModal {
    fn default() -> Self {
        Self {
            directory_path: "".to_string(),
            file_name: "".to_string(),
            is_cast_to_float: false,
        }
    }
}

impl ExportModal {
    pub fn show(&mut self, ctx: &Context, models: &mut Models) {
        Modal::new(String::from("export_modal_view")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.set_height(250.0);

            egui::CentralPanel::default()
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {

                    ui.set_style(ui.ctx().style());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);

                    ui.heading("Export Data");

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut 0, 0, "PCACHE");
                        ui.selectable_value(&mut 0, 1, "CSV");
                    });

                    ui.separator();

                    egui::Grid::new("my_grid")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .show(ui, |ui| {
                            ui.add(egui::Label::new("Directory"));
                            ui.horizontal(|ui| {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().button_padding = DEFAULT_BUTTON_PADDING;

                                        if ui.button("•••").clicked() {
                                            self.directory_path = path_to_string(&pick_folder())
                                                .unwrap_or(self.directory_path.clone());
                                        }

                                        ui.vertical_centered_justified(|ui| {
                                            ui.add(
                                                egui::TextEdit::singleline(
                                                    &mut self.directory_path,
                                                )
                                                .hint_text("")
                                                .desired_width(200.),
                                            );
                                        });
                                    },
                                );
                            });

                            ui.end_row();

                            ui.add(egui::Label::new("File name"));
                            ui.horizontal(|ui| {

                                let text = ui.add(
                                    egui::TextEdit::singleline(
                                        &mut self.file_name,
                                    )
                                        .hint_text("graph")
                                        .desired_width(150.)
                                );
                                let file_name = if self.file_name.len() != 0 { &self.file_name } else { DEFAULT_FILE_NAME };
                                text.on_hover_text(egui::RichText::new(format!("{0}_node.pcache \n{0}_edge.pcache", file_name)).weak());
                            });

                            ui.end_row();

                            ui.add(egui::Label::new(""));
                            ui.checkbox(&mut self.is_cast_to_float, "Cast uint to float")
                                .on_hover_text("For Unity VFX Graph");


                        })
                });

            egui::TopBottomPanel::bottom("v")
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
                    ui.set_style(ui.ctx().style());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_enabled_ui(self.directory_path.len() != 0, |ui| {
                            if ui.button("   Done   ").clicked() {
                                self.on_click_done(models).expect("TODO: panic message");
                                models.app_model.is_export_visible = false;
                            }
                        });
                        if ui.button("   Cancel   ").clicked() {
                            models.app_model.is_export_visible = false;
                        }
                    });
                })
        });
    }

    fn on_click_done(&mut self, models: &mut Models) -> std::io::Result<()> {
        let file_name = if self.file_name.len() != 0 { &self.file_name } else { DEFAULT_FILE_NAME };
        let path_prefix = self.directory_path.clone() + "/" + file_name;
        let node_path = path_prefix.clone() + "_node.pcache";
        let edge_path = path_prefix + "_edge.pcache";

        let graphics_resource = &mut models.graphics_model.graphics_resources;
        graphics_resource.debug();
        if let Some(graph_resources) = &graphics_resource.graph_resources {

            let mut file = File::create(&node_path)?;
            file.write_all(b"pcache\n")?;
            file.write_all(b"comment Node PCACHE file Exported from GraphPU\n")?;
            file.write_all(b"format binary 1.0\n")?;
            file.write_fmt(format_args!("elements {}\n", graph_resources.status.node_count))?;

            file.write_fmt(format_args!("property {} {}\n", "float", "position.x"))?;
            file.write_fmt(format_args!("property {} {}\n", "float", "position.y"))?;
            file.write_fmt(format_args!("property {} {}\n", "float", "position.z"))?;

            file.write_all(b"end_header\n")?;
            file.write_all(bytemuck::cast_slice(graph_resources.buffer_bytes.as_ref().unwrap()))?;
        }


        if let Some(source_target_list) = &models.data_model.source_target_list {

            let result: Vec<u8> = if self.is_cast_to_float {
                let par_iter = source_target_list.par_iter().map(|x| *x as f32);
                let doubles: Vec<f32> = par_iter.collect();
                bytemuck::cast_slice(&doubles).to_vec()
            } else {
                bytemuck::cast_slice(&source_target_list).to_vec()
            };

            let mut file = File::create(&edge_path)?;
            file.write_all(b"pcache\n")?;
            file.write_all(b"comment Edge PCACHE file Exported from GraphPU\n")?;
            file.write_all(b"format binary 1.0\n")?;
            file.write_fmt(format_args!("elements {}\n", models.data_model.status.edge_count))?;

            let type_string = if self.is_cast_to_float { "float" } else { "uint" };
            file.write_fmt(format_args!("property {} {}\n", type_string, "source"))?;
            file.write_fmt(format_args!("property {} {}\n", type_string, "target"))?;

            file.write_all(b"end_header\n")?;
            file.write_all(bytemuck::cast_slice(&result))?;
        }

        let text = format!(
            "Node file: {}  \nEdge file: {}",
            node_path,
            edge_path
        );
        message_info("Export PCACHE Data Succeeded", text.as_str());
        Ok(())
    }
}
