
use egui::{epaint, Response, Ui, Vec2};
use crate::models::app_model::DockStage;

use crate::models::Models;
use crate::utils::message::{MessageLevel, messenger};
use crate::widgets::frames::{drawer_message_content_frame, drawer_frame, drawer_title_frame};

use super::AppView;

#[derive(Default)]
pub struct DrawerView;

impl AppView for DrawerView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::bottom("drawer")
            .resizable(true)
            .default_height(200.0)
            .height_range(100.0..=350.0)
            .frame(drawer_frame(ui.style()))
            .show_inside(ui, |ui| {

                drawer_title_frame(ui.style()).show(ui, |ui| {

                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(), |ui| {

                            let (egui_id, rect) = ui.allocate_space(Vec2::splat(15.0));
                            if close_button(ui, egui_id, rect).clicked() {
                                models.app_model.dock_stage = DockStage::None;
                            };


                            ui.allocate_ui_with_layout(
                                ui.available_size(),
                                egui::Layout::left_to_right(),
                                |ui| {

                                    let title = match models.app_model.dock_stage {
                                        DockStage::None => "None",
                                        DockStage::Messages => "Messages",
                                        DockStage::Timeline => "Timeline",
                                    };

                                    ui.style_mut().text_styles.get_mut(&egui::TextStyle::Body).unwrap().size = 14.0;
                                    ui.label(egui::RichText::new(format!("{}", title)).strong());

                                },
                            );

                        });

                    });

                });


                ui.add( egui::Separator::default().spacing(0.0) );

                match models.app_model.dock_stage {
                    DockStage::Messages => {
                        drawer_message_content_frame(ui.style()).show(ui, |ui| {

                            let messages = messenger();
                            let row_height = 50.0;
                            let num_rows = messages.len();
                            egui::ScrollArea::vertical().stick_to_bottom().auto_shrink([false; 2]).show_rows(
                                ui,
                                row_height,
                                num_rows,
                                |ui, row_range| {
                                    for row in row_range {
                                        let message = &messages[row];
                                        let title_text = egui::RichText::new(&message.title);
                                        let content_text = &message.content;
                                        ui.vertical(|ui| {

                                            ui.set_height(row_height);

                                            if row != 0 {
                                                ui.add( egui::Separator::default().spacing(0.0) );
                                            }

                                            ui.add_space(5.0);

                                            ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::LeftToRight, egui::Align::Min), |ui| {

                                                message_icon(ui, &message.level);

                                                ui.vertical(|ui| {
                                                    ui.label(title_text);

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

fn message_icon(ui: &mut egui::Ui, icon_type: &MessageLevel) {
    let label = format!("[{}]", icon_type.to_string()).to_uppercase();
    ui.vertical(|ui| {
        ui.set_width(65.);
        ui.add(
            egui::Label::new(egui::RichText::new(label)).wrap(true)
        )
    });
}