pub mod compute;
pub mod graphics;
pub mod app;

pub struct Models {
  pub compute_model: compute::ComputeModel,
  pub graphic_model: graphics::GraphicsModel,
  pub app_model: app::AppModel,
}