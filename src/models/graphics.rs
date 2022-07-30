use std::{collections::HashMap, rc::Rc, path::PathBuf};
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
    pub posititon_set: (f32, f32, f32),
    
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
            posititon_set: (0.0, 0.0, 0.0),
            color_type: ColorType::Constant,
            color_constant: Color32::WHITE,
            color_ramp: (Rc::new(String::from("None")), ColorRamp::Ramp1),
            color_partition: (Rc::new(String::from("None")), ColorPalette::Palette1),
            size_type: SizeType::Constant,
            size_constant: 0.0,
            size_ramp: (Rc::new(String::from("None")), [0.5, 2.0]),
        }
    }
}

pub struct GraphicsModel {
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
    pub max_id: usize,
    pub status: GraphicsStatus,
    pub node_settings: NodeSettings,
}

impl Default for GraphicsModel {
    fn default() -> Self {
        Self {
            node_data: ExternalData::default(),
            edge_data: ExternalData::default(),
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

pub fn pick_csv() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text File", &["txt", "csv"])
        .pick_file()
}

pub fn read_headers_from_csv(path: &Option<PathBuf>) -> Result<Vec<Rc<String>>, String> {
    let path = path.as_deref().ok_or("Can't find file")?;
    let err_fomatter = |err| format!("{}", err);

    let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
    Ok(rdr.headers()
        .map_err(err_fomatter)?
        .into_iter()
        .map(|s| Rc::new(s.to_string()))
        .collect()
    )
}

pub fn read_from_csv(path: &Option<PathBuf>) -> Result<ExternalData, String> {
    let path = path.as_deref().ok_or("Can't find file")?;
    let err_fomatter = |err| format!("{}", err);

    let mut rdr = csv::Reader::from_path(path).map_err(err_fomatter)?;
    let data_headers: Vec<_> = rdr
        .headers()
        .map_err(err_fomatter)?
        .into_iter()
        .map(|s| Rc::new(s.to_string()))
        .collect();
    let data: Vec<HashMap<_, _>> = rdr
        .records()
        .into_iter()
        .map(|record| {
            data_headers
                .iter()
                .map(|s| s.clone())
                .zip(record.unwrap().into_iter().map(str::to_string))
                .collect()
        })
        .collect();
    Ok(ExternalData { data_headers, data })
}
