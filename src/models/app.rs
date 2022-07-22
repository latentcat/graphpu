pub struct AppModel {
  pub import_visible: bool,
}

impl Default for AppModel {
    fn default() -> Self {
      Self {
        import_visible: false,
      }
    }
}