use egui::Ui;

use crate::{MainApp, models::Models};

pub mod menubar_view;
pub mod inspector_view;
pub mod graphics_view;
pub mod detail_view;
pub mod import_modal;
pub mod table_view;

pub trait AppView {
  fn show(&mut self, models: &mut Models, ui: &mut Ui);
}