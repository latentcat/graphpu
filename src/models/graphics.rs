use std::{collections::HashMap, rc::Rc, path::PathBuf};

#[derive(Debug, Default, Clone)]
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

pub struct GraphicsModel {
    pub node_data: ExternalData,
    pub edge_data: ExternalData,
    pub max_id: usize,
    pub status: GraphicsStatus,
}

impl Default for GraphicsModel {
    fn default() -> Self {
        Self {
            node_data: ExternalData::default(),
            edge_data: ExternalData::default(),
            max_id: 0,
            status: GraphicsStatus::default(),
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
