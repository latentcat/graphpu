
use egui::{Color32, Ui};
use crate::components::AppView;
use crate::constant::FONT_SIZE_BODY;
use crate::models::graphics_model::KERNEL_NAMES;

use crate::models::Models;
use crate::widgets::frames::{drawer_kernel_content_frame};


#[derive(Default)]
pub struct KernelView {
    pub selected_kernel: usize,
}

fn get_kernel_status_desc(code: i32) -> &'static str {
    match code {
        -1  => "Uninitialized",
        0   => "Running",
        _   => ""
    }
}

impl AppView for KernelView {
    fn show(&mut self, _models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        drawer_kernel_content_frame(ui.style()).show(ui, |ui| {

            ui.set_style(ui.ctx().style());
            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

            ui.horizontal_wrapped(|ui| {
                if let Some(graphics_resources) = &mut _models.graphics_model.graphics_resources {
                    let _kernels = &mut graphics_resources.graph_compute.kernels;
                    for (index, &name) in KERNEL_NAMES.iter().enumerate() {
                        kernel_label(ui, &mut self.selected_kernel ,index, name, graphics_resources.kernel_status_codes[index]);
                    }
                }
            });

            ui.separator();

            egui::ScrollArea::vertical()
                // .always_show_scroll(true)
                .auto_shrink([false; 2])
                .id_source("kernel")
                .show(ui, |ui| {

                    let mut code = 0;

                    if let Some(graphics_resources) = &mut _models.graphics_model.graphics_resources {
                        code = graphics_resources.kernel_status_codes[self.selected_kernel];
                    }

                    inspector_grid("kernel_grid")
                        .show(ui, |ui| {
                            grid_label(ui, "Kernel ID");
                            grid_content(ui, format!("{}", self.selected_kernel).as_str());

                            ui.end_row();

                            grid_label(ui, "Kernel name");
                            grid_content(ui, KERNEL_NAMES[self.selected_kernel]);

                            ui.end_row();

                            grid_label(ui, "Kernel status code");
                            grid_content(ui, format!("{}", code).as_str());

                            ui.end_row();

                            grid_label(ui, "Status code description");
                            grid_content(ui, get_kernel_status_desc(code));
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

fn kernel_label(ui: &mut egui::Ui, selected_kernel: &mut usize, kernel_index: usize, kernel_name: &str, kernel_code: i32) {

    let size = egui::Vec2::new(150., 18.);
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
    ui.allocate_ui_at_rect(rect, |ui| {
        ui.set_min_width(150.);

        let mut job = egui::text::LayoutJob::single_section("✱ ".parse().unwrap(), egui::TextFormat {
            font_id: egui::FontId::new(FONT_SIZE_BODY, Default::default()),
            color: match kernel_code {
                x if x < 0 => Color32::from_rgb(255, 255, 0),
                0 => Color32::from_rgb(0, 255, 0),
                x if x > 0 => Color32::from_rgb(255, 0, 0),
                _ => Color32::from_rgb(255, 0, 0)

            },
            valign: egui::Align::Center,
            ..Default::default()
        });
        job.append(&*format!("[{}] {} ", kernel_index, kernel_name), 0.0, egui::TextFormat {
            font_id: egui::FontId::new(FONT_SIZE_BODY, Default::default()),
            color: egui::Color32::from_gray(220),
            valign: egui::Align::Center,
            ..Default::default()
        });
        ui.selectable_value(selected_kernel, kernel_index, job);

    });
    // ui.painter()
    //     .circle_filled(rect.center(), r, ui.visuals().text_color());

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