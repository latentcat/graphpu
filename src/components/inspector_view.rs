use egui::Ui;

use crate::{models::compute::ComputeMethod, MainApp};

use super::AppView;

pub struct InspectorView;

impl Default for InspectorView {
    fn default() -> Self {
        Self {}
    }
}

impl AppView for InspectorView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        let MainApp { compute_model: model, .. } = ctx;
        egui::SidePanel::right("inspector_view")
            .default_width(250.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                egui::ComboBox::from_label("Compute Method")
                    .selected_text(format!("{:?}", model.compute_method))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut model.compute_method, ComputeMethod::ForceAtlas2, "Force Atlas 2");
                        ui.selectable_value(&mut model.compute_method, ComputeMethod::Randomize, "Randomize");
                    });
                let reset_button = ui.button(if !model.is_computing { "Start Computing" } else { "Pause Computing" });
                if reset_button.clicked() {
                    model.switch_computing();
                }
                egui::ScrollArea::vertical()
                    // .always_show_scroll(true)
                    .auto_shrink([false, false])
                    .id_source("source")
                    .show(ui, |_ui| {
                        // ui.label("Inspector View");
                        // lorem_ipsum(ui);
                    });
            });
    }
}