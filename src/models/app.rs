#[derive(Default)]
pub struct AppModel {
  pub import_visible: bool,
  pub node_file_path: Option<String>,
  pub edge_file_path: Option<String>,
}