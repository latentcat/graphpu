use egui::Ui;

use crate::MainApp;

use super::AppView;

pub struct GraphicsView;

impl Default for GraphicsView {
    fn default() -> Self {
        Self {}
    }
}

impl AppView for GraphicsView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        let style = (*ui.style()).clone();
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                egui::Frame::none()
                    .fill(style.visuals.extreme_bg_color)
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        ctx.graphic_model.graphic_delegation.custom_painting(ctx, ui);
                    });
            });
    }
}
