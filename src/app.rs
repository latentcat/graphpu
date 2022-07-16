use crate::{
    components::{main_canvas::MainCanvas, AppComponent},
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
        egui::TopBottomPanel::top("menubar_view").show(ctx, |ui| {
            egui::menu::bar(ui, |_| {
                // TODO: Menu Bar
            });
        });

        egui::SidePanel::right("inspector_view")
            .resizable(false)
            .show(ctx, |_| {
                // TODO: Inspector
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                MainCanvas::new(&mut self.boids).add(ctx, ui);
            });

        egui::TopBottomPanel::bottom("detail").show(ctx, |_| {
            // TODO: Detail
        });
    }
}
