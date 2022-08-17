use egui::{Color32, epaint, Ui, Vec2};
use crate::components::AppView;

use crate::models::Models;
use crate::utils::message::{MessageLevel, messenger};
use crate::widgets::frames::{drawer_message_content_frame};


#[derive(Default)]
pub struct MessageView;

impl AppView for MessageView {
    fn show(&mut self, _models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        drawer_message_content_frame(ui.style()).show(ui, |ui| {

            let messages = messenger();
            let row_height = 52.0;
            let num_rows = messages.len();


            ui.spacing_mut().interact_size = Vec2::new(4.0, 4.0);

            egui::ScrollArea::vertical().stick_to_bottom().auto_shrink([false; 2]).show_rows(
                ui,
                row_height,
                num_rows,
                |ui, row_range| {
                    for row in row_range {
                        let message = &messages[row];
                        let title_text = egui::RichText::new(&message.title);
                        let time_text = egui::RichText::new(message.display_time()).weak();
                        let content_text = &message.content;
                        ui.vertical(|ui| {

                            ui.set_height(row_height);

                            if row != 0 {
                                ui.add( egui::Separator::default().spacing(0.0) );
                            }

                            ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Min), |ui| {

                                message_icon(ui, &message.level);

                                ui.vertical(|ui| {

                                    ui.add_space(5.0);

                                    ui.horizontal(|ui| {
                                        ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                            ui.add_space(6.0);
                                            ui.label(time_text);
                                            ui.allocate_ui_with_layout(ui.available_size(), egui::Layout::left_to_right(), |ui| {
                                                ui.label(title_text);
                                            });
                                        })
                                    });

                                    let mut job = egui::text::LayoutJob::single_section(content_text.to_owned(), egui::TextFormat {
                                        font_id: egui::FontId::new(13.0, Default::default()),
                                        color: egui::Color32::from_gray(120),
                                        ..Default::default()
                                    });
                                    job.wrap = epaint::text::TextWrapping {
                                        max_rows: 2,
                                        break_anywhere: false,
                                        overflow_character: Some('…'),
                                        ..Default::default()
                                    };
                                    ui.label(job);
                                    // ui.label(content_text);
                                });
                            });
                        });
                    }
                },
            );
        });
    }
}

fn message_icon(ui: &mut egui::Ui, icon_type: &MessageLevel) {
    let (label, color) = match icon_type {
        MessageLevel::Info => ("ℹ", Color32::GRAY),
        MessageLevel::Warning => ("⚠", Color32::YELLOW),
        MessageLevel::Error => ("！", Color32::RED),
    };
    let job = egui::text::LayoutJob::single_section(label.to_owned(), egui::TextFormat {
        font_id: egui::FontId::new(24.0, Default::default()),
        color: color,
        ..Default::default()
    });
    ui.vertical(|ui| {
        ui.set_width(30.);
        ui.add_space(5.0);
        ui.add(
            egui::Label::new(job)
        )
    });
}