pub mod menubar_view;
pub mod inspector_view;
pub mod graphics_view;
pub mod detail_view;

pub trait AppView {
  fn show(self, ctx: &egui::Context);
}

pub trait AppComponent {
  fn add(self, ctx: &egui::Context, ui: &mut egui::Ui);
}