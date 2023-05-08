use crate::models::data_model::GraphicsStatus;
use crate::utils::file::{path_to_string, pick_folder};
use crate::utils::message::message_info;
use std::path::PathBuf;

use self::{app_model::ImportState, data_model::ExternalData};

pub mod app_model;
pub mod data_model;
pub mod exhibision_model;
pub mod graphics_lib;
pub mod graphics_model;

pub struct Models {
    pub graphics_model: graphics_model::GraphicsModel,
    pub data_model: data_model::DataModel,
    pub app_model: app_model::AppModel,
}

#[derive(Debug)]
pub struct ImportedData {
    pub node_file_path: PathBuf,
    pub edge_file_path: PathBuf,
    pub file_name: String,
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
    pub source_index: usize,
    pub target_index: usize,
    pub source_target_list: Vec<u32>,
    pub max_id: u32,
}

unsafe impl Send for ImportedData {}

impl Models {
    pub fn setup_data(&mut self, data: &ImportedData) {
        let ImportedData {
            node_file_path,
            edge_file_path,
            file_name,
            node_data,
            edge_data,
            source_index,
            target_index,
            source_target_list,
            max_id,
        } = data;
        self.data_model.node_data = node_data.clone();
        self.data_model.edge_data = edge_data.clone();
        self.data_model.edge_source = Some(*source_index);
        self.data_model.edge_target = Some(*target_index);
        self.data_model.source_target_list = Some(source_target_list.clone());
        self.data_model.max_id = *max_id;
        self.data_model.set_status();
        self.app_model.node_file_path = Some(node_file_path.clone());
        self.app_model.edge_file_path = Some(edge_file_path.clone());
        self.app_model.file_name = Some(file_name.clone());
        self.app_model.import_state = ImportState::Success;
        self.app_model.is_import_visible = false;
        self.graphics_model.graphics_resources.init_data(
            self.graphics_model.compute_render_state.clone(),
            &mut self.data_model,
        );
        let text = format!(
            "Node file: {}  \nEdge file: {}",
            self.app_model.node_file_name().unwrap_or(""),
            self.app_model.edge_file_name().unwrap_or("")
        );
        message_info("Import CSV Data Succeeded", text.as_str());
    }

    pub fn clear_data(&mut self) {
        self.app_model.import_state = ImportState::Initial;
        self.app_model.node_file_path = None;
        self.app_model.edge_file_path = None;
        self.data_model.node_data = ExternalData::default();
        self.data_model.edge_data = ExternalData::default();
        self.data_model.max_id = 0;
        self.graphics_model.reset();
        self.data_model.status = GraphicsStatus::default();
    }

    pub fn pick_output_folder_and_then(
        output_folder: &mut String,
        mut then: impl FnMut(&str) -> (),
    ) {
        if output_folder.is_empty() {
            *output_folder = path_to_string(&pick_folder()).unwrap_or(output_folder.clone());
        }
        if !output_folder.is_empty() {
            then(output_folder);
        }
    }

    pub fn render_output(&mut self) {
        Self::pick_output_folder_and_then(&mut self.app_model.output_folder, |folder| {
            self.graphics_model.render_output(String::from(folder));
        });
    }
}
