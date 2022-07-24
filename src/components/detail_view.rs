use egui::Ui;

use crate::MainApp;

use super::AppView;

#[derive(Default)]
pub struct DetailView;

impl AppView for DetailView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("detail").show_inside(ui, |ui| {
            ui.set_style(ui.ctx().style());
            let layout = egui::Layout::top_down(egui::Align::LEFT).with_main_justify(true);
            ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {


                ui.label(
                    egui::RichText::new(
                        format!(
                            "Nodes: {}  |  Edges: {}",
                            ctx.graphic_model.node_data.data.len(),
                            ctx.graphic_model.edge_data.data.len()
                        )
                    ).weak()
                );


            })
        });
    }
}
