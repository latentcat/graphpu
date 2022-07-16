use crate::widgets::GraphicObject;

use super::AppComponent;

pub struct MainCanvas<'a> {
  graphic_object: &'a mut dyn GraphicObject,
}

impl<'a> MainCanvas<'a> {
  pub fn new(graphic_object: &'a mut dyn GraphicObject) -> Self {
    Self {
      graphic_object,
    }
  }
}

impl AppComponent for MainCanvas<'_> {
  fn add(self, ctx: &egui::Context, ui: &mut egui::Ui) {
    egui::Frame::canvas(&ctx.style()).show(ui, |ui| {
      self.graphic_object.custom_painting(ui);
    });
  }
}