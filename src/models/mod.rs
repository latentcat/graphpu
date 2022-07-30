use std::path::PathBuf;

use self::{app::ImportState, graphics::ExternalData, compute::ComputeResources};

pub mod compute;
pub mod graphics;
pub mod app;

pub struct Models {
  pub compute_model: compute::ComputeModel,
  pub graphic_model: graphics::GraphicsModel,
  pub app_model: app::AppModel,
}

impl Models {
  pub fn import_data(&mut self, node_file_path: String, edge_file_path: String) {
    self.graphic_model.set_status();
    self.app_model.node_file_path = Option::Some(PathBuf::from(node_file_path));
    self.app_model.edge_file_path = Option::Some(PathBuf::from(edge_file_path));
    self.app_model.import_state = ImportState::Success;
    self.app_model.import_visible = false;

    if let Some(render_state) = &self.compute_model.compute_render_state {
      self.compute_model.compute_resources = Some(ComputeResources::new(render_state.clone()));
    }
  }

  pub fn clear_data(&mut self) {
    self.app_model.import_state = ImportState::Initial;
    self.app_model.node_file_path = None;
    self.app_model.edge_file_path = None;
    self.graphic_model.node_data = ExternalData::default();
    self.graphic_model.edge_data = ExternalData::default();
    self.graphic_model.max_id = 0;
    self.compute_model.compute_resources = None;
  }
}