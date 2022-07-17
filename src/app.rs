use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView,
    },
    widgets::boids::Boids,
};
use egui::epaint;

pub struct MainApp {
    boids: Boids,
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.widgets.noninteractive.rounding = epaint::Rounding::from(0.0);
        cc.egui_ctx.set_style(style);

        Self {
            boids: Boids::new(cc),
        }
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        MenuBarView::default().show(ctx);
        InspectorView::default().show(ctx);
        GraphicsView::new(&mut self.boids).show(ctx);
        DetailView::default().show(ctx);
    }
}
