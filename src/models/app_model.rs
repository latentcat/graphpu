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
pub enum TableTab {
    #[default]
    Node,
    Edge,
}

#[derive(Default, PartialEq)]
pub enum InspectorTab {
    #[default]
    Node,
    Edge,
    Render,
}

pub struct AppModel {
    pub import_visible: bool,
    pub import_state: ImportState,
    pub node_file_path: Option<PathBuf>,
    pub edge_file_path: Option<PathBuf>,
    pub stage: Stage,
    pub table_tab: TableTab,
    pub inspector_tab: InspectorTab,
    pub message: String,
    pub pixels_per_point: f32,
}

impl Default for AppModel {
    fn default() -> Self {

        Self { 
            import_visible: false,
            import_state: Default::default(),
            node_file_path: None,
            edge_file_path: None,
            stage: Default::default(),
            table_tab: Default::default(),
            inspector_tab: Default::default(),
            message: String::from("中文消息测试"),
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
