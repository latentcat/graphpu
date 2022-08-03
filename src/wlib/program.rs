use wgpu::{Device, Queue};
use crate::wlib::{Buffer, ComputeShader, Renderer};

pub struct WlibProgram {

}


impl WlibProgram {

    pub fn init(device: Device, queue: Queue) {}
}

impl WlibProgram {

    // -> Renderer
    pub fn create_renderer() {

    }

    pub fn create_compute_shader() -> ComputeShader {
        ComputeShader {}
    }

    pub fn create_buffer() -> Buffer {
        Buffer {}
    }
}