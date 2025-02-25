use std::sync::Arc;
use egui::epaint::ahash::HashMap;
use wgpu::{Device, ShaderModule, BindGroupLayoutEntry};

pub struct ComputeShader {
    pub shader: ShaderModule,
    pub device: Arc<Device>,
    pub kernels: HashMap<String, ComputeKernel>
}

pub struct ComputeBuffer<'a> {
    pub(crate) binding:        u32,
    pub(crate) buffer_type:    ComputeBufferType,
    pub(crate) buffer:         wgpu::BindingResource<'a>
}

pub enum ComputeBufferType {
    Storage,
    Uniform,
    StorageReadOnly,
}

impl ComputeShader {
    // pub fn create_pipeline(&mut self, entry_point: &str) -> ComputePipeline {
    //     let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    //         label: Some("Gen Node Pipeline"),
    //         layout: Some(&self.pipeline_layout),
    //         module: &self.shader,
    //         entry_point,
    //     });
    //
    //     compute_pipeline
    // }

    pub fn create_compute_kernel(&mut self, entry_point: &str, buffers: Vec<ComputeBuffer>) {

        let bind_group_layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: buffers.iter().map(|compute_buffer| BindGroupLayoutEntry {
                binding: compute_buffer.binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: match compute_buffer.buffer_type {
                        ComputeBufferType::Storage => wgpu::BufferBindingType::Storage { read_only: false },
                        ComputeBufferType::Uniform => wgpu::BufferBindingType::Uniform,
                        ComputeBufferType::StorageReadOnly => wgpu::BufferBindingType::Storage { read_only: true },
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect::<Vec<_>>()
            .as_slice(),
            label: None,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: buffers.iter().map(|compute_buffer| wgpu::BindGroupEntry {
                binding: compute_buffer.binding,
                resource: compute_buffer.buffer.clone(),
            })
            .collect::<Vec<_>>()
            .as_slice(),
            label: None,
        });

        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&*(entry_point.to_owned() + " pipeline layout")),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let compute_pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&*(entry_point.to_owned() + " pipeline")),
            layout: Some(&pipeline_layout),
            module: &self.shader,
            entry_point,
        });

        let compute_kernel = ComputeKernel {
            bind_group,
            compute_pipeline
        };

        self.kernels.insert(
            entry_point.parse().unwrap(),
            compute_kernel
        );

    }
}

pub struct ComputeKernel {
    pub bind_group: wgpu::BindGroup,
    pub compute_pipeline: wgpu::ComputePipeline
}