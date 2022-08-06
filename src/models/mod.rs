use crate::models::data_model::GraphicsStatus;
use std::path::PathBuf;

use self::{app_model::ImportState, data_model::ExternalData, graphics_model::GraphicsResources};

pub mod app_model;
pub mod data_model;
pub mod graphics_lib;
pub mod graphics_model;

pub struct Models {
    pub compute_model: graphics_model::GraphicsModel,
    pub graphic_model: data_model::DataModel,
    pub app_model: app_model::AppModel,
}

#[derive(Debug)]
pub struct ImportedData {
    pub node_file_path: String,
    pub edge_file_path: String,
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
    pub source_index: usize,
    pub target_index: usize,
    pub source_target_list: Vec<u32>,
    pub max_id: u32,
}

unsafe impl Send for ImportedData {}

impl Models {
    pub fn setup_data(&mut self, data: ImportedData) {
        let ImportedData {
            node_file_path,
            edge_file_path,
            node_data,
            edge_data,
            source_index,
            target_index,
            source_target_list,
            max_id,
        } = data;
        self.graphic_model.node_data = node_data;
        self.graphic_model.edge_data = edge_data;
        self.graphic_model.edge_source = Some(source_index);
        self.graphic_model.edge_target = Some(target_index);
        self.graphic_model.source_target_list = Some(source_target_list);
        self.graphic_model.max_id = max_id;
        self.graphic_model.set_status();
        self.app_model.node_file_path = Some(PathBuf::from(node_file_path));
        self.app_model.edge_file_path = Some(PathBuf::from(edge_file_path));
        self.app_model.import_state = ImportState::Success;
        self.app_model.import_visible = false;
        self.compute_model.graphics_resources = Some(GraphicsResources::new(
            self.compute_model.compute_render_state.clone(),
            &self.graphic_model,
        ));
    }

    pub fn clear_data(&mut self) {
        self.app_model.import_state = ImportState::Initial;
        self.app_model.node_file_path = None;
        self.app_model.edge_file_path = None;
        self.graphic_model.node_data = ExternalData::default();
        self.graphic_model.edge_data = ExternalData::default();
        self.graphic_model.max_id = 0;
        self.compute_model.reset();
        self.compute_model.graphics_resources = None;
        self.graphic_model.status = GraphicsStatus::default();
    }
}
