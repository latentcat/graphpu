use std::{fs, path::PathBuf};

use serde::Deserialize;

use crate::utils::csv_loader::load_data;

use super::ImportedData;

#[derive(Deserialize)]
struct DataMetadata {
    name: String,
    path: PathBuf,
}

pub struct ExhibisionModel {
    metadata_list: Vec<DataMetadata>,
    dataset: Vec<ImportedData>,
    cursor: usize,
}

impl ExhibisionModel {
    pub fn new(path: &PathBuf) -> Self {
        let mut model = Self {
            metadata_list: Vec::new(),
            dataset: Vec::new(),
            cursor: 0,
        };

        model.load_from_json(path).unwrap();
        model
    }

    fn load_from_json(&mut self, path: &PathBuf) -> Result<(), std::io::Error> {
        let raw = fs::read_to_string(path)?;
        self.metadata_list.append(&mut serde_json::from_str(&raw)?);
        for metadata in self.metadata_list.iter() {
            let data_path = path.parent().unwrap().join(&metadata.path);
            let data = load_data("", data_path.to_str().unwrap(), 0, 1, metadata.name.as_str()).unwrap();
            self.dataset.push(data);
        }
        Ok(())
    }
}

impl ExhibisionModel {
    pub fn prev(&mut self) {
        self.cursor = (self.cursor + 1) % self.dataset.len();
    }

    pub fn next(&mut self) {
        self.cursor = (self.cursor + self.dataset.len() - 1) % self.dataset.len();
    }

    pub fn current(&self) -> Option<&ImportedData> {
        self.dataset.get(self.cursor)
    }
}
