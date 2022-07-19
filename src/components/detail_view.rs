use egui::Ui;

use crate::{MainApp};

use super::AppView;

pub struct DetailView;

impl Default for DetailView {
    fn default() -> Self {
        Self {  }
    }
}

impl AppView for DetailView {
    fn show(self, _: &mut MainApp, ui: &mut Ui) {
        egui::TopBottomPanel::bottom("detail").show_inside(ui, |ui| {
            ui.set_style(ui.ctx().style());
            let layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
            ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                ui.label("Detail View");
            })
        });
    }
}
