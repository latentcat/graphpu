use bytemuck::{Pod, Zeroable};
use crate::models::graphics_lib::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct Uniforms {
    view:       [f32; 16],
    projection: [f32; 16],
    time:       [f32; 4],
    screen:     [f32; 4],
    camera:     [f32; 4],
}

pub fn generate_uniforms(camera: &Camera, viewport_size: glam::Vec2) -> Uniforms {
    Uniforms {
        view:       *camera.view_matrix.as_ref(),
        projection: *camera.projection_matrix.as_ref(),
        time:       [0.0, 0.0, 0.0, 0.0],
        screen:
        [
            viewport_size.x as f32,
            viewport_size.y as f32,
            0.0,
            0.0
        ],
        camera:
        [
            camera.aspect_ratio as f32,
            camera.zoom_factor  as f32,
            camera.near_far.x,
            camera.near_far.y
        ],
    }
}
