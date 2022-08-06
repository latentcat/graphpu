use std::f32::consts;

#[derive(Debug, Default)]
pub struct Camera {
    pub position: glam::Vec3,
    pub aspect_ratio: f32,
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
}

impl Camera {
    pub fn from(position: glam::Vec3) -> Self {
        let mut camera = Self {
            position,
            aspect_ratio: 1.0,
            ..Default::default()
        };

        camera.update_projection_matrix();

        camera
    }
}

impl Camera {

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_projection_matrix();
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position;
        self.update_projection_matrix();
    }

    pub fn update_projection_matrix(&mut self) {
        let zoom_factor = if self.aspect_ratio > 1.0 { self.aspect_ratio } else { 1.0 };
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4 / zoom_factor, self.aspect_ratio, 0.1, 1000.0);
        let view = glam::Mat4::look_at_rh(
            self.position,
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );
        self.view_matrix = view;
        self.projection_matrix = projection;
    }
}