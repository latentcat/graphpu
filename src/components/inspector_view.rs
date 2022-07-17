use crate::{context::AppContext, models::inspector::Enum};

use super::AppView;

pub struct InspectorView;

impl Default for InspectorView {
    fn default() -> Self {
        Self {}
    }
}

impl AppView for InspectorView {
    fn show(self, ctx: &mut AppContext) {
        let AppContext {
            egui_ctx,
            app,
        } = ctx;
        egui::SidePanel::right("inspector_view")
            .default_width(250.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show(egui_ctx, |ui| {
                app.inspector_model.radio_arr.iter_mut().enumerate().for_each(|(index, radio)| {
                    egui::ComboBox::from_label(format!("Take your pick {}", index))
                        .selected_text(format!("{:?}", radio))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(radio, Enum::First, "First");
                            ui.selectable_value(radio, Enum::Second, "Second");
                            ui.selectable_value(radio, Enum::Third, "Third");
                        });
                });
                if ui.button("Reset").clicked() {
                    app.inspector_model.reset();
                }
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
