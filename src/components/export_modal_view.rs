use std::{fs::File, io::Write};

use egui::Context;

use crate::{
    models::Models,
    utils::file::{path_to_string, pick_folder},
    widgets::{
        frames::{inner_panel_frame, DEFAULT_BUTTON_PADDING},
        modal::Modal,
    },
};
use crate::utils::message::message_info;

#[derive(Default)]
pub struct ExportModal {
    directory_path: String,
}

impl ExportModal {
    pub fn show(&mut self, ctx: &Context, models: &mut Models) {
        Modal::new(String::from("export_modal_view")).show(ctx, |ui| {
            ui.set_width(400.0);
            ui.set_height(250.0);

            egui::CentralPanel::default()
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
                    ui.heading("Export Data");
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
                            })
                        })
                });

            egui::TopBottomPanel::bottom("v")
                .frame(inner_panel_frame(ui.style()))
                .show_inside(ui, |ui| {
                    ui.set_style(ui.ctx().style());
                    ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("   Done   ").clicked() {
                            self.on_click_done(models).expect("TODO: panic message");
                            models.app_model.is_export_visible = false;
                        }
                        if ui.button("   Cancel   ").clicked() {
                            models.app_model.is_export_visible = false;
                        }
                    });
                })
        });
    }

    fn on_click_done(&mut self, models: &mut Models) -> std::io::Result<()> {
        let path = self.directory_path.clone() + "/node.pcache";
        if let Some(graphics_resource) = &mut models.graphics_model.graphics_resources {
            graphics_resource.debug();
            if let Some(data) = &graphics_resource.buffer_bytes {

                let mut file = File::create(&path)?;
                file.write_all(b"pcache\n")?;
                file.write_all(b"comment PCACHE file Exported from GraphPU\n")?;
                file.write_all(b"format binary 1.0\n")?;
                file.write_fmt(format_args!("elements {}\n", graphics_resource.status.node_count))?;

                file.write_fmt(format_args!("property {} {}\n", "float", "position.x"))?;
                file.write_fmt(format_args!("property {} {}\n", "float", "position.y"))?;
                file.write_fmt(format_args!("property {} {}\n", "float", "position.z"))?;

                file.write_all(b"end_header\n")?;
                file.write_all(bytemuck::cast_slice(&data))?;
            }
        }
        message_info("Export data succeeded", path.as_str());
        Ok(())
    }
}
