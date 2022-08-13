use std::ops::Mul;
use egui::{Ui, Vec2, Widget};

use crate::models::Models;
use crate::widgets::frames::{DEFAULT_BUTTON_MARGIN, graphics_frame, toolbar_inner_frame, toolbar_timeline_frame};
use crate::widgets::toolbar_modal::ToolbarModal;

use super::AppView;

#[derive(Default)]
pub struct GraphicsView;

impl AppView for GraphicsView {
    fn show(&mut self, models: &mut Models, ui: &mut Ui, _frame: &mut eframe::Frame) {


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
                            let is_viewport_updated = compute_resources.update_viewport(
                                max_rect.size()
                                    .mul(Vec2::from([models.app_model.pixels_per_point; 2])
                                )
                            );

                            let is_control_updated = compute_resources.update_control(ui, models.graphics_model.is_hover_toolbar);

                            // 若有任何变化，渲染并请求 egui UI 更新
                            if is_computing || is_dispatching || is_viewport_updated || is_control_updated {
                                compute_resources.render();
                                ui.ctx().request_repaint();
                            }

                            // 获取已经注册的 wgpu 材质的 egui 材质 ID
                            let texture_id = compute_resources.texture_id;

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

                    egui::TopBottomPanel::bottom("toolbar-bottom")
                        .frame(toolbar_inner_frame(ui.style()))
                        .show_inside(ui, |ui| {
                            toolbar_timeline_frame(ui.style())
                                .show(ui, |ui| {
                                    ui.set_style(ui.ctx().style());
                                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                                    // ui.spacing_mut().button_padding = DEFAULT_BUTTON_MARGIN;

                                    ui.centered_and_justified(|ui| {
                                        ui.set_min_height(60.0);
                                        ui.label(egui::RichText::new("Timeline View").weak());
                                    }).response.hovered().then(||{is_hover_toolbar = true});
                                });

                        });

                    egui::SidePanel::left("toolbar-left")
                        .frame(toolbar_inner_frame(ui.style()))
                        .default_width(100.0)
                        .resizable(false)
                        .show_inside(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            // ui.spacing_mut().button_padding = DEFAULT_BUTTON_MARGIN;

                            ui.vertical(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    let render_button = ui.button("Button 1");
                                    if render_button.clicked() {
                                        //
                                    }
                                });

                                ui.vertical_centered_justified(|ui| {
                                    let render_button = ui.button("Button 2");
                                    if render_button.clicked() {
                                        //
                                    }
                                });
                            }).response.hovered().then(||{is_hover_toolbar = true});

                        });

                    egui::TopBottomPanel::top("toolbar-top")
                        .frame(toolbar_inner_frame(ui.style()))
                        .show_inside(ui, |ui| {
                            ui.set_style(ui.ctx().style());
                            ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);
                            ui.spacing_mut().button_padding = DEFAULT_BUTTON_MARGIN;

                            ui.with_layout(egui::Layout::right_to_left(), |ui| {
                                ui.horizontal(|ui| {

                                    let _ = ui.button("🗁");
                                    let _ = ui.button("🗁");
                                    let _ = ui.button("🗁");
                                }).response.hovered().then(||{is_hover_toolbar = true});
                            });

                        });

                    models.graphics_model.is_hover_toolbar = is_hover_toolbar;

                });


            });




    }
}
