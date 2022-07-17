pub mod boids;

pub trait GraphicObject {

  /// Graphics View 图形视图绘制方法
  fn custom_painting(&mut self, ui: &mut egui::Ui);

}