use egui::Ui;

use crate::models::Models;

use super::AppView;

#[derive(Default)]
pub struct DetailView;

impl AppView for DetailView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("detail").show_inside(ui, |ui| {

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&models.app_model.message).weak());
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::right_to_left(),
                    |ui| {
                        ui.label(
                            egui::RichText::new(
                                format!(
                                    "Nodes: {}  |  Edges: {}",
                                    models.graphic_model.status.node_count,
                                    models.graphic_model.status.edge_count
                                )
                            ).weak()
                        );
                    },
                );
            });
        });
    }
}
