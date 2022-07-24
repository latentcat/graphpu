use std::{collections::HashMap, rc::Rc};

use crate::widgets::GraphicDelegation;

#[derive(Debug, Default)]
pub struct ExternalData {
    pub data_headers: Vec<Rc<String>>,
    pub data: Vec<HashMap<Rc<String>, String>>,
}

pub struct GraphicsModel {
    pub graphic_delegation: Rc<dyn GraphicDelegation>,
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
}

impl GraphicsModel {
    pub fn new(graphic_delegation: Rc<dyn GraphicDelegation>) -> Self {
        Self {
            graphic_delegation,
            node_data: ExternalData::default(),
            edge_data: ExternalData::default(),
        }
    }
}

pub fn read_from_csv(path: &Option<String>) -> Result<ExternalData, String> {
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
    println!("{:?}, {:?}", data_headers, data);
    Ok(ExternalData { data_headers, data })
}
