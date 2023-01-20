use std::borrow::BorrowMut;
use egui::{Ui, Vec2};
use crate::constant::FONT_SIZE_BODY;
use crate::models::app_model::DockStage;

use crate::models::Models;
use crate::utils::message::messenger;
use crate::widgets::frames::dock_frame;

use super::AppView;

#[derive(Default)]
pub struct DockView;

impl AppView for DockView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("dock")
            .frame(dock_frame(ui.style()))
            .default_height(0.0)
            .show_inside(ui, |ui| {
                ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Center), |ui| {

                    ui.set_style(models.app_model.dock_style.clone());
                    ui.spacing_mut().item_spacing = Vec2::ZERO;
                    ui.style_mut().text_styles.get_mut(&egui::TextStyle::Button).unwrap().size = FONT_SIZE_BODY;

                    dock_button(ui, models, DockStage::Messages, format!("🕫 Messages({})", messenger().len()));
                    dock_button(ui, models, DockStage::Timeline, "🕙 Timeline");
                    dock_button(ui, models, DockStage::Kernel, "✱ Kernels");


                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {

                            if let Some(graphics_resources) = &mut models.graphics_model.graphics_resources  {
                                toggle_button(ui, &mut graphics_resources.render_options.is_showing_debug, "ℹ State");
                            } else {
                                toggle_button(ui, &mut false, "ℹ State");
                            }


                        },
                    );

            });
        });
    }
}

fn dock_button(ui: &mut egui::Ui, models: &mut Models, current: DockStage, text: impl Into<egui::WidgetText>) {
    let checked = models.app_model.dock_stage == current;
    if ui.selectable_label(checked, text).clicked() {
        if checked {
            models.app_model.dock_stage = DockStage::None;
        } else {
            models.app_model.dock_stage = current;
        }
    }
}

fn toggle_button(ui: &mut egui::Ui, selected: &mut bool, text: impl Into<egui::WidgetText>) -> egui::Response {

    ui.toggle_value(selected.borrow_mut(), text)
}