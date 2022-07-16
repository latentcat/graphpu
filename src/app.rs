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
        egui::TopBottomPanel::top("menubar_view")
            .show(ctx, |ui| {
                egui::menu::bar(ui, |_| {
                    // TODO: Menu Bar
                });
        });

        egui::SidePanel::right("inspector_view")
            .default_width(250.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show(ctx, |ui| {

                egui::ScrollArea::vertical()
                    // .always_show_scroll(true)
                    .auto_shrink([false, false])
                    .id_source("source")
                    .show(ui, |ui| {
                        ui.label("Inspector View");
                        lorem_ipsum(ui);
                    });

            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                MainCanvas::new(&mut self.boids).add(ctx, ui);
            });

        egui::TopBottomPanel::bottom("detail").show(ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
            ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                ui.label("Detail View");
            })
        });
    }
}

fn lorem_ipsum(ui: &mut egui::Ui) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT).with_cross_align(egui::Align::Min),
        |ui| {
            ui.label(egui::RichText::new(crate::LOREM_IPSUM_LONG).weak());
        },
    );
}