use crate::{models::graphics::GraphicsStatus, utils::csv_loader::read_from_csv};
use std::path::PathBuf;

use self::{app::ImportState, compute::ComputeResources, graphics::ExternalData};

pub mod app;
pub mod compute;
pub mod graphics;

pub struct Models {
    pub compute_model: compute::ComputeModel,
    pub graphic_model: graphics::GraphicsModel,
    pub app_model: app::AppModel,
}

impl Models {
    pub fn load_data(
        &mut self,
        node_file_path: &str,
        edge_file_path: &str,
        edge_source: usize,
        edge_target: usize,
    ) -> Result<(), String> {
        self.graphic_model.node_data =
            read_from_csv(&Some(PathBuf::from(node_file_path))).unwrap_or(ExternalData::default());
        self.graphic_model.edge_data = read_from_csv(&Some(PathBuf::from(edge_file_path)))?;

        let source_key = &self.graphic_model.edge_data.data_headers[edge_source];
        let target_key = &self.graphic_model.edge_data.data_headers[edge_target];
        let err_mapper = |_| String::from("Source and target isn't uint");
        self.graphic_model.max_id = *self
            .graphic_model
            .edge_data
            .data
            .iter()
            .map::<Result<usize, String>, _>(|item| {
                let source = item
                    .get(source_key)
                    .unwrap()
                    .parse::<usize>()
                    .map_err(err_mapper)?;
                let target = item
                    .get(target_key)
                    .unwrap()
                    .parse::<usize>()
                    .map_err(err_mapper)?;
                Ok(std::cmp::max(source, target))
            })
            .collect::<Result<Vec<_>, _>>()?
            .iter()
            .max()
            .unwrap();
        self.complete_import_data(node_file_path.to_string(), edge_file_path.to_string());
        Ok(())
    }

    pub fn clear_data(&mut self) {
        self.app_model.import_state = ImportState::Initial;
        self.app_model.node_file_path = None;
        self.app_model.edge_file_path = None;
        self.graphic_model.node_data = ExternalData::default();
        self.graphic_model.edge_data = ExternalData::default();
        self.graphic_model.max_id = 0;
        self.compute_model.reset();
        self.compute_model.compute_resources = None;
        self.graphic_model.status = GraphicsStatus::default();
    }
}

impl Models {
    fn complete_import_data(&mut self, node_file_path: String, edge_file_path: String) {
        self.graphic_model.set_status();
        self.app_model.node_file_path = Some(PathBuf::from(node_file_path));
        self.app_model.edge_file_path = Some(PathBuf::from(edge_file_path));
        self.app_model.import_state = ImportState::Success;
        self.app_model.import_visible = false;

        if let Some(render_state) = &self.compute_model.compute_render_state {
            self.compute_model.compute_resources = Some(ComputeResources::new(
                render_state.clone(),
                self.graphic_model.status.clone(),
            ));
        }
    }
}
