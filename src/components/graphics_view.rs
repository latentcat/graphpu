use egui::Ui;

use crate::models::Models;
use crate::widgets::frames::graphics_frame;

use super::AppView;

#[derive(Default)]
pub struct GraphicsView;

impl AppView for GraphicsView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui) {


        let is_computing = models.compute_model.is_computing;
        let is_dispatching = models.compute_model.is_dispatching;
        models.compute_model.set_dispatching(false);

        if is_computing {
            models.compute_model.compute_resources.compute();
        }

        if is_dispatching {
            models.compute_model.compute_resources.randomize();
        }

        if is_computing || is_dispatching {
            models.compute_model.compute_resources.render();
            ui.ctx().request_repaint();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show_inside(ui, |ui| {
                ui.set_style(ui.ctx().style());
                graphics_frame(ui.style())
                    .show(ui, |ui| {
                        let texture_id = models.compute_model.compute_resources.texture_id;
                        ui.image(texture_id, ui.max_rect().size());
                    });
            });
    }
}
