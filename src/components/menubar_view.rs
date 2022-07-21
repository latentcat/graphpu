use egui::Ui;

use crate::MainApp;

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
    fn show(self, _: &mut MainApp, ui: &mut Ui) {
        egui::TopBottomPanel::top("menubar_view")
            .frame(panel_style(ui.style()))
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("New");
                            ui.button("Open");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Close");
                            ui.button("Save");
                        });

                        ui.separator();

                        ui.button("Import");
                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Export");
                        });

                        ui.separator();

                        if ui.button("Quit").clicked() {
                            // frame.quit();
                        }
                    });
                    ui.menu_button("Edit", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Undo");
                            ui.button("Redo");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Preference");
                        });

                    });
                    ui.menu_button("Render", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Render Image");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Show Result");
                        });

                    });
                    ui.menu_button("Window", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Toggle Window Fullscreen");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Save Screenshot");
                        });

                    });
                    ui.menu_button("Help", |ui| {

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Official Website");
                            ui.button("Manual");
                            ui.button("Tutorial");
                        });

                        ui.separator();

                        ui.add_enabled_ui(false, |ui| {
                            ui.button("Report a Bug");
                        });

                    });
                });
            });
    }
}
