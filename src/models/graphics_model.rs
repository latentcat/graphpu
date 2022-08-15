use std::borrow::Cow;
use std::mem;
use egui::{Ui, Vec2};
use glam::Vec3;
use wgpu::{Queue, ShaderModule};
use wgpu::util::DeviceExt;
use crate::models::data_model::GraphicsStatus;
use crate::models::graphics_lib::{Camera, Controls, RenderPipeline, Texture};

use rayon::prelude::*;

use super::data_model::DataModel;


// 须同步修改 Compute WGSL 中每一个函数的 @workgroup_size
// WGSL 中没有宏，不能使用类似 HLSL 的 #define 方法
const PARTICLES_PER_GROUP: u32 = 128;


// 须同步修改各个 WGSL 中的 Node struct
#[repr(C)]
pub struct Node {
    _position: [f32; 3],
    _velocity: [f32; 3],
    _mass: u32,
}

#[repr(C)]
struct _Transform {
    _view: glam::Mat4,
    _projection: glam::Mat4,
}

#[repr(C)]
pub struct RenderUniform {
    project_matrix: glam::Mat4,
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

// 绘图 Model，存放计算状态与计算资源
pub struct GraphicsModel {
    pub is_computing: bool,
    pub is_dispatching: bool,
    pub is_hover_toolbar: bool,
    pub compute_render_state: egui_wgpu::RenderState,
    pub graphics_resources: Option<GraphicsResources>,
}

impl GraphicsModel {

    // 初始化计算 Model，传入 eframe 初始化中的 render_state
    // 仅启动时调用一次
    pub fn init(cc: &eframe::CreationContext) -> Self {
        Self {
            is_computing: false,
            is_dispatching: false,
            is_hover_toolbar: false,
            compute_render_state: cc.render_state.as_ref().unwrap().clone(),
            graphics_resources: None,
        }
    }

    // 重置计算 Model，dispose 并删除计算资源
    pub fn reset(&mut self) {
        if let Some(compute_resources) = &mut self.graphics_resources {
            compute_resources.dispose();
        }
        self.graphics_resources = None;
    }
}

impl GraphicsModel {

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

pub struct RenderOptions {
    pub is_rendering_node: bool,
    pub is_rendering_edge: bool,
    pub is_rendering_axis: bool,
    pub is_showing_debug:  bool,
}

// 绘图资源 Model，存放和计算和绘图相关的一切资源
pub struct GraphicsResources {

    // 包含 Node / Edge Count
    status: GraphicsStatus,

    // 包含 Device、Queue、target_format 和 egui_rpass
    render_state: egui_wgpu::RenderState,

    depth_texture: Option<Texture>,

    // 用于呈现渲染结果的 Texture View，在 egui 中注册后用 Texture ID 在 Image 组件显示
    texture_view: Option<wgpu::TextureView>,
    msaa_texture_view: Option<wgpu::TextureView>,
    pub texture_id: egui::TextureId,

    // 视口大小
    viewport_size: egui::Vec2,

    // 相机
    camera: Camera,
    control: Controls,

    // Buffers
    uniform_buffer: wgpu::Buffer,                   // 传递 Frame Num 等参数
    quad_buffer:    wgpu::Buffer,                   // Quad 四个顶点数据
    node_buffer:    wgpu::Buffer,
    edge_buffer:    wgpu::Buffer,
    render_uniform_buffer:    wgpu::Buffer,

    // Bind Group
    compute_bind_group:     wgpu::BindGroup,
    node_render_bind_group: wgpu::BindGroup,
    edge_render_bind_group: wgpu::BindGroup,
    render_uniform_bind_group: wgpu::BindGroup,

    // 渲染管线
    node_render_pipeline: wgpu::RenderPipeline,
    edge_render_pipeline: wgpu::RenderPipeline,
    axis_render_pipeline: wgpu::RenderPipeline,

    // 计算管线
    gen_node_pipeline:   wgpu::ComputePipeline,
    cal_mass_pipeline:   wgpu::ComputePipeline,
    attractive_force_pipeline:   wgpu::ComputePipeline,
    compute_pipeline:   wgpu::ComputePipeline,      // 计算
    randomize_pipeline: wgpu::ComputePipeline,      // 随机位置
    copy_pipeline:      wgpu::ComputePipeline,      // 拷贝

    // 线程组数 = 线程数 / 每组线程数
    node_work_group_count:   u32,
    edge_work_group_count:   u32,
    pub compute_frame_count:          u32,                      // 帧计数器
    pub render_frame_count:          u32,                      // 帧计数器

    pub render_options: RenderOptions,
    pub need_update: bool,

}

impl GraphicsResources {

    // 在导入数据后调用的方法，初始化计算和绘图的资源
    pub fn new(render_state: egui_wgpu::RenderState, model: &mut DataModel) -> Self {

        // 从 Graphics Model 中获取 Node 和 Edge 的数量
        let node_count: u32 = model.status.node_count as u32;
        let edge_count: u32 = model.status.edge_count as u32;

        // Node 和 Edge 结构体的占内存大小，用于计算 Buffer 长度
        let node_struct_size = mem::size_of::<Node>();

        // 从 render_state 中获取 wgpu 的 device 和 queue
        let device = &render_state.device;
        let _queue = &render_state.queue;

        // 从文件中创建 Shader

        let shader_files = [
            include_str!("../assets/shader/boids/CS_boids.wgsl"),
            include_str!("../assets/shader/boids/S_node.wgsl"),
            include_str!("../assets/shader/boids/S_edge.wgsl"),
            include_str!("../assets/shader/boids/S_axis.wgsl"),
        ];

        let shaders = shader_files.par_iter().map(|shader_file| {

            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_file)),
            });

            shader

        }).collect::<Vec<ShaderModule>>();

        // Compute Shader
        let compute_shader = &shaders[0];

        // Node Shader
        let node_shader = &shaders[1];

        // Edge Shader
        let edge_shader = &shaders[2];

        // Axis Shader
        let axis_shader = &shaders[3];


        // Boids 模拟所用到的参数
        let sim_param_data = [
            0.04f32, // deltaT
            0.2,     // rule1Distance
            0.025,   // rule2Distance
            0.025,   // rule3Distance
            0.02,    // rule1Scale
            0.05,    // rule2Scale
            0.01,   // rule3Scale
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
        // 1 - Uniform Buffer
        // 2 - Node Buffer
        // 3 - Edge Buffer
        let compute_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
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
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        // Edge Render Bind Group Layout
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
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
            label: None,
        });

        let render_uniform_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
            bind_group_layouts: &[&render_uniform_bind_group_layout, &node_render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let edge_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("edge render"),
            bind_group_layouts: &[&render_uniform_bind_group_layout, &edge_render_bind_group_layout],
            push_constant_ranges: &[],
        });

        let axis_render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("axis render"),
            bind_group_layouts: &[&render_uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render Pipeline
        // 渲染管线

        // Node Render Pipeline
        // 拓扑结构：Triangle Strip
        let node_render_pipeline = RenderPipeline::create_node_render_pipeline(device, node_render_pipeline_layout, node_shader).render_pipeline;

        // Edge Render Pipeline
        // 拓扑结构：Line List
        let edge_render_pipeline = RenderPipeline::create_edge_render_pipeline(device, edge_render_pipeline_layout, edge_shader).render_pipeline;

        // Node Render Pipeline
        // 拓扑结构：Triangle Strip
        let axis_render_pipeline = RenderPipeline::create_axis_render_pipeline(device, axis_render_pipeline_layout, axis_shader).render_pipeline;



        // Gen Node Pipeline
        let gen_node_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Gen Node Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "gen_node",
        });

        // Cal Mass Pipeline
        let cal_mass_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Cal Mass Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "cal_mass",
        });

        // attractive_force Pipeline
        let attractive_force_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("attractive_force Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: "attractive_force",
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

        let spring_force_buffer_size = node_count * 3 * 4;

        let spring_force_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Buffer"),
            size: spring_force_buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        // 新建 Edge Buffer 并传入数据
        let edge_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Edge Buffer"),
            contents: bytemuck::cast_slice(model.source_target_list.as_ref().unwrap()),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        // 清空 Model 中的 Source Target List
        model.clear_source_target_list();


        let camera = Camera::from(Vec3::new(3.0, 3.0, 6.0));
        let control = Controls::new();

        let uniform_data = generate_uniform_data(&camera);

        // 指定大小，创建 Node Buffer，不初始化数据
        let render_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Edge Buffer"),
            contents: bytemuck::cast_slice(&uniform_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: edge_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: spring_force_buffer.as_entire_binding(),
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

        let render_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: render_uniform_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });

        // 计算线程组数
        // 线程组数 = 线程数 / 每组线程数（取整）
        let node_work_group_count =
            ((node_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;
        let edge_work_group_count =
            ((edge_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let mut boids_resources = GraphicsResources {
            status: model.status.clone(),
            render_state,
            depth_texture: None,
            texture_view: None,
            msaa_texture_view: None,
            texture_id: Default::default(),
            viewport_size: Default::default(),
            camera,
            control,
            uniform_buffer,
            quad_buffer,
            node_buffer,
            edge_buffer,
            render_uniform_buffer,
            compute_bind_group,
            node_render_bind_group,
            edge_render_bind_group,
            render_uniform_bind_group,
            node_render_pipeline,
            edge_render_pipeline,
            axis_render_pipeline,
            gen_node_pipeline,
            cal_mass_pipeline,
            attractive_force_pipeline,
            compute_pipeline,
            randomize_pipeline,
            copy_pipeline,
            node_work_group_count,
            edge_work_group_count,
            compute_frame_count: 0,
            render_frame_count: 0,
            render_options: RenderOptions {
                is_rendering_node: true,
                is_rendering_edge: true,
                is_rendering_axis: false,
                is_showing_debug: false
            },
            need_update: true,
        };

        boids_resources.gen_node();
        // boids_resources.update_viewport(Vec2::from([100., 100.]));
        // boids_resources.render();

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
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
            cpass.set_pipeline(&self.attractive_force_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.edge_work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));
        self.compute_frame_count += 1;
    }


    pub fn gen_node(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.push_debug_group("compute boids movement");
        {
            // compute pass
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.gen_node_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
            cpass.set_pipeline(&self.cal_mass_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.edge_work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));
        self.compute_frame_count += 1;
    }

    pub fn randomize(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.compute_frame_count as u32]));
        command_encoder.push_debug_group("randomize boids position");
        {
            // compute pass
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.randomize_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
            cpass.set_pipeline(&self.copy_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));
        self.compute_frame_count += 1;
    }

    pub fn render(&mut self) {

        self.render_frame_count += 1u32;

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let view = &self.texture_view.as_ref().unwrap();
        let msaa_view = &self.msaa_texture_view.as_ref().unwrap();


        // create render pass descriptor and its color attachments
        let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            view: msaa_view,
            resolve_target: Some(view),
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: false,
            },
        })];
        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.as_ref().unwrap().view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
            // depth_stencil_attachment: None,
        };

        update_transform_matrix(queue, &mut self.camera, &self.render_uniform_buffer, glam::Vec2::new(self.viewport_size.x, self.viewport_size.y));

        // get command encoder
        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.push_debug_group("render boids");
        {
            // render pass
            let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);

            if self.render_options.is_rendering_axis {
                rpass.set_pipeline(&self.axis_render_pipeline);
                rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                rpass.draw(0..4, 0..2);
            }

            if self.render_options.is_rendering_node {
                rpass.set_pipeline(&self.node_render_pipeline);
                rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                rpass.set_bind_group(1, &self.node_render_bind_group, &[]);
                rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                rpass.draw(0..4, 0..self.status.node_count as u32);
            }


            if self.render_options.is_rendering_edge {
                rpass.set_pipeline(&self.edge_render_pipeline);
                rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                rpass.set_bind_group(1, &self.edge_render_bind_group, &[]);
                rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                rpass.draw(0..2, 0..self.status.edge_count as u32);
            }

        }
        command_encoder.pop_debug_group();

        queue.submit(Some(command_encoder.finish()));

    }

    pub fn update_viewport(&mut self, new_size: Vec2) {

        let device = &self.render_state.device;
        let _queue = &self.render_state.queue;

        if self.viewport_size != new_size {
            self.viewport_size = new_size;

            self.camera.set_aspect_ratio(new_size.x / new_size.y as f32);

            let texture_extent = wgpu::Extent3d {
                width: self.viewport_size.x as u32,
                height: self.viewport_size.y as u32,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_extent.clone(),
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                label: None,
            });

            let msaa_texture = device.create_texture(&wgpu::TextureDescriptor {
                size: texture_extent.clone(),
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                label: None,
            });

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mass_texture_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());

            drop(texture);

            if self.texture_view.is_some() {
                self.render_state.egui_rpass.write().free_texture(&self.texture_id);
            }
            let texture_id = self.render_state.egui_rpass.write().register_native_texture(device, &texture_view, wgpu::FilterMode::Linear);

            self.depth_texture = Some(Texture::create_depth_texture(&device, &texture_extent, "depth_texture"));

            self.texture_view = Option::from(texture_view);
            self.msaa_texture_view = Option::from(mass_texture_view);
            self.texture_id = texture_id;

            self.need_update = true;
        }
    }

    pub fn update_control(&mut self, ui: &mut Ui, is_hover_toolbar: bool) {

        self.control.update_interaction(ui, is_hover_toolbar);
        self.control.update_camera(&mut self.camera);

        if self.control.is_update {
            self.need_update = true;
        }

        self.control.is_update = false;

    }

    pub fn dispose(&mut self) {
        self.node_buffer.destroy();
        self.edge_buffer.destroy();
    }


}

fn update_transform_matrix(queue: &Queue, camera: &mut Camera, render_uniform_buffer: &wgpu::Buffer, viewport_size: glam::Vec2) {

    if camera.is_updated {

        camera.update_projection_matrix();

        let uniform_data = generate_uniform_data(camera);

        queue.write_buffer(&render_uniform_buffer, 0, bytemuck::cast_slice(&uniform_data));
        camera.is_updated = false;

    }

}

fn generate_uniform_data(camera: &Camera) -> Vec<f32> {

    let mut uniform_data: Vec<f32> = camera.view_matrix.as_ref().to_vec();
    uniform_data.append(&mut camera.projection_matrix.as_ref().to_vec());
    uniform_data.append(&mut [camera.aspect_ratio, camera.zoom_factor].to_vec());
    uniform_data.append(&mut camera.near_far.to_array().to_vec());

    uniform_data

}

// 计算对齐后的 Buffer 长度
// 结构体 Buffer 须向 16 byte 对齐，也就是 4 个 32 位变量
fn pad_size(node_struct_size: usize, num_particles: u32) -> wgpu::BufferAddress {

    let align_mask = wgpu::COPY_BUFFER_ALIGNMENT * 4 - 1;
    let padded_size = ((node_struct_size as u64 + align_mask) & !align_mask).max(wgpu::COPY_BUFFER_ALIGNMENT);
    let padded_size = (padded_size * num_particles as u64) as wgpu::BufferAddress;

    padded_size
}