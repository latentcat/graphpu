use egui::Ui;

use crate::{models::compute::ComputeMethod, MainApp};
use crate::models::compute::ComputeMethodType;

use super::AppView;

pub struct InspectorView;

impl Default for InspectorView {
    fn default() -> Self {
        Self {}
    }
}

pub fn panel_style(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 8.0),
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

impl AppView for InspectorView {
    fn show(self, ctx: &mut MainApp, ui: &mut Ui) {
        let MainApp { compute_model: model, .. } = ctx;
        egui::SidePanel::right("inspector_view")
            .frame(panel_style(ui.style()))
            .default_width(250.0)
            .width_range(150.0..=400.0)
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                egui::ComboBox::from_label("Compute Method")
                    .selected_text(model.compute_method.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut model.compute_method, ComputeMethod::FORCE_ATLAS2, ComputeMethod::FORCE_ATLAS2.0);
                        ui.separator();
                        ui.selectable_value(&mut model.compute_method, ComputeMethod::RANDOMIZE, ComputeMethod::RANDOMIZE.0);
                    });
                if model.compute_method.1 == ComputeMethodType::Continuous {
                    let continuous_button = ui.button(if !model.is_computing { "Start Computing" } else { "Pause Computing" });
                    if continuous_button.clicked() {
                        model.switch_computing();
                    }
                } else {
                    model.set_computing(false);
                    let one_step_button = ui.button("Dispatch");
                    if one_step_button.clicked() {
                        model.set_dispatching(true);
                    }
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