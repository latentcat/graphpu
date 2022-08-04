use crate::models::graphics_lib::Camera;

pub struct OrbitControls {
    camera: Box<Camera>,
}

impl OrbitControls {

    pub fn from_target(camera: Box<Camera>) -> Self {
        Self {
            camera
        }
    }

}