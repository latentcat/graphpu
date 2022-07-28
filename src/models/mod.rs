use self::{app::ImportState, graphics::ExternalData};

pub mod compute;
pub mod graphics;
pub mod app;

pub struct Models {
  pub compute_model: compute::ComputeModel,
  pub graphic_model: graphics::GraphicsModel,
  pub app_model: app::AppModel,
}

impl Models {
  pub fn clear_data(&mut self) {
    self.app_model.import_state = ImportState::Initial;
    self.app_model.node_file_path = None;
    self.app_model.edge_file_path = None;
    self.graphic_model.node_data = ExternalData::default();
    self.graphic_model.edge_data = ExternalData::default();
    self.graphic_model.max_id = 0;
  }
}