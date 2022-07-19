use egui::Ui;

use crate::MainApp;

use super::AppView;

pub struct MenuBarView;

impl Default for MenuBarView {
    fn default() -> Self {
        Self {  }
    }
}

impl AppView for MenuBarView {
    fn show(self, _: &mut MainApp, ui: &mut Ui) {
        egui::TopBottomPanel::top("menubar_view").show_inside(ui, |ui| {
            egui::menu::bar(ui, |_| {
                // TODO: Menu Bar
            });
        });
    }
}
