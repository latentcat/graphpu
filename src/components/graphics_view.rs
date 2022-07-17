use crate::context::AppContext;

use super::AppView;

pub struct GraphicsView;

impl Default for GraphicsView {
    fn default() -> Self {
        Self {}
    }
}

impl AppView for GraphicsView {
    fn show(self, ctx: &mut AppContext) {
        let style = &ctx.egui_ctx.style();
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx.egui_ctx, |ui| {
                egui::Frame::none()
                    .fill(style.visuals.extreme_bg_color)
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        ctx.app.graphic_model.graphic_delegation.custom_painting(ui);
                    });
            });
    }
}
