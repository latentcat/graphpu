use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView, import_modal::ImportModal, table_view::TableView,
    },
    models::{app::{AppModel, Stage}, compute::ComputeModel, graphics::GraphicsModel, Models},
    widgets::boids::Boids,
};
use egui::Color32;

pub struct MainApp {
    pub models: Models,
    inspector_view: InspectorView,
    import_modal: ImportModal,
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.widgets.active.fg_stroke.color = Color32::from_white_alpha(220);
        style.visuals.widgets.inactive.fg_stroke.color = Color32::from_white_alpha(190);
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_white_alpha(170);
        cc.egui_ctx.set_style(style);

        Self {
            models: Models { 
                compute_model: ComputeModel::default(),
                graphic_model: GraphicsModel::new(std::rc::Rc::new(Boids::new(cc))),
                app_model: AppModel::default(),
            },
            inspector_view: InspectorView::default(),
            import_modal: ImportModal::default(),
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
                MenuBarView::default().show(&mut self.models, ui);
                DetailView::default().show(&mut self.models, ui);
                self.inspector_view.show(&mut self.models, ui);
                match self.models.app_model.stage {
                    Stage::Graphics => GraphicsView::default().show(&mut self.models, ui),
                    Stage::Table => TableView::default().show(&mut self.models, ui),
                };
            });

        if self.models.app_model.import_visible {
            self.import_modal.show(ctx, &mut self.models);
        }
    }
}
