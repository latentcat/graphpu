use std::path::PathBuf;
use std::sync::Arc;
use egui::{Color32, Rounding, Stroke, Style, Vec2, Visuals};
use egui::style::{Selection, Spacing, Widgets, WidgetVisuals};
use crate::utils::file::desktop_dir_or_empty;

#[derive(Debug, Default, PartialEq)]
pub enum ImportState {
    #[default]
    Initial,
    Error(String),
    Success,
}

#[derive(Default, PartialEq)]
pub enum MainStage {
    #[default]
    Graphics,
    Table,
}

#[derive(Default, PartialEq)]
pub enum DockStage {
    #[default]
    None,
    Messages,
    Timeline,
}

#[derive(Default, PartialEq)]
pub enum TableTab {
    #[default]
    Node,
    Edge,
}

#[derive(Default, PartialEq)]
pub enum InspectorTab {
    #[default]
    Graph,
    Node,
    Edge,
    Camera,
    Options,
}

#[derive(Default, PartialEq)]
pub enum Tool {
    Select,
    Handle,
    #[default]
    View,
}

pub struct AppModel {
    pub is_import_visible: bool,
    pub is_timeline_expand: bool,
    pub import_state: ImportState,
    pub node_file_path: Option<PathBuf>,
    pub edge_file_path: Option<PathBuf>,
    pub output_folder: String,
    pub main_stage: MainStage,
    pub dock_stage: DockStage,
    pub table_tab: TableTab,
    pub inspector_tab: InspectorTab,
    pub pixels_per_point: f32,
    pub current_tool: Tool,
    pub ui_frame_count: u32,
    pub dock_style: Arc<Style>,
}

impl Default for AppModel {
    fn default() -> Self {
        let dock_style = Style {
            spacing: Spacing {
                button_padding: Vec2::from([8.0, 3.0]),
                interact_size: Vec2::from([0.0, 0.0]),
                ..Default::default()
            },
            visuals: Visuals {
                widgets: Widgets {
                    inactive: WidgetVisuals {
                        bg_fill: Color32::from_gray(60), // button background
                        bg_stroke: Default::default(),
                        fg_stroke: Stroke::new(1.0, Color32::from_white_alpha(120)), // button text
                        rounding: Rounding::none(),
                        expansion: 0.0,
                    },
                    hovered: WidgetVisuals {
                        bg_fill: Color32::from_gray(45),
                        bg_stroke: Stroke::NONE, // e.g. hover over window edge or button
                        fg_stroke: Stroke::new(1.5, Color32::from_gray(240)),
                        rounding: Rounding::none(),
                        expansion: 0.0,
                    },
                    active: WidgetVisuals {
                        bg_fill: Color32::from_gray(70),
                        bg_stroke: Stroke::NONE,
                        fg_stroke: Stroke::new(2.0, Color32::WHITE),
                        rounding: Rounding::none(),
                        expansion: 0.0,
                    },

                    ..Default::default()
                },
                selection: Selection {
                    bg_fill: Color32::from_gray(60),
                    stroke: Stroke::new(1.0, Color32::from_white_alpha(220)), // button text
                },
                ..Default::default()
            },
            ..Default::default()
        };

        Self {
            is_import_visible: false,
            is_timeline_expand: false,
            import_state: Default::default(),
            node_file_path: None,
            edge_file_path: None,
            output_folder: desktop_dir_or_empty(),
            main_stage: Default::default(),
            dock_stage: Default::default(),
            table_tab: Default::default(),
            inspector_tab: Default::default(),
            pixels_per_point: 1.0,
            current_tool: Default::default(),
            ui_frame_count: 0,
            dock_style: Arc::new(dock_style),
        }
    }
}

impl AppModel {
    pub fn node_file_name(&self) -> Option<&str> {
        self.node_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }

    pub fn edge_file_name(&self) -> Option<&str> {
        self.edge_file_path.as_ref()?.file_name().and_then(|s| s.to_str())
    }
}
