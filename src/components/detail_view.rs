use egui::Ui;

use crate::MainApp;

use super::AppView;

#[derive(Default)]
pub struct DetailView;

impl AppView for DetailView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("detail").show_inside(ui, |ui| {

            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Messages").weak());
                ui.allocate_ui_with_layout(
                    ui.available_size(),
                    egui::Layout::right_to_left(),
                    |ui| {
                        ui.label(
                            egui::RichText::new(
                                format!(
                                    "Nodes: {}  |  Edges: {}",
                                    ctx.graphic_model.node_data.data.len(),
                                    ctx.graphic_model.edge_data.data.len()
                                )
                            ).weak()
                        );
                    },
                );
            });
        });
    }
}
