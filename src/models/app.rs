use std::path::PathBuf;

#[derive(Debug, Default)]
pub enum ImportState {
    #[default]
    Initial,
    Error,
    Success,
}

#[derive(Default, PartialEq)]
pub enum Stage {
    #[default]
    Graphics,
    Table,
}

#[derive(Default, PartialEq)]
pub enum NodeEdgeTab {
    #[default]
    Node,
    Edge,
}

#[derive(Default)]
pub struct AppModel {
    pub import_visible: bool,
    pub import_state: ImportState,
    pub node_file_path: Option<PathBuf>,
    pub edge_file_path: Option<PathBuf>,
    pub stage: Stage,
    pub ne_tab: NodeEdgeTab,
}

impl AppModel {
    pub fn node_file_name(&self) -> Option<&str> {
        self.node_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }

    pub fn edge_file_name(&self) -> Option<&str> {
        self.edge_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }
}
