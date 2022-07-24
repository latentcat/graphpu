use std::path::PathBuf;

#[derive(Debug)]
pub enum ImportState {
    Initial,
    Error,
    Success,
}

impl Default for ImportState {
    fn default() -> Self {
        Self::Initial
    }
}

#[derive(Default)]
pub struct AppModel {
    pub import_visible: bool,
    pub import_state: ImportState,
    pub node_file_path: Option<PathBuf>,
    pub edge_file_path: Option<PathBuf>,
}

impl AppModel {
    pub fn node_file_name(&self) -> Option<&str> {
        self.node_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }

    pub fn edge_file_name(&self) -> Option<&str> {
        self.edge_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }
}
