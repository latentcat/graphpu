use egui::Ui;

use crate::widgets::frames::menu_panel_style;
use crate::{
    models::{
        app_model::{ImportState, MainStage},
        Models,
    },
    widgets::frames::button_group_style,
};

use super::AppView;

pub struct MenuBarView;

impl Default for MenuBarView {
    fn default() -> Self {
        Self {}
    }
}

impl AppView for MenuBarView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar_view")
            .frame(menu_panel_style(
                ui.style(),
                frame.info().window_info.fullscreen,
            ))
            .show_separator_line(false)
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                egui::menu::bar(ui, |ui| {
                    spacing_ui(ui);
                    ui.menu_button("File", |ui| {
                        spacing_ui_start(ui);
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
                                    models.app_model.is_import_visible = true;
                                    ui.close_menu();
                                }
                            }
                            ImportState::Success => {
                                if ui.button("Reimport Data").clicked() {
                                    models.clear_data();
                                    models.app_model.is_import_visible = true;
                                    ui.close_menu();
                                }
                            }
                            _ => {}
                        }

                        ui.add_enabled_ui(
                            models.app_model.import_state == ImportState::Success,
                            |ui| {
                                if ui.button("Export Data").clicked() {
                                    models.app_model.is_export_visible = true;
                                    ui.close_menu();
                                }
                            },
                        );

                        ui.separator();

                        if ui.button("Quit").clicked() {
                            frame.close();
                        }
                        spacing_ui_end(ui);
                    });
                    ui.menu_button("Edit", |ui| {
                        spacing_ui_start(ui);

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Undo");
                            let _ = ui.button("Redo");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Preference");
                        });

                        spacing_ui_end(ui);
                    });
                    ui.menu_button("Render", |ui| {
                        spacing_ui_start(ui);

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Render Image");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Show Result");
                        });
                        spacing_ui_end(ui);
                    });
                    ui.menu_button("Window", |ui| {
                        spacing_ui_start(ui);

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Fullscreen");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Save Screenshot");
                        });
                        spacing_ui_end(ui);
                    });
                    ui.menu_button("Help", |ui| {
                        spacing_ui_start(ui);

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Official Website");
                            let _ = ui.button("Manual");
                            let _ = ui.button("Tutorial");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            let _ = ui.button("Report a Bug");
                        });
                        spacing_ui_end(ui);
                    });

                    ui.add_space(12.0);

                    button_group_style(ui.style()).show(ui, |ui| {
                        ui.set_style(ui.ctx().style());
                        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
                        ui.selectable_value(
                            &mut models.app_model.main_stage,
                            MainStage::Graphics,
                            "  Graphics  ",
                        );
                        ui.selectable_value(
                            &mut models.app_model.main_stage,
                            MainStage::Table,
                            "  Table  ",
                        );
                    });
                });
            });
    }
}

fn spacing_ui(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
    ui.spacing_mut().button_padding = egui::vec2(8.0, 1.0);
}

fn spacing_ui_start(ui: &mut Ui) {
    ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);
    ui.spacing_mut().button_padding = egui::vec2(6.0, 0.0);
    ui.add_space(2.0);
    // ui.add_space(4.0);
}

fn spacing_ui_end(_ui: &mut Ui) {
    // ui.add_space(2.0);
}
