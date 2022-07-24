use egui::Color32;

pub fn window_frame(style: &egui::Style) -> egui::Frame {
  egui::Frame {
      inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
      rounding: egui::Rounding::same(8.0),
      fill: style.visuals.window_fill(),
      stroke: style.visuals.window_stroke(),
      ..Default::default()
  }
}

pub fn inner_panel_frame(_: &egui::Style) -> egui::Frame {
  egui::Frame {
      inner_margin: egui::style::Margin::symmetric(16.0, 16.0),
      rounding: egui::Rounding::none(),
      ..Default::default()
  }
}

pub fn button_group_style(_: &egui::Style) -> egui::Frame {
  egui::Frame {
      inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
      rounding: egui::Rounding::same(2.0),
      fill: Color32::from_white_alpha(10),
      stroke: egui::Stroke::none(),
      ..Default::default()
  }
}