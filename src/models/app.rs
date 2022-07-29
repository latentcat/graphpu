use std::path::PathBuf;

#[derive(Debug, Default, PartialEq)]
pub enum ImportState {
    #[default]
    Initial,
    Error(String),
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

pub struct AppModel {
    pub import_visible: bool,
    pub import_state: ImportState,
    pub node_file_path: Option<PathBuf>,
    pub edge_file_path: Option<PathBuf>,
    pub stage: Stage,
    pub ne_tab: NodeEdgeTab,
    pub message: String,
    pub pixels_per_point: f32,
}

impl Default for AppModel {
    fn default() -> Self {

        Self { 
            import_visible: false,
            import_state: ImportState::default(),
            node_file_path: None,
            edge_file_path: None,
            stage: Stage::default(),
            ne_tab: NodeEdgeTab::default(),
            message: String::from("Message"),
            pixels_per_point: 1.0
        }
    }
}

impl AppModel {
    pub fn node_file_name(&self) -> Option<&str> {
        self.node_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }

    pub fn edge_file_name(&self) -> Option<&str> {
        self.edge_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }
}
