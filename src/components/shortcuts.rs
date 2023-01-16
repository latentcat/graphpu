use egui::{Modifiers, Ui};
use crate::models::graphics_model::ComputeMethodType;
use crate::models::Models;

#[derive(Default)]
pub struct Shortcut;

impl Shortcut {
    pub fn apply(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let organize_shortcut =
            egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::R);
        let organize_shortcut2 =
            egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::F2);

        let mut should_switch_compute = false;
        if ui.input_mut().consume_shortcut(&organize_shortcut) { should_switch_compute = true }
        if ui.input_mut().consume_shortcut(&organize_shortcut2) { should_switch_compute = true }

        if should_switch_compute {
            let node_settings = &mut models.data_model.node_settings;
            if node_settings.position_compute.1 == ComputeMethodType::Continuous {
                models.graphics_model.switch_computing();
            } else {
                models.graphics_model.set_dispatching(true);
            }
        }

        let organize_shortcut =
            egui::KeyboardShortcut::new(Modifiers::COMMAND, egui::Key::S);

        if ui.input_mut().consume_shortcut(&organize_shortcut) {
            models.render_output();
        }
    }
}