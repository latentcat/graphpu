use egui::{Color32, Ui};
use crate::components::AppView;
use crate::constant::FONT_SIZE_BODY;

use crate::models::Models;
use crate::widgets::frames::{drawer_kernel_content_frame};


#[derive(Default)]
pub struct KernelView;

impl AppView for KernelView {
    fn show(&mut self, _models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        drawer_kernel_content_frame(ui.style()).show(ui, |ui| {

            ui.set_style(ui.ctx().style());
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            ui.horizontal_wrapped(|ui| {
                for i in 0..10 {

                    let mut job = egui::text::LayoutJob::single_section("✱ ".parse().unwrap(), egui::TextFormat {
                        font_id: egui::FontId::new(FONT_SIZE_BODY, Default::default()),
                        color: if i == 3 || i == 7 { Color32::from_rgb(255, 0, 0) } else { egui::Color32::from_rgb(0, 255, 0) },
                        valign: egui::Align::Center,
                        ..Default::default()
                    });
                    job.append(&*format!("kernel_name({}) ", i), 0.0, egui::TextFormat {
                        font_id: egui::FontId::new(FONT_SIZE_BODY, Default::default()),
                        color: egui::Color32::from_gray(220),
                        valign: egui::Align::Center,
                        ..Default::default()
                    });
                    // ui.label(job);
                    // let mut text = egui::RichText::new(format!("({}) kernel_name", i));
                    // if i == 3 || i == 7 {
                    //     text = text.color(Color32::from_rgb(255, 0, 0))
                    // }
                    ui.selectable_value(&mut 0,i, job);
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                // .always_show_scroll(true)
                .auto_shrink([false; 2])
                .id_source("kernel")
                .show(ui, |ui| {

                    inspector_grid("kernel_grid")
                        .show(ui, |ui| {
                            grid_label(ui, "Kernel ID");
                            grid_content(ui, "0");

                            ui.end_row();

                            grid_label(ui, "Kernel name");
                            grid_content(ui, "compute");

                            ui.end_row();

                            grid_label(ui, "Kernel status code");
                            grid_content(ui, "503");

                            ui.end_row();

                            grid_label(ui, "Status code description");
                            grid_content(ui, "Error(Loop out of limit)");
                        });

                });

            // ui.centered_and_justified(|ui| {
            //     ui.set_min_height(100.0);
            //     ui.label(egui::RichText::new("Error 503"));
            // });

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
    let label = format!("{}: ", title);
    ui.set_width(250.);
    ui.add(
        egui::Label::new(egui::RichText::new(label))
    );
}

fn grid_content(ui: &mut egui::Ui, title: &str) {
    let label = format!("{}", title);
    ui.add(
        egui::Label::new(egui::RichText::new(label)).wrap(true)
    );
}