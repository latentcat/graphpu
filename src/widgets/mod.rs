pub mod boids;

pub trait GraphicObject {
  fn custom_painting(&mut self, ui: &mut egui::Ui);
}