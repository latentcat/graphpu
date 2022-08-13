use egui::Ui;

use crate::models::Models;

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
                            ui.label(egui::RichText::new(format!("Message: {}", &models.app_model.message)).weak());
                        },
                    );

                });

            });
        });
    }
}
