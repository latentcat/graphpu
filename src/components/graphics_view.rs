use crate::widgets::GraphicDelegation;

use super::AppView;

pub struct GraphicsView<'a> {
    graphic_object: &'a mut dyn GraphicDelegation,
}

impl<'a> GraphicsView<'a> {
    pub fn new(graphic_object: &'a mut dyn GraphicDelegation) -> Self {
        Self { graphic_object }
    }
}

impl AppView for GraphicsView<'_> {
    fn show(self, ctx: &egui::Context) {
        let style = &ctx.style();
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(style.visuals.extreme_bg_color)
                    .stroke(style.visuals.window_stroke())
                    .show(ui, |ui| {
                        self.graphic_object.custom_painting(ui);
                    });
            });
    }
}
