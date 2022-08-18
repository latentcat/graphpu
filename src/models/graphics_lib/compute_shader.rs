use std::sync::Arc;
use wgpu::{ComputePipeline, Device, ShaderModule};

pub struct ComputeShader {
    pub pipeline_layout: wgpu::PipelineLayout,
    pub shader: ShaderModule,
    pub device: Arc<Device>,
}

impl ComputeShader {

    pub fn create(device: Arc<Device>, shader: wgpu::ShaderModule, bind_group_layouts: &[&wgpu::BindGroupLayout]) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compute"),
            bind_group_layouts,
            push_constant_ranges: &[],
        });

        Self {
            pipeline_layout,
            shader,
            device,
        }
    }

}

impl ComputeShader {
    pub fn create_pipeline(&mut self, entry_point: &str) -> ComputePipeline {
        let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gen Node Pipeline"),
            layout: Some(&self.pipeline_layout),
            module: &self.shader,
            entry_point,
        });

        compute_pipeline
    }
}