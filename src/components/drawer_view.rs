use egui::{Response, Ui, Vec2};
use crate::components::drawers::MessageView;
use crate::constant::FONT_SIZE_TITLE;
use crate::models::app_model::DockStage;

use crate::models::Models;
use crate::widgets::frames::{drawer_frame, drawer_title_frame};

use super::AppView;

#[derive(Default)]
pub struct DrawerView;

impl AppView for DrawerView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("drawer")
            .resizable(true)
            .default_height(220.0)
            .height_range(100.0..=350.0)
            .frame(drawer_frame(ui.style()))
            .show_inside(ui, |ui| {

                drawer_title_frame(ui.style()).show(ui, |ui| {

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {

                            let (egui_id, rect) = ui.allocate_space(Vec2::splat(15.0));
                            if close_button(ui, egui_id, rect).clicked() {
                                models.app_model.dock_stage = DockStage::None;
                            };


                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {

                                    let title = match models.app_model.dock_stage {
                                        DockStage::None => "None",
                                        DockStage::Messages => "Messages",
                                        DockStage::Timeline => "Timeline",
                                    };

                                    ui.style_mut().text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = FONT_SIZE_TITLE;
                                    ui.label(egui::RichText::new(format!("{}", title)).strong());

                                },
                            );

                        });

                    });

                });


                ui.add( egui::Separator::default().spacing(0.0) );

                match models.app_model.dock_stage {
                    DockStage::Messages => {
                        MessageView::default().show(models, ui, frame);
                    },
                    _ => {
                        ui.centered_and_justified(|ui| {
                            ui.set_min_height(100.0);
                            ui.label(egui::RichText::new("Drawer View").weak());
                        });
                    }
                }

        });
    }
}


fn close_button(ui: &mut Ui, id: egui::Id, rect: egui::Rect) -> Response {
    let response = ui.interact(rect, id, egui::Sense::click());
    ui.expand_to_include_rect(response.rect);

    let visuals = ui.style().interact(&response);
    let rect = rect.shrink(2.0).expand(visuals.expansion);
    let stroke = visuals.fg_stroke;
    ui.painter() // paints \
        .line_segment([rect.left_top(), rect.right_bottom()], stroke);
    ui.painter() // paints /
        .line_segment([rect.right_top(), rect.left_bottom()], stroke);
    response
}
