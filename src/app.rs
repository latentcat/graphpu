use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView, import_modal::ImportModal,
    },
    models::{app::AppModel, compute::ComputeModel, graphics::GraphicsModel},
    widgets::boids::Boids,
};
use egui::Color32;

pub struct MainApp {
    pub compute_model: ComputeModel,
    pub graphic_model: GraphicsModel,
    pub app_model: AppModel,
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.widgets.active.fg_stroke.color = Color32::from_white_alpha(220);
        style.visuals.widgets.inactive.fg_stroke.color = Color32::from_white_alpha(190);
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_white_alpha(170);
        cc.egui_ctx.set_style(style);

        Self {
            compute_model: ComputeModel::default(),
            graphic_model: GraphicsModel::new(std::rc::Rc::new(Boids::new(cc))),
            app_model: AppModel::default(),
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
                DetailView::default().show(self, ui);
                InspectorView::default().show(self, ui);
                GraphicsView::default().show(self, ui);
            });

        if self.app_model.import_visible {
            ImportModal::show(ctx, self);
        }
    }
}
