use std::borrow::BorrowMut;
use std::ops::Mul;
use egui::{InnerResponse, Response, Ui, Vec2, Widget, WidgetText};
use crate::models::app_model::Tool;
use crate::models::graphics_model::GraphicsResources;

use crate::models::Models;
use crate::widgets::frames::{button_group_style, DEFAULT_BUTTON_PADDING, graphics_frame, TOOL_BUTTON_PADDING, tool_item_group_style, toolbar_inner_frame, toolbar_inner_frame_bottom, toolbar_inner_frame_top};

use super::AppView;

#[derive(Default)]
pub struct GraphicsView;

impl AppView for GraphicsView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {

        if ui.input().key_down(egui::Key::Space) { models.graphics_model.set_computing(false); }

        // 获取是否持续计算、下一帧是否 Dispatch
        let is_computing = models.graphics_model.is_computing;
        let is_dispatching = models.graphics_model.is_dispatching;

        // 无论当前帧是否 Dispatch，下一帧都停止 Dispatch
        models.graphics_model.set_dispatching(false);
    
        egui::CentralPanel::default()
            .frame(graphics_frame(ui.style()))
            .show_inside(ui, |ui| {

                let max_rect = ui.max_rect();

                // 新建一个空 Frame，用于存放 Image
                egui::Frame::none()
                    .show(ui, |ui| {

                        // 如果 Compute Model 已经初始化，即数据导入完成，可以开始渲染
                        // 则获取 Compute Resource
                        if let Some(compute_resources) = &mut models.graphics_model.graphics_resources {

                            // 如果正在持续计算，则计算一次
                            if is_computing {
                                compute_resources.compute();
                            }

                            // 如果当前帧需要 Dispatch，则 Dispatch 一次
                            if is_dispatching {
                                compute_resources.randomize();
                            }

                            // 更新 Viewport，用于处理窗口 resize
                            // update_viewport 方法会判断传入的 Viewport 大小和之前的是否一致
                            // 若发生变化，则更新材质视图，注册 egui 材质 ID，并返回 true
                            // 若无变化，不更新材质视图，返回 false
                            // 其中，pixels_per_point 代表当前每点像素密度

                            compute_resources.update_viewport(
                                max_rect.size().mul(Vec2::from([models.app_model.pixels_per_point; 2]))
                            );

                            compute_resources.update_control(ui, models.graphics_model.is_hover_toolbar);

                            // 若有任何变化，渲染并请求 egui UI 更新
                            if is_computing || is_dispatching || compute_resources.need_update {
                                compute_resources.render();
                                compute_resources.need_update = false;
                                ui.ctx().request_repaint();
                            }

                            // 获取已经注册的 wgpu 材质的 egui 材质 ID
                            let texture_id = compute_resources.viewport_texture_id;

                            // 通过材质 ID 绘制 Image
                            // ui.image(texture_id, max_rect.size());

                            // let response = ui.allocate_rect(max_rect, egui::Sense::hover());

                            ui.allocate_ui_at_rect(max_rect, |ui| {
                                egui::Image::new(texture_id, max_rect.size()).ui(ui)
                            });

                        }


                    });



                ui.allocate_ui_at_rect(max_rect, |ui| {

                    let mut is_hover_toolbar = false;

                    egui::SidePanel::left("toolbar-left-11")
                        .frame(toolbar_inner_frame(ui.style()))
                        .show_separator_line(false)
                        .width_range(0.0..=0.0)
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            ui.spacing_mut().button_padding = TOOL_BUTTON_PADDING;

                            tool_item_group_style(ui.style()).show(ui, |ui| {
                                ui.vertical(|ui| {

                                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 2.0);

                                    tool_item_box(ui, |ui| {
                                        let button = ui.selectable_label(
                                            models.app_model.current_tool == Tool::Select,
                                            egui::RichText::new("☉").size(24.0)
                                        ).on_hover_text("Select");
                                        if button.clicked() {
                                            models.app_model.current_tool = Tool::Select;
                                        }
                                    });

                                    tool_item_box(ui, |ui| {
                                        let button = ui.selectable_label(
                                            models.app_model.current_tool == Tool::Handle,
                                            egui::RichText::new("🕂").size(24.0)
                                        ).on_hover_text("Handle");
                                        if button.clicked() {
                                            models.app_model.current_tool = Tool::Handle;
                                        }
                                    });

                                    tool_item_box(ui, |ui| {
                                        let button = ui.selectable_label(
                                            models.app_model.current_tool == Tool::View,
                                            egui::RichText::new("🎥").size(24.0)
                                        ).on_hover_text("View");
                                        if button.clicked() {
                                            models.app_model.current_tool = Tool::View;
                                        }
                                    });

                                }).response.hovered().then(||{is_hover_toolbar = true});
                            });


                        });

                    egui::TopBottomPanel::top("toolbar-top")
                        .frame(toolbar_inner_frame_top(ui.style()))
                        .show_separator_line(false)
                        .show_inside(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            ui.spacing_mut().button_padding = DEFAULT_BUTTON_PADDING;

                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.horizontal(|ui| {
                                    if let Some(graphics_resources) = &mut models.graphics_model.graphics_resources {

                                        toggle_button(ui, &mut graphics_resources.render_options.is_rendering_bounding_box, "⛶")
                                            .on_hover_text("Toggle Bounding Box")
                                            .clicked().then(|| { need_update(ui, graphics_resources) });

                                        toggle_button(ui, &mut graphics_resources.render_options.is_rendering_axis, "×")
                                            .on_hover_text("Toggle Axes")
                                            .clicked().then(|| { need_update(ui, graphics_resources) });

                                        toggle_button(ui, &mut graphics_resources.render_options.is_rendering_edge, "➖")
                                            .on_hover_text("Toggle Edges")
                                            .clicked().then(|| { need_update(ui, graphics_resources) });

                                        toggle_button(ui, &mut graphics_resources.render_options.is_rendering_node, "⚫")
                                            .on_hover_text("Toggle Nodes")
                                            .clicked().then(|| { need_update(ui, graphics_resources) });

                                    } else {
                                        ui.set_enabled(false);

                                        toggle_button(ui, &mut false, "⛶");
                                        toggle_button(ui, &mut false, "➖");
                                        toggle_button(ui, &mut false, "⚫");
                                    }
                                }).response.hovered().then(||{is_hover_toolbar = true});
                            });

                        });

                    if let Some(graphics_resources) = &mut models.graphics_model.graphics_resources {
                        if !graphics_resources.render_options.is_showing_debug {
                            ui.set_visible(false);
                        }

                        egui::TopBottomPanel::bottom("toolbar-top-2")
                            .frame(toolbar_inner_frame_bottom(ui.style()))
                            .show_separator_line(false)
                            .show_inside(ui, |ui| {
                                ui.set_style(ui.ctx().style());
                                ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                                ui.spacing_mut().button_padding = DEFAULT_BUTTON_PADDING;
                                ui.spacing_mut().interact_size = Vec2::new(4.0, 4.0);

                                ui.with_layout(egui::Layout::from_main_dir_and_cross_align(egui::Direction::TopDown, egui::Align::Max), |ui| {

                                    ui.horizontal(|ui| {
                                        ui.label(format!("{:06}", graphics_resources.compute_frame_count));
                                        ui.label(egui::RichText::new("Compute frames: ").weak());
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label(format!("{:06}", graphics_resources.render_frame_count));
                                        ui.label(egui::RichText::new("Render frames: ").weak());
                                    });

                                    ui.horizontal(|ui| {
                                        ui.label(format!("{:06}", models.app_model.ui_frame_count));
                                        ui.label(egui::RichText::new("UI frames: ").weak());
                                    });

                                });

                            });


                    }

                    models.graphics_model.is_hover_toolbar = is_hover_toolbar;

                });


            });




    }
}


fn toggle_button(ui: &mut egui::Ui, selected: &mut bool, text: impl Into<WidgetText>) -> Response {
    button_group_style(ui.style()).show(ui, |ui| {
        ui.toggle_value(selected.borrow_mut(), text)
            // .clicked().then(||{ graphics_resources.need_update = true; ui.ctx().request_repaint(); });
    }).inner
}

fn need_update(ui: &mut egui::Ui, graphics_resources: &mut GraphicsResources) {
    graphics_resources.need_update = true;
    ui.ctx().request_repaint()
}

fn tool_item_box<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {

    ui.allocate_ui(Vec2::splat(40.0), |ui| {
        button_group_style(ui.style()).show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                add_contents(ui)
            }).inner
        }).inner
    })

}