use std::f32::consts::{FRAC_PI_2, PI};
use egui::{PointerButton, Pos2, Ui, Vec2};
use crate::models::graphics_lib::Camera;

pub struct OrbitControls {
    pos: Option<Pos2>,
    primary_clicked: bool,
    primary_down: bool,
    scroll_delta: Vec2,
    delta: Vec2,
    viewport_size: Vec2,
}

impl OrbitControls {

    pub fn new() -> Self {
        Self {
            pos: None,
            primary_clicked: false,
            primary_down: false,
            scroll_delta: Vec2::ZERO,
            delta: Vec2::ZERO,
            viewport_size: Vec2::ZERO,
        }
    }

}

impl OrbitControls {

    pub fn update_interaction(&mut self, ui: &mut Ui) -> bool {
        self.pos = None;
        self.primary_clicked = false;
        self.scroll_delta = ui.input().scroll_delta;
        self.delta = ui.input().pointer.delta();

        let viewport_rect = ui.max_rect();
        self.viewport_size = viewport_rect.size();

        let mut is_updated = false;

        if let Some(pos) = ui.input().pointer.interact_pos() {
            if viewport_rect.contains(pos) {

                self.pos = Some(pos - viewport_rect.min.to_vec2());
                if ui.input().pointer.primary_clicked() { self.primary_clicked = true; is_updated = true; }
                if ui.input().pointer.primary_down()    { self.primary_down = true; is_updated = true; }
                if self.scroll_delta != Vec2::ZERO { is_updated = true; }

            }
        }

        if !ui.input().pointer.primary_down() {
            self.primary_down = false;
        }

        return is_updated;
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        if self.pos.is_some() {
            camera.zoom(f32::powf(1.2, -self.scroll_delta.y * 0.1) );
        }
        if self.primary_down {
            let mut angles = glam::Vec2::new(self.delta.x, self.delta.y);
            angles = angles / glam::Vec2::new(self.viewport_size.x, self.viewport_size.y);
            camera.rotate(angles * PI);
        }
    }

}