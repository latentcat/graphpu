use crate::context::AppContext;

use super::AppView;

pub struct DetailView;

impl Default for DetailView {
    fn default() -> Self {
        Self {  }
    }
}

impl AppView for DetailView {
    fn show(self, ctx: &mut AppContext) {
        egui::TopBottomPanel::bottom("detail").show(ctx.egui_ctx, |ui| {
            let layout = egui::Layout::top_down(egui::Align::Center).with_main_justify(true);
            ui.allocate_ui_with_layout(ui.available_size(), layout, |ui| {
                ui.label("Detail View");
            })
        });
    }
}
