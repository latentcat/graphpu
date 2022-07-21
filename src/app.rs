use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView,
    },
    models::{compute::ComputeModel, graphics::GraphicsModel},
    widgets::boids::Boids,
};
use egui::epaint;

pub struct MainApp {
    pub compute_model: ComputeModel,
    pub graphic_model: GraphicsModel,
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let mut style = (*cc.egui_ctx.style()).clone();
        // style.visuals.widgets.noninteractive.rounding = epaint::Rounding::from(0.0);
        // cc.egui_ctx.set_style(style);

        Self {
            compute_model: ComputeModel::default(),
            graphic_model: GraphicsModel::new(std::rc::Rc::new(Boids::new(cc))),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                // ui.set_enabled(false);
                MenuBarView::default().show(self, ui);
                InspectorView::default().show(self, ui);
                DetailView::default().show(self, ui);
                GraphicsView::default().show(self, ui);
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {

            });
    }
}
