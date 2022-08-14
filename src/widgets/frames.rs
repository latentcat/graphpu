use eframe::epaint::Stroke;
use egui::{Color32, Vec2};

pub const DEFAULT_BUTTON_MARGIN: Vec2 = egui::vec2(4.0, 1.0);

pub fn window_frame(style: &egui::Style) -> egui::Frame {
  egui::Frame {
      inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
      rounding: egui::Rounding::same(8.0),
      fill: style.visuals.window_fill(),
      stroke: style.visuals.window_stroke(),
      ..Default::default()
  }
}

pub fn central_panel_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 8.0),
        rounding: egui::Rounding::none(),
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

pub fn inspector_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

pub fn inspector_inner_frame(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 8.0),
        rounding: egui::Rounding::none(),
        ..Default::default()
    }
}

pub fn graphics_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.5, 0.5),
        rounding: egui::Rounding::none(),
        fill: Color32::from_gray(20),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

pub fn toolbar_inner_frame(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 8.0),
        rounding: egui::Rounding::none(),
        ..Default::default()
    }
}

pub fn toolbar_inner_frame_bottom(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin {
            left:   8.0,
            right:  8.0,
            top:    0.0,
            bottom: 8.0,
        },
        rounding: egui::Rounding::none(),
        ..Default::default()
    }
}

pub fn toolbar_timeline_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 8.0),
        rounding: egui::Rounding::same(4.0),
        fill: style.visuals.window_fill(),
        stroke: Stroke {
            width: 1.0,
            color: Color32::from_white_alpha(10),
        },
        ..Default::default()
    }
}