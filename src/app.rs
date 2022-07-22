use egui::{Color32, Pos2, Rect, Vec2};
use crate::{
    components::{
        detail_view::DetailView, graphics_view::GraphicsView, inspector_view::InspectorView,
        menubar_view::MenuBarView, AppView,
    },
    models::{compute::ComputeModel, graphics::GraphicsModel},
    widgets::boids::Boids,
};

pub struct MainApp {
    pub compute_model: ComputeModel,
    pub graphic_model: GraphicsModel,
}

pub fn window_cover_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::none(),
        fill: Color32::from_black_alpha(180),
        ..Default::default()
    }
}

pub fn window_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::same(8.0),
        fill: style.visuals.window_fill(),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

pub fn inner_panel_style(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(16.0, 16.0),
        rounding: egui::Rounding::none(),
        ..Default::default()
    }
}

impl MainApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals.widgets.inactive.fg_stroke.color = Color32::from_white_alpha(190);
        style.visuals.widgets.noninteractive.fg_stroke.color = Color32::from_white_alpha(170);
        cc.egui_ctx.set_style(style);

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
                DetailView::default().show(self, ui);
                InspectorView::default().show(self, ui);
                GraphicsView::default().show(self, ui);
            });
        egui::Window::new("My Window")
            .anchor(egui::Align2::CENTER_CENTER, Vec2::new(0., 0.))
            .frame(window_frame(&ctx.style()))
            .resizable(false)
            .title_bar(false)
            .show(ctx, |ui| {
                ui.set_width(400.);
                ui.set_height(250.);

                egui::TopBottomPanel::bottom("v")
                    .frame(inner_panel_style(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
                        ui.horizontal(|ui| {
                            ui.label("");
                            ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::right_to_left(), |ui| {
                                let remove_data_button = ui.button("   Import   ");
                                if remove_data_button.clicked() {
                                    //
                                }
                                let reimport_data_button = ui.button("   Cancel   ");
                                if reimport_data_button.clicked() {
                                    //
                                }
                            });
                        });
                    });

                egui::CentralPanel::default()
                    .frame(inner_panel_style(ui.style()))
                    .show_inside(ui, |ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);


                        ui.heading("Import Data");


                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut 0, 0, "CSV");
                            ui.selectable_value(&mut 0, 1, "GraphML");
                            ui.selectable_value(&mut 0, 2, "DOT");
                        });


                        ui.separator();


                        let mut text = String::from("");

                        egui::Grid::new("my_grid")
                            .num_columns(2)
                            .spacing([20.0, 8.0])
                            .show(ui, |ui| {

                                ui.add(egui::Label::new("Node File"));
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut text).hint_text("").desired_width(200.));
                                    ui.button("•••");
                                });

                                ui.end_row();

                                ui.add(egui::Label::new("Edge File"));
                                ui.horizontal(|ui| {
                                    ui.add(egui::TextEdit::singleline(&mut text).hint_text("").desired_width(200.));
                                    ui.button("•••");
                                });

                                ui.end_row();
                            });

                    });

            });
        egui::Area::new("my_area")
            .anchor(egui::Align2::LEFT_TOP, Vec2::new(0., 0.))
            .order(egui::Order::Background)
            .show(ctx, |ui| {
                egui::CentralPanel::default()
                    .frame(window_cover_frame(ui.style()))
                    .show_inside(ui, |ui| {
                        // ui.label("Cover Test");
                    });
            });
    }
}
