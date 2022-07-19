use crate::MainApp;

pub mod boids;

pub trait GraphicDelegation {

  /// Graphics View Render View
  fn custom_painting(&self, ctx: &MainApp, ui: &mut egui::Ui);

}

pub trait GraphicObject {

  /// Initialize wgpu
  fn init<'a>(cc: &'a eframe::CreationContext<'a>);

  /// Compute Dispatch
  fn compute(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

  /// Render Call
  fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>);

}