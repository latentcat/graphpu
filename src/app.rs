use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView, import_modal_view::ImportModal, table_view::TableView,
    },
    models::{app_model::{AppModel, MainStage}, graphics_model::GraphicsModel, data_model::DataModel, Models},
};
use egui::{Color32, TextStyle};
use crate::components::dock_view::DockView;
use crate::components::drawer_view::DrawerView;
use crate::models::app_model::DockStage;

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
        style.visuals.selection.bg_fill = Color32::from_rgb(86, 89, 225);
        style.visuals.selection.stroke.color = Color32::from_white_alpha(240);

        style.spacing.icon_width = 12.0;
        style.spacing.indent = 16.0;

        style.text_styles.get_mut(&TextStyle::Body).unwrap().size = 13.0;
        style.text_styles.get_mut(&TextStyle::Button).unwrap().size = 13.0;

        style.spacing.button_padding = egui::vec2(8.0, 1.0);

        cc.egui_ctx.set_style(style);

        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert("prop_font".to_owned(),
                               egui::FontData::from_static(include_bytes!("./assets/fonts/droidsans.ttf")));

        fonts.font_data.insert("mono_font".to_owned(),
                               egui::FontData::from_static(include_bytes!("./assets/fonts/bmonofont-i18n.ttf"))); // .ttf and .otf supported

        fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap()
            .insert(0, "prop_font".to_owned());

        fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap()
            .insert(0, "mono_font".to_owned());
            // .push("mono_font".to_owned());

        cc.egui_ctx.set_fonts(fonts);

        // cc.egui_ctx.set_debug_on_hover(true);

        let mut main_app = MainApp {
            models: Models { 
                graphics_model: GraphicsModel::init(cc),
                data_model: DataModel::default(),
                app_model: AppModel::default(),
            },
            inspector_view: InspectorView::default(),
            import_modal: ImportModal::default(),
        };

        if let Some(pixels_per_point) = cc.integration_info.native_pixels_per_point {
            main_app.models.app_model.pixels_per_point = pixels_per_point;
        }

        main_app
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {

        self.models.app_model.ui_frame_count += 1u32;

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                // ui.set_enabled(false);
                MenuBarView::default().show(&mut self.models, ui, frame);
                DetailView::default().show(&mut self.models, ui, frame);
                self.inspector_view.show(&mut self.models, ui, frame);
                DockView::default().show(&mut self.models, ui, frame);
                if self.models.app_model.dock_stage != DockStage::None {
                    DrawerView::default().show(&mut self.models, ui, frame);
                }
                match self.models.app_model.main_stage {
                    MainStage::Graphics => GraphicsView::default().show(&mut self.models, ui, frame),
                    MainStage::Table => TableView::default().show(&mut self.models, ui, frame),
                };
            });

        if self.models.app_model.is_import_visible {
            self.import_modal.show(ctx, &mut self.models);
        }
    }
}
