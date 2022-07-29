use egui::Ui;

use crate::{models::{app::{Stage, ImportState}, Models}, widgets::frames::button_group_style};

use super::AppView;

pub struct MenuBarView;

impl Default for MenuBarView {
    fn default() -> Self {
        Self {  }
    }
}

pub fn panel_style(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin {
            left: 8.0,
            right: 8.0,
            top: 3.0,
            bottom: 1.0
        },
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

impl AppView for MenuBarView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar_view")
            .frame(panel_style(ui.style()))
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("New");
                            let _ = ui.button("Open");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Close");
                            let _ = ui.button("Save");
                        });

                        ui.separator();

                        match models.app_model.import_state {
                            ImportState::Initial => {
                                if ui.button("Import Data").clicked() {
                                    models.app_model.import_visible = true;
                                    ui.close_menu();
                                }
                            },
                            ImportState::Success => {
                                if ui.button("Reimport Data").clicked() {
                                    models.clear_data();
                                    models.app_model.import_visible = true;
                                    ui.close_menu();
                                }
                            },
                            _ => {},
                        }

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Export Data");
                        });

                        ui.separator();

                        if ui.button("Quit").clicked() {
                            // frame.quit();
                        }
                    });
                    ui.menu_button("Edit", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Undo");
                            let _ = ui.button("Redo");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Preference");
                        });

                    });
                    ui.menu_button("Render", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Render Image");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Show Result");
                        });

                    });
                    ui.menu_button("Window", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Fullscreen");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Save Screenshot");
                        });

                    });
                    ui.menu_button("Help", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Official Website");
                            let _ = ui.button("Manual");
                            let _ = ui.button("Tutorial");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Report a Bug");
                        });

                    });

                    ui.add_space(12.0);

                    button_group_style(ui.style())
                        .show(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                            ui.selectable_value(&mut models.app_model.stage, Stage::Graphics, "    Graphics    ");
                            ui.selectable_value(&mut models.app_model.stage, Stage::Table, "    Table    ");
                        });
                });
            });
    }
}
