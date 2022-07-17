pub mod boids;

pub trait GraphicDelegation {

  /// Graphics View 图形视图绘制方法
  fn custom_painting(&mut self, ui: &mut egui::Ui);

}

pub trait GraphicObject {
  
  fn init<'a>(cc: &'a eframe::CreationContext<'a>);

  fn compute(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

  fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>);
}