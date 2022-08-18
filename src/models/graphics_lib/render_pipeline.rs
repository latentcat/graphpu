
use crate::models::graphics_lib::Texture;

const MULTISAMPLE_STATE: wgpu::MultisampleState = wgpu::MultisampleState {
    count: 4,
    mask: !0,
    alpha_to_coverage_enabled: false
};

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

pub struct RenderPipeline {
    pub render_pipeline: wgpu::RenderPipeline,
}

impl RenderPipeline {

    pub fn create_node_render_pipeline( device: &wgpu::Device, node_render_pipeline_layout: wgpu::PipelineLayout, node_shader: &wgpu::ShaderModule ) -> Self {

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&node_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &node_shader,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &node_shader,
                entry_point: "main_fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        // alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: MULTISAMPLE_STATE,
            multiview: None,
        });

        Self {
            render_pipeline,
        }

    }

    pub fn create_edge_render_pipeline( device: &wgpu::Device, edge_render_pipeline_layout: wgpu::PipelineLayout, edge_shader: &wgpu::ShaderModule ) -> Self {

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&edge_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &edge_shader,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &edge_shader,
                entry_point: "main_fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: MULTISAMPLE_STATE,
            multiview: None,
        });

        Self {
            render_pipeline,
        }

    }

    pub fn create_axis_render_pipeline( device: &wgpu::Device, axis_render_pipeline_layout: wgpu::PipelineLayout, axis_shader: &wgpu::ShaderModule ) -> Self {

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&axis_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &axis_shader,
                entry_point: "main_vs",
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: 2 * 4,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &axis_shader,
                entry_point: "main_fs",
                targets: &[Some(wgpu::ColorTargetState {
                    format: TEXTURE_FORMAT,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        // alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(), // 2.
                bias: wgpu::DepthBiasState::default(),
            }),
            // depth_stencil: None,
            multisample: MULTISAMPLE_STATE,
            multiview: None,
        });

        Self {
            render_pipeline,
        }

    }

}