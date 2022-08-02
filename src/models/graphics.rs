use std::{collections::HashMap, rc::Rc};
use eframe::epaint::Color32;

use strum::Display;

use super::compute::ComputeMethod;

#[derive(Debug, Default)]
pub struct ExternalData {
    pub data_headers: Vec<Rc<String>>,
    pub data: Vec<HashMap<Rc<String>, String>>,
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
    pub color_ramp: (Rc<String>, ColorRamp),
    pub color_partition: (Rc<String>, ColorPalette),
    
    pub size_type: SizeType,
    pub size_constant: f32,
    pub size_ramp: (Rc<String>, [f32; 2]),
}

impl Default for NodeSettings {
    fn default() -> Self {
        Self {
            position_type: PositionType::Compute,
            position_compute: ComputeMethod::FORCE_ATLAS2,
            position_set: (0.0, 0.0, 0.0),
            color_type: ColorType::Constant,
            color_constant: Color32::WHITE,
            color_ramp: (Rc::new(String::from("None")), ColorRamp::Ramp1),
            color_partition: (Rc::new(String::from("None")), ColorPalette::Palette1),
            size_type: SizeType::Constant,
            size_constant: 1.0,
            size_ramp: (Rc::new(String::from("None")), [0.5, 2.0]),
        }
    }
}

pub struct GraphicsModel {
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
    pub edge_source: Option<Rc<String>>,
    pub edge_target: Option<Rc<String>>,
    pub max_id: usize,
    pub status: GraphicsStatus,
    pub node_settings: NodeSettings,
}

impl Default for GraphicsModel {
    fn default() -> Self {
        Self {
            node_data: ExternalData::default(),
            edge_data: ExternalData::default(),
            edge_source: None,
            edge_target: None,
            max_id: 0,
            status: GraphicsStatus::default(),
            node_settings: NodeSettings::default(),
        }
    }
}

impl GraphicsModel {
    pub fn set_status(&mut self) {
        self.status.node_count =
            std::cmp::max(
                self.node_data.data.len(),
                if self.edge_data.data.len() > 0 { self.max_id + 1 } else { 0 }
            );
        self.status.edge_count = self.edge_data.data.len();
        self.status.node_data_length = self.node_data.data.len();
        self.status.edge_data_length = self.edge_data.data.len();
    }
}