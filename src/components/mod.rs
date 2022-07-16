pub mod main_canvas;

pub trait AppComponent {
  fn add(self, ctx: &egui::Context, ui: &mut egui::Ui);
}