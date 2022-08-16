use egui::{Sense, Ui};
use crate::models::app_model::DockStage;

use crate::models::Models;
use crate::utils::message::messenger;

use super::AppView;

#[derive(Default)]
pub struct DetailView;

impl AppView for DetailView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("detail").show_inside(ui, |ui| {

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(), |ui| {
                    ui.label(
                        egui::RichText::new(
                            format!(
                                "Nodes: {}  |  Edges: {}",
                                models.data_model.status.node_count,
                                models.data_model.status.edge_count
                            )
                        ).weak()
                    );

                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::left_to_right(),
                        |ui| {
                            let messages = messenger();
                            if messages.len() > 0 {
                                let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
                                ui.allocate_ui_at_rect(rect, |ui| {
                                    ui.label(egui::RichText::new(format!("{}", messages[0])).weak())
                                });
                                response.clicked().then(||{ models.app_model.dock_stage = DockStage::Messages });
                            }
                        },
                    );

                });

            });
        });
    }
}
