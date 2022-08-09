use std::cmp::max;
use std::f32::consts;
use std::f32::consts::FRAC_PI_2;

#[derive(Debug, Default)]
pub struct Camera {
    pub position: glam::Vec3,
    pub center: glam::Vec3,
    pub aspect_ratio: f32,
    pub view_matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
    pub is_updated: bool,
    pub near_far: glam::Vec2,
    pub zoom_factor: f32,
}

impl Camera {
    pub fn from(position: glam::Vec3) -> Self {
        let mut camera = Self {
            position,
            aspect_ratio: 1.0,
            near_far: glam::Vec2::new(0.01, 10000.0),
            ..Default::default()
        };

        camera.update_projection_matrix();

        camera
    }
}

impl Camera {

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.is_updated = true;
    }

    pub fn set_position(&mut self, position: glam::Vec3) {
        self.position = position;
        self.is_updated = true;
    }

    pub fn zoom(&mut self, zoom_factor: f32) {

        let dir = self.position - self.center;
        let ( mut length, norm_dir ) = ( dir.length(), dir.normalize() );

        length = f32::clamp(length * zoom_factor, 0.1, 1000.0);

        self.position = length * norm_dir + self.center;
        self.is_updated = true;
    }

    pub fn rotate(&mut self, delta_angles: glam::Vec2) {

        let (mut angles, length) = pos_to_angles_length(self.position - self.center);

        angles += glam::Vec2::new(-delta_angles.x, delta_angles.y);
        angles.y = f32::clamp(angles.y, -0.9 * FRAC_PI_2, 0.9 * FRAC_PI_2);

        self.position = angles_length_to_pos(angles, length) + self.center;

        self.is_updated = true;
    }

    pub fn update_projection_matrix(&mut self) {
        self.zoom_factor = if self.aspect_ratio > 1.0 { 1.0 / self.aspect_ratio } else { 1.0 };
        let projection = glam::Mat4::perspective_rh(consts::FRAC_PI_4 * self.zoom_factor, self.aspect_ratio, self.near_far.x, self.near_far.y);
        let view = glam::Mat4::look_at_rh(
            self.position,
            self.center,
            glam::Vec3::Y,
        );
        self.view_matrix = view;
        self.projection_matrix = projection;
    }
}

fn pos_to_angles_length(pos: glam::Vec3) -> (glam::Vec2, f32) {
    let length = pos.length();

    let angle_x = f32::atan2(pos.x, pos.z);
    let angle_y = f32::atan2(pos.y, f32::sqrt(pos.x * pos.x + pos.z * pos.z));

    return (glam::Vec2::new(angle_x, angle_y), length)


}

fn angles_length_to_pos(angles: glam::Vec2, length: f32) -> glam::Vec3 {

    let pos_y = f32::sin(angles.y);
    let pos_xz_factor = f32::cos(angles.y);

    let pos_x = pos_xz_factor * f32::sin(angles.x);
    let pos_z = pos_xz_factor * f32::cos(angles.x);

    let pos = glam::Vec3::new(pos_x, pos_y, pos_z) * length;

    pos

}