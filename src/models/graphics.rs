use std::{collections::HashMap, rc::Rc, path::PathBuf};

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

    pub fn load_data(&mut self, node_path: &Option<PathBuf>, edge_path: &Option<PathBuf>) -> Result<(), String> {
        self.node_data = read_from_csv(node_path).unwrap_or(ExternalData::default());
        self.edge_data = read_from_csv(edge_path)?;

        // validate edge data
        if self.edge_data.data_headers.len() < 2 {
            Err("The edge file must contain source and target node IDs".to_owned())
        } else {
            Ok(())
        }
    }
}

pub fn pick_csv() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text File", &["txt", "csv"])
        .pick_file()
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
