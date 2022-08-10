use std::f32::consts::{PI};
use egui::{Pos2, Ui, Vec2};
use crate::models::graphics_lib::Camera;

pub struct Controls {

    // 鼠标指针位置，存在时为 Some
    pointer_pos: Option<Pos2>,

    // 鼠标事件
    primary_clicked: bool,
    primary_down: bool,
    secondary_down: bool,

    // 滚轮 delta
    scroll_delta: Vec2,

    // 鼠标指针 delta
    pointer_delta: Vec2,

    // 视图大小
    viewport_size: Vec2,
    
}

impl Controls {

    pub fn new() -> Self {
        Self {
            pointer_pos: None,
            primary_clicked: false,
            primary_down: false,
            secondary_down: false,
            scroll_delta: Vec2::ZERO,
            pointer_delta: Vec2::ZERO,
            viewport_size: Vec2::ZERO,
        }
    }

}

impl Controls {

    // 传入 egui Ui，更新交互
    // 该函数在 viewport 更新后，渲染开始前调用
    pub fn update_interaction(&mut self, ui: &mut Ui) -> bool {

        // 归零参数
        self.pointer_pos = None;
        self.primary_clicked = false;

        // 记录滚动 delta、指针位置 delta
        self.scroll_delta = ui.input().scroll_delta;
        self.pointer_delta = ui.input().pointer.delta();

        // 从当前的 Ui 中获取 viewport_rect
        // viewport_rect 记录了绘图区域的左上角、右下角
        let viewport_rect = ui.max_rect();
        self.viewport_size = viewport_rect.size();

        // 假设交互无更新...
        let mut is_updated = false;

        // 如果鼠标指针在应用区域内，获取指针位置
        // 注：egui 的 interact_pos 是 Option<Pos2> 类型
        // 仅当鼠标指针在应用窗口范围内，或拖动至 native 应用外是为 Some
        if let Some(pos) = ui.input().pointer.interact_pos() {

            // 如果鼠标指针在绘图区域内
            if viewport_rect.contains(pos) {

                // 记录鼠标指针在绘图区域的相对位置
                // 范围是 0, 0 至 width, height
                self.pointer_pos = Some(pos - viewport_rect.min.to_vec2());

                // 鼠标事件记录
                // 若有事件，标记交互已更新

                if ui.input().pointer.primary_clicked() { self.primary_clicked = true; is_updated = true; }
                if ui.input().pointer.primary_down()    { self.primary_down = true;    is_updated = true; }
                if ui.input().pointer.secondary_down()  { self.secondary_down = true;  is_updated = true; }
                if self.scroll_delta != Vec2::ZERO      { is_updated = true; }

            }
        }

        // 鼠标事件归零

        if !ui.input().pointer.primary_down() {
            self.primary_down = false;
        }

        if !ui.input().pointer.secondary_down() {
            self.secondary_down = false;
        }

        // 返回交互是否更新
        return is_updated;
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {

        // 滚轮缩放
        // 如果鼠标指针在绘图区域内，且当前帧滚轮纵向 delta 不为零
        // 用滚轮纵向 delta 缩放相机
        if self.pointer_pos.is_some() && self.scroll_delta.y != 0.0 {

            // 相机 zoom 函数的传入参数为缩放比例
            // 须满足偏移量累加时缩放比例累乘，故偏移量为常量的指数
            camera.zoom(f32::powf(1.2, -self.scroll_delta.y * 0.03) );

        }

        // 右键上下缩放
        // 如果鼠标右键在绘图区域内按下，且鼠标指针纵向 delta 不为零
        // 用鼠标指针纵向 delta 缩放相机
        if self.secondary_down && self.pointer_delta.y != 0.0 {

            // 原理同滚轮缩放
            camera.zoom(f32::powf(1.2, -self.pointer_delta.y * 0.03) );

        }

        // 左键旋转
        // 如果鼠标左键在绘图区域内按下
        // 用鼠标指针的 x、y delta 在绘图区域的占比计算旋转角度，旋转相机
        if self.primary_down {

            let mut angles = glam::Vec2::new(self.pointer_delta.x, self.pointer_delta.y);

            // 从绘图区域的最左拖动至最右，相机绕 y 轴旋转 PI
            // 从绘图区域的中央拖动至上中，相机绕 x 轴旋转 PI * 0.5
            angles = angles / glam::Vec2::new(self.viewport_size.x, self.viewport_size.y) * PI;

            camera.rotate(angles);

        }
    }

}