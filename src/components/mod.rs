use egui::Ui;

use crate::MainApp;

pub mod menubar_view;
pub mod inspector_view;
pub mod graphics_view;
pub mod detail_view;
pub mod import_modal;
pub mod table_view;

pub trait AppView {
  fn show(self, ctx: &mut MainApp, ui: &mut Ui);
}