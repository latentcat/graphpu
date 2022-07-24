use egui::Ui;

use crate::MainApp;

use super::AppView;

#[derive(Default)]
pub struct GraphicsView;

impl AppView for GraphicsView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        let style = (*ui.style()).clone();
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                egui::Frame::none()
                    .fill(style.visuals.extreme_bg_color)
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        let graphic_delegation = ctx.graphic_model.graphic_delegation.clone();
                        graphic_delegation.custom_painting(ctx, ui);
                    });
            });
    }
}
