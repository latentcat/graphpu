use eframe::epaint::Color32;
use std::collections::HashMap;

use strum::Display;

use super::graphics_model::ComputeMethod;

#[derive(Debug, Default, Clone)]
pub struct ExternalData {
    pub headers_str_index: HashMap<String, usize>,
    pub headers_index_str: Vec<String>,
    pub data: Vec<Vec<String>>,
}

#[derive(Debug, Default, Clone)]
pub struct GraphicsStatus {
    pub node_count: usize,
    pub edge_count: usize,
    pub node_data_length: usize,
    pub edge_data_length: usize,
}

#[derive(Display, PartialEq)]
pub enum PositionType {
    Compute,
    Set,
}

#[derive(Display, PartialEq)]
pub enum ColorRamp {
    Ramp1,
    Ramp2,
}

#[derive(Display, PartialEq)]
pub enum ColorPalette {
    Palette1,
    Palette2,
}

#[derive(Display, PartialEq)]
pub enum ColorType {
    Constant,
    Ramp,
    Partition,
}

#[derive(Display, PartialEq)]
pub enum SizeType {
    Constant,
    Ramp,
}

pub struct NodeSettings {
    pub position_type: PositionType,
    pub position_compute: ComputeMethod,
    pub position_set: (f32, f32, f32),

    pub color_type: ColorType,
    pub color_constant: Color32,
    pub color_ramp: (String, ColorRamp),
    pub color_partition: (String, ColorPalette),

    pub size_type: SizeType,
    pub size_constant: f32,
    pub size_ramp: (String, [f32; 2]),
}

pub struct CameraSettings {
    pub look_at: (f32, f32, f32),
    pub rotation: (f32, f32, f32),
    pub distance: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            look_at: (0.0, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0),
            distance: 10.0,
        }
    }
}

impl Default for NodeSettings {
    fn default() -> Self {
        Self {
            position_type: PositionType::Compute,
            position_compute: ComputeMethod::FORCE_ATLAS2,
            position_set: (0.0, 0.0, 0.0),
            color_type: ColorType::Constant,
            color_constant: Color32::WHITE,
            color_ramp: (String::from("None"), ColorRamp::Ramp1),
            color_partition: (String::from("None"), ColorPalette::Palette1),
            size_type: SizeType::Constant,
            size_constant: 1.0,
            size_ramp: (String::from("None"), [0.5, 2.0]),
        }
    }
}

pub struct DataModel {
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
    pub edge_source: Option<usize>,
    pub edge_target: Option<usize>,
    pub source_target_list: Option<Vec<u32>>,
    pub max_id: u32,
    pub status: GraphicsStatus,
    pub node_settings: NodeSettings,
    pub camera_settings: CameraSettings,
}

impl Default for DataModel {
    fn default() -> Self {
        Self {
            node_data: ExternalData::default(),
            edge_data: ExternalData::default(),
            edge_source: None,
            edge_target: None,
            source_target_list: None,
            max_id: 0,
            status: GraphicsStatus::default(),
            node_settings: NodeSettings::default(),
            camera_settings: Default::default(),
        }
    }
}

impl DataModel {
    pub fn set_status(&mut self) {
        self.status.node_count = std::cmp::max(
            self.node_data.data.len(),
            if self.edge_data.data.len() > 0 {
                (self.max_id + 1) as usize
            } else {
                0
            },
        );
        self.status.edge_count = self.edge_data.data.len();
        self.status.node_data_length = self.node_data.data.len();
        self.status.edge_data_length = self.edge_data.data.len();
    }

    pub fn clear_source_target_list(&mut self) {
        // self.source_target_list = None;
    }
}
