use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView,
    },
    widgets::boids::Boids, models::{inspector::InspectorModel, graphics::GraphicsModel},
};
use egui::epaint;

pub struct MainApp {
    pub inspector_model: InspectorModel,
    pub graphic_model: GraphicsModel,
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.widgets.noninteractive.rounding = epaint::Rounding::from(0.0);
        cc.egui_ctx.set_style(style);

        Self {
            inspector_model: InspectorModel::default(),
            graphic_model: GraphicsModel::new(Box::new(Boids::new(cc))),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            MenuBarView::default().show(self, ui);
            InspectorView::default().show(self, ui);
            DetailView::default().show(self, ui);
            GraphicsView::default().show(self, ui);
        });
    }
}
