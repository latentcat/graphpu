use crate::context::AppContext;

pub mod menubar_view;
pub mod inspector_view;
pub mod graphics_view;
pub mod detail_view;

pub trait AppView {
  fn show(self, ctx: &mut AppContext);
}

pub trait AppComponent {
  fn add(self, ctx: &mut AppContext, ui: &mut egui::Ui);
}