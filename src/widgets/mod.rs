use crate::models::Models;

pub mod boids;
pub mod modal;
pub mod frames;

pub trait GraphicDelegation {

  /// Graphics View Render View
  fn custom_painting(&self, models: &mut Models, ui: &mut egui::Ui);

}

pub trait GraphicObject {

  /// Initialize wgpu
  fn init<'a>(cc: &'a eframe::CreationContext<'a>);

  /// Compute Dispatch
  fn compute(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

  /// Randomize Dispatch
  fn randomize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

  /// Render Call
  fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>);

}