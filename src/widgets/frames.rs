use eframe::epaint::Stroke;
use egui::{Color32, Vec2};

pub const DEFAULT_BUTTON_PADDING: Vec2 = egui::vec2(4.0, 1.0);
pub const TOOL_BUTTON_PADDING: Vec2 = egui::vec2(0.0, 0.0);

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
        // stroke: style.visuals.window_stroke(),
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
      fill: Color32::from_gray(60),
      stroke: Stroke::NONE,
      ..Default::default()
  }
}

pub fn tool_item_group_style(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::same(2.0),
        fill: Color32::from_gray(30),
        stroke: Stroke {
            width: 4.0,
            color: Color32::from_gray(30),
        },
        ..Default::default()
    }
}

pub fn inspector_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        // stroke: style.visuals.window_stroke(),
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
        // stroke: style.visuals.window_stroke(),
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

pub fn toolbar_inner_frame_top(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin {
            left:   8.0,
            right:  8.0,
            top:    8.0,
            bottom: 0.0,
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

pub fn dock_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(8.0, 0.0),
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        // stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

pub fn drawer_frame(style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin::symmetric(0.0, 0.0),
        rounding: egui::Rounding::none(),
        fill: style.visuals.window_fill(),
        stroke: style.visuals.window_stroke(),
        ..Default::default()
    }
}

pub fn drawer_title_frame(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin {
            left: 12.0,
            right: 12.0,
            top: 6.0,
            bottom: 6.0
        },
        rounding: egui::Rounding::none(),
        ..Default::default()
    }
}

pub fn drawer_message_content_frame(_style: &egui::Style) -> egui::Frame {
    egui::Frame {
        inner_margin: egui::style::Margin {
            left: 12.0,
            right: 6.0,
            top: 3.0,
            bottom: 3.0
        },
        ..Default::default()
    }
}