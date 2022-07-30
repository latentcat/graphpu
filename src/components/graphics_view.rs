use std::ops::Mul;
use egui::{Ui, Vec2};

use crate::models::Models;
use crate::widgets::frames::graphics_frame;

use super::AppView;

#[derive(Default)]
pub struct GraphicsView;

impl AppView for GraphicsView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, frame: &mut eframe::Frame) {
        let is_computing = models.compute_model.is_computing;
        let is_dispatching = models.compute_model.is_dispatching;
        models.compute_model.set_dispatching(false);
        
        if let Some(compute_resources) = &mut models.compute_model.compute_resources {
            if is_computing {
                compute_resources.compute();
            }
    
            if is_dispatching {
                compute_resources.randomize();
            }
    
            egui::CentralPanel::default()
                .frame(egui::Frame::none())
                .show_inside(ui, |ui| {
                    ui.set_style(ui.ctx().style());
                    graphics_frame(ui.style())
                        .show(ui, |ui| {
    
                            compute_resources.update_viewport(
                                ui.max_rect().size().mul(
                                    Vec2::from([models.app_model.pixels_per_point, models.app_model.pixels_per_point])
                                )
                            );
    
                            let is_viewport_update = compute_resources.is_viewport_update;
                            compute_resources.is_viewport_update = false;
                            if is_computing || is_dispatching || is_viewport_update {
                                compute_resources.render();
                                ui.ctx().request_repaint();
                            }
                            let texture_id = compute_resources.texture_id;
                            ui.image(texture_id, ui.max_rect().size());
                        });
                });
        }
    }
}
