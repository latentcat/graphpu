use crate::context::AppContext;

use super::AppView;

pub struct InspectorView;

impl Default for InspectorView {
    fn default() -> Self {
        Self {}
    }
}

impl AppView for InspectorView {
    fn show(self, ctx: &mut AppContext) {
        egui::SidePanel::right("inspector_view")
            .default_width(250.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show(ctx.egui_ctx, |ui| {
                egui::ScrollArea::vertical()
                    // .always_show_scroll(true)
                    .auto_shrink([false, false])
                    .id_source("source")
                    .show(ui, |ui| {
                        ui.label("Inspector View");
                        lorem_ipsum(ui);
                    });
            });
    }
}

fn lorem_ipsum(ui: &mut egui::Ui) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT).with_cross_align(egui::Align::Min),
        |ui| {
            ui.label(egui::RichText::new(crate::LOREM_IPSUM_LONG).weak());
        },
    );
}
