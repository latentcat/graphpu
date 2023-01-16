use egui::{Sense, Ui};
use crate::constant::FONT_SIZE_BODY;
use crate::models::app_model::DockStage;

use crate::models::Models;
use crate::utils::message::messenger;

use super::AppView;

#[derive(Default)]
pub struct DetailView;

impl AppView for DetailView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("detail")
            .show_inside(ui, |ui| {

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(
                            format!(
                                "Nodes: {}  |  Edges: {}",
                                models.data_model.status.node_count,
                                models.data_model.status.edge_count
                            )
                        ).weak()
                    );

                    ui.add_space(30.0);

                    ui.allocate_ui_with_layout(
                        ui.available_size(),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            let messages = messenger();
                            if messages.len() > 0 {
                                let message = &messages[messages.len() - 1].to_string();
                                let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click());
                                ui.allocate_ui_at_rect(rect, |ui| {
                                    ui.vertical(|ui| {
                                        ui.add_space(3.0);

                                        let mut job = egui::text::LayoutJob::single_section(message.to_owned(), egui::TextFormat {
                                            font_id: egui::FontId::new(FONT_SIZE_BODY, Default::default()),
                                            color: egui::Color32::from_gray(120),
                                            ..Default::default()
                                        });
                                        job.wrap = egui::epaint::text::TextWrapping {
                                            max_rows: 1,
                                            break_anywhere: true,
                                            overflow_character: Some('…'),
                                            ..Default::default()
                                        };
                                        ui.label(job);
                                    });
                                    // ui.label(egui::RichText::new(format!("{}", messages[0])).weak())
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
