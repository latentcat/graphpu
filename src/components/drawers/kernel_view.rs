use egui::{Color32, epaint, Ui, Vec2};
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
                ui.selectable_value(&mut 0,0,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
                ui.selectable_value(&mut 0,1,"compute");
            });

            ui.separator();

            ui.centered_and_justified(|ui| {
                ui.set_min_height(100.0);
                ui.label(egui::RichText::new("Kernel Error").color(Color32::from_rgb(255, 0, 0)));
            });

        });
    }
}