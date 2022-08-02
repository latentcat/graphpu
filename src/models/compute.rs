use std::borrow::Cow;
use std::mem;
use egui::Vec2;
use wgpu::Label;
use wgpu::util::DeviceExt;
use crate::models::graphics::GraphicsStatus;

use super::graphics::GraphicsModel;


// 须同步修改 Compute WGSL 中每一个函数的 @workgroup_size
// WGSL 中没有宏，不能使用类似 HLSL 的 #define 方法
const PARTICLES_PER_GROUP: u32 = 128;

// 须同步修改各个 WGSL 中的 Node struct
#[repr(C)]
pub struct Node {
    position: [f32; 3],
    velocity: [f32; 3],
}

// 须同步修改各个 WGSL 中的 Edge struct
#[repr(C)]
pub struct Edge {
    source_id: u32,
    target_id: u32,
}

// 计算方法的类型
// Continuous 为需要连续迭代模拟的方法，如 Force Atlas 2
// OneStep 为单次方法，如 Randomize
#[derive(PartialEq)]
pub enum ComputeMethodType {
    Continuous,
    OneStep,
}

// ComputeMethod 结构体，包含名字、类型
#[derive(PartialEq)]
pub struct ComputeMethod(pub &'static str, pub ComputeMethodType);

// 计算方法列表
impl ComputeMethod {
    pub const FORCE_ATLAS2: ComputeMethod = ComputeMethod("Force Atlas 2", ComputeMethodType::Continuous);
    pub const RANDOMIZE: ComputeMethod = ComputeMethod("Randomize", ComputeMethodType::OneStep);
}

// 计算 Model，存放计算状态与计算资源
pub struct ComputeModel {
    pub is_computing: bool,
    pub is_dispatching: bool,
    pub compute_render_state: egui_wgpu::RenderState,
    pub compute_resources: Option<ComputeResources>,
}

impl ComputeModel {

    // 初始化计算 Model，传入 eframe 初始化中的 render_state
    // 仅启动时调用一次
    pub fn init(cc: &eframe::CreationContext) -> Self {
        Self {
            is_computing: false,
            is_dispatching: false,
            compute_render_state: cc.render_state.as_ref().unwrap().clone(),
            compute_resources: None,
        }
    }

    // 重置计算 Model，dispose 并删除计算资源
    pub fn reset(&mut self) {
        if let Some(compute_resources) = &mut self.compute_resources {
            compute_resources.dispose();
        }
        self.compute_resources = None;
    }
}

impl ComputeModel {

    // 切换是否持续计算
    // 仅对 ComputeMethodType::Continuous 生效
    pub fn switch_computing(&mut self) {
        self.is_computing = !self.is_computing;
    }

    // 设置是否持续计算
    // 仅对 ComputeMethodType::Continuous 生效
    pub fn set_computing(&mut self, state: bool) {
        self.is_computing = state;
    }

    // 设置下一帧是否 Dispatch
    // 仅对 ComputeMethodType::OneStep 生效
    pub fn set_dispatching(&mut self, state: bool) {
        self.is_dispatching = state;
    }
}

pub struct ComputeResources {

    // 包含 Node / Edge Count
    status: GraphicsStatus,

    // 包含 Device、Queue、target_format 和 egui_rpass
    render_state: egui_wgpu::RenderState,

    // 用于呈现渲染结果的 Texture View，在 egui 中注册后用 Texture ID 在 Image 组件显示
    texture_view: Option<wgpu::TextureView>,
    pub texture_id: egui::TextureId,

    // UI 视口相关
    viewport_size: egui::Vec2,                      // 视口大小
    pub is_viewport_update: bool,                   // 视口是否更新（在 compute.rs 中设置 true，在
                                                    // graphics_view.rs 中设置 false）

    // Buffers
    uniform_buffer: wgpu::Buffer,                   // 传递 Frame Num 等参数
    quad_buffer:    wgpu::Buffer,                   // Quad 四个顶点数据
    node_buffer:    wgpu::Buffer,
    edge_buffer:    wgpu::Buffer,

    // Bind Group
    compute_bind_group:     wgpu::BindGroup,
    node_render_bind_group: wgpu::BindGroup,
    edge_render_bind_group: wgpu::BindGroup,

    // 渲染管线
    node_render_pipeline: wgpu::RenderPipeline,
    edge_render_pipeline: wgpu::RenderPipeline,

    // 计算管线
    compute_pipeline:   wgpu::ComputePipeline,      // 计算
    randomize_pipeline: wgpu::ComputePipeline,      // 随机位置
    copy_pipeline:      wgpu::ComputePipeline,      // 拷贝

    work_group_count:   u32,                        // 线程组数 = 线程数 / 每组线程数
    frame_num:          usize,                      // 帧计数器
}

// 计算资源 Model，存放和计算和绘图相关的一切资源
impl ComputeResources {

    // 在导入数据后调用的方法，初始化计算和绘图的资源
    pub fn new(render_state: egui_wgpu::RenderState, model: &GraphicsModel) -> Self {

        // 从 Graphics Model 中获取 Node 和 Edge 的数量
        let node_count: u32 = model.status.node_count as u32;
        let edge_count: u32 = model.status.edge_count as u32;

        // Node 和 Edge 结构体的占内存大小，用于计算 Buffer 长度
        let node_struct_size = mem::size_of::<Node>();
        let _edge_struct_size = mem::size_of::<Edge>();

        // 从 render_state 中获取 wgpu 的 device 和 queue
        let device = &render_state.device;
        let _queue = &render_state.queue;

        // 从文件中创建 Shader

        // Compute Shader
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../assets/shader/boids/compute.wgsl"))),
        });

        // Node Shader
        let node_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../assets/shader/boids/M_node.wgsl"))),
        });

        // Edge Shader
        let edge_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../assets/shader/boids/M_edge.wgsl"))),
        });


        // Boids 模拟所用到的参数
        let sim_param_data = [
            0.04f32, // deltaT
            0.1,     // rule1Distance
            0.025,   // rule2Distance
            0.025,   // rule3Distance
            0.02,    // rule1Scale
            0.05,    // rule2Scale
            0.005,   // rule3Scale
        ].to_vec();

        // 创建 Sim Param Buffer 并传入数据
        // 其他 Buffer 在后面创建
        let sim_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation Parameter Buffer"),
            contents: bytemuck::cast_slice(&sim_param_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Bind Group Layout
        // Bind Group 的布局，允许在布局不变的情况下更换 Bind Group 绑定的 Buffer

        // Compute Bind Group Layout
        // 0 - Uniform Buffer
        // 1 - Node Buffer
        // 2 - Edge Buffer
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (sim_param_data.len() * mem::size_of::<f32>()) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (node_count as usize * node_struct_size) as _
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        // Node Render Bind Group Layout
        // 0 - Node Buffer
        let node_render_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((node_count * 16) as _),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        // Node Render Bind Group Layout
        // 0 - Node Buffer
        // 1 - Edge Buffer
        let edge_render_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((node_count * 16) as _),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new((edge_count * 16) as _),
                    },
                    count: None,
                },
            ],
            label: None,
        });

        // Pipeline Layout
        // Pipeline 布局，用于关联 Pipeline 和 Bind Group Layout
        // 在 RenderPass set_pipeline 后，可以通过 set_bind_group 更换符合 Pipeline Layout 中 Bind Group Layout 的 Bind Group

        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("compute"),
            bind_group_layouts: &[&compute_bind_group_layout],
            push_constant_ranges: &[],
        });

        let node_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("node render"),
            bind_group_layouts: &[&node_render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let edge_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edge render"),
            bind_group_layouts: &[&edge_render_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render Pipeline
        // 渲染管线

        // Node Render Pipeline
        // 拓扑结构：Triangle Strip
        let node_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: render_state.target_format.into(),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Edge Render Pipeline
        // 拓扑结构：Line List
        let edge_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: render_state.target_format.into(),
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
                topology: wgpu::PrimitiveTopology::LineList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Compute Pipeline
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "main",
        });

        // Randomize Pipeline
        let randomize_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "randomize",
        });

        // Copy Pipeline
        let copy_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "copy",
        });

        // Buffer 创建

        // 创建 Uniform Buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0u32]),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::MAP_WRITE
                | wgpu::BufferUsages::UNIFORM,
        });

        // Quad 顶点数据
        let vertex_buffer_data =
            [
                // 0
                -1.0f32, -1.0,
                // 1
                1.0, -1.0,
                // 2
                -1.0, 1.0,
                // 3
                1.0, 1.0,
            ];

        // 创建 Quad Buffer 并传入数据
        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::bytes_of(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // 计算对齐后的 Node Buffer 大小
        let node_buffer_size = pad_size(node_struct_size, node_count);

        // 指定大小，创建 Node Buffer，不初始化数据
        let node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Buffer"),
            size: node_buffer_size,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        // 新建初始化 Edge 数据，向 4 个 32 位对齐
        let mut initial_edge_data: Vec<u32> = vec![0; (4 * edge_count) as usize];
        let edge_data = &model.edge_data.data;
        let (source_id, target_id) = (model.edge_source.as_ref().unwrap(), model.edge_target.as_ref().unwrap());

        // 在每四个的第 1、2 个写入 Edge 的 Source 和 Target 数据，转化为与 Compute Shader 同构的 u32
        for (index, edge_instance_chunk) in initial_edge_data.chunks_mut(4).enumerate() {
            edge_instance_chunk[0] = edge_data[index].get(source_id).unwrap().parse::<u32>().unwrap();
            edge_instance_chunk[1] = edge_data[index].get(target_id).unwrap().parse::<u32>().unwrap();
        }

        // 新建 Edge Buffer 并传入数据
        let edge_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Edge Buffer"),
            contents: bytemuck::cast_slice(&initial_edge_data),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        // Bind Group
        // 用于绑定 Buffer 与 Bind Group Layout，后连接 Pipeline Layout 和 Pipeline
        // 需与 Bind Group Layout 保持索引和容量一致

        // Compute Bind Group
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: sim_param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        // Node Render Bind Group
        let node_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &node_render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: node_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        // Edge Render Bind Group
        let edge_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &edge_render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: edge_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        // 计算线程组数
        // 线程组数 = 线程数 / 每组线程数（取整）
        let work_group_count =
            ((node_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let mut boids_resources = ComputeResources {
            status: model.status.clone(),
            render_state,
            texture_view: None,
            texture_id: Default::default(),
            viewport_size: Default::default(),
            is_viewport_update: true,
            uniform_buffer,
            quad_buffer,
            node_buffer,
            edge_buffer,
            compute_bind_group,
            node_render_bind_group,
            edge_render_bind_group,
            node_render_pipeline,
            edge_render_pipeline,
            compute_pipeline,
            randomize_pipeline,
            copy_pipeline,
            work_group_count,
            frame_num: 0,
        };

        boids_resources.randomize();
        boids_resources.update_viewport(Vec2::from([100., 100.]));
        boids_resources.render();

        boids_resources

    }

    pub fn compute(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.push_debug_group("compute boids movement");
        {
            // compute pass
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));
        self.frame_num += 1;
    }

    pub fn randomize(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.frame_num as u32]));
        command_encoder.push_debug_group("randomize boids position");
        {
            // compute pass
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.randomize_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.work_group_count, 1, 1);
            cpass.set_pipeline(&self.copy_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));
        self.frame_num += 1;
    }

    pub fn render(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let view = &self.texture_view.as_ref().unwrap();


        // create render pass descriptor and its color attachments
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: true,
            },
        })];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };

        // get command encoder
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Label::from("qwe") });
        command_encoder.push_debug_group("render boids");
        {
            // render pass
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);
            rpass.set_pipeline(&self.node_render_pipeline);
            rpass.set_bind_group(0, &self.node_render_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
            rpass.draw(0..4, 0..self.status.node_count as u32);

            rpass.set_pipeline(&self.edge_render_pipeline);
            rpass.set_bind_group(0, &self.edge_render_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
            rpass.draw(0..2, 0..self.status.edge_count as u32);
        }
        command_encoder.pop_debug_group();

        queue.submit(Some(command_encoder.finish()));

    }

    pub fn update_viewport(&mut self, new_size: Vec2) {

        let device = &self.render_state.device;
        let _queue = &self.render_state.queue;

        if self.viewport_size != new_size {
            self.viewport_size = new_size;
            self.is_viewport_update = true;

            let texture_extent = wgpu::Extent3d {
                width: self.viewport_size.x as u32,
                height: self.viewport_size.y as u32,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                label: None,
            });

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

            drop(texture);

            if self.texture_view.is_some() {
                self.render_state.egui_rpass.write().free_texture(&self.texture_id);
            }
            let texture_id = self.render_state.egui_rpass.write().register_native_texture(device, &texture_view, wgpu::FilterMode::Linear);

            self.texture_view = Option::from(texture_view);
            self.texture_id = texture_id;
        }
    }

    pub fn dispose(&mut self) {
        self.node_buffer.destroy();
        self.edge_buffer.destroy();
    }


}

// 计算对齐后的 Buffer 长度
// 结构体 Buffer 须向 16 byte 对齐，也就是 4 个 32 位变量
fn pad_size(node_struct_size: usize, num_particles: u32) -> wgpu::BufferAddress {

    let align_mask = wgpu::COPY_BUFFER_ALIGNMENT * 4 - 1;
    let padded_size = ((node_struct_size as u64 + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT);
    let padded_size = (padded_size * num_particles as u64) as wgpu::BufferAddress;

    padded_size
}