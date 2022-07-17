use crate::MainApp;

pub struct AppContext<'a> {
  pub app: &'a mut MainApp,
  pub egui_ctx: &'a egui::Context,
}

impl<'a> AppContext<'a>  {
    pub fn from(app: &'a mut MainApp, egui_ctx: &'a egui::Context) -> Self {
      Self {
        app,
        egui_ctx,
      }
    }
}