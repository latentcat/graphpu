use std::borrow::Cow;
use std::mem;
use chrono::Local;
use egui::{Ui, Vec2};
use glam::Vec3;
use wgpu::{ Queue, ShaderModule };
use wgpu::util::DeviceExt;
use crate::models::data_model::GraphicsStatus;
use crate::models::graphics_lib::{BufferDimensions, Camera, Controls, RenderPipeline, Texture};

use rayon::prelude::*;
use crate::models::graphics_lib::bind_group_layout::BindGroupLayout;
use crate::models::graphics_lib::compute_shader::ComputeShader;
use crate::utils::file::create_png;

use super::data_model::DataModel;


// 须同步修改 Compute WGSL 中每一个函数的 @workgroup_size
// WGSL 中没有宏，不能使用类似 HLSL 的 #define 方法
const PARTICLES_PER_GROUP: u32 = 128;


// 须同步修改各个 WGSL 中的 Node struct
#[repr(C)]
pub struct Node {
    _position: [f32; 3],
    _force: [f32; 3],
    _prev_force: [f32; 3],
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

#[repr(C)]
pub struct Bound {
    _bound_min: [f32; 3],
    _bound_max: [f32; 3],
}

#[repr(C)]
pub struct BHTree {
    _max_depth: u32,
    _bottom: u32,
    _radius: f32,
}

#[repr(C)]
pub struct BHTreeNode {
    _position: [f32; 3],
    _mass: i32,
    _count: i32,
    _start: i32,
    _sort: i32,
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
            compute_render_state: cc.wgpu_render_state.as_ref().unwrap().clone(),
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

    pub fn render_output(&mut self, out_folder:String) {
        if let Some(graphics_resources) = &mut self.graphics_resources {
            graphics_resources.prepare_output();
            graphics_resources.render();
            graphics_resources.output_png_after_render(out_folder.to_owned());
        }
    }
}

pub struct RenderOptions {
    pub is_rendering_node: bool,
    pub is_rendering_edge: bool,
    pub is_rendering_axis: bool,
    pub is_showing_debug:  bool,
}

pub struct ComputePipelines {
    gen_node:              wgpu::ComputePipeline,
    cal_mass:              wgpu::ComputePipeline,
    cal_gravity:           wgpu::ComputePipeline,
    attractive_force:      wgpu::ComputePipeline,
    reduction_bounding:    wgpu::ComputePipeline,
    bounding_box:          wgpu::ComputePipeline,
    clear_1:               wgpu::ComputePipeline,
    tree_building:         wgpu::ComputePipeline,
    clear_2:               wgpu::ComputePipeline,
    summarization:         wgpu::ComputePipeline,
    sort:                  wgpu::ComputePipeline,
    electron_force:        wgpu::ComputePipeline,
    compute:               wgpu::ComputePipeline,
    displacement:          wgpu::ComputePipeline,
    randomize:             wgpu::ComputePipeline,
    copy:                  wgpu::ComputePipeline,
}

// 绘图资源 Model，存放和计算和绘图相关的一切资源
pub struct GraphicsResources {

    // 包含 Node / Edge Count
    status:                         GraphicsStatus,

    // 包含 Device、Queue、target_format 和 egui_rpass
    render_state:                   egui_wgpu::RenderState,

    // 视口大小
    viewport_size:                  egui::Vec2,

    viewport_texture_extent:        wgpu::Extent3d,
    viewport_texture:               Option<Texture>,
    viewport_msaa_texture:          Option<Texture>,
    viewport_depth_texture:         Option<Texture>,

    pub viewport_texture_id:        egui::TextureId,

    is_render_output:               bool,

    output_texture_extent:          wgpu::Extent3d,
    output_texture:                 Option<Texture>,
    output_msaa_texture:            Option<Texture>,
    output_depth_texture:           Option<Texture>,

    // 相机
    camera:                         Camera,
    control:                        Controls,

    // Buffers
    uniform_buffer:                 wgpu::Buffer,
    quad_buffer:                    wgpu::Buffer,
    node_buffer:                    wgpu::Buffer,
    edge_buffer:                    wgpu::Buffer,
    render_uniform_buffer:          wgpu::Buffer,
    bounding_buffer:                wgpu::Buffer,
    tree_buffer:                    wgpu::Buffer,
    tree_node_buffer:               wgpu::Buffer,
    tree_child_buffer:              wgpu::Buffer,

    // Bind Group
    compute_bind_group:             wgpu::BindGroup,
    node_render_bind_group:         wgpu::BindGroup,
    edge_render_bind_group:         wgpu::BindGroup,
    render_uniform_bind_group:      wgpu::BindGroup,

    // 渲染管线
    node_render_pipeline:           wgpu::RenderPipeline,
    edge_render_pipeline:           wgpu::RenderPipeline,
    axis_render_pipeline:           wgpu::RenderPipeline,

    // 计算管线
    compute_pipelines:              ComputePipelines,

    // 线程组数 = 线程数 / 每组线程数
    node_work_group_count:          u32,
    edge_work_group_count:          u32,
    tree_node_work_group_count:     u32,
    pub compute_frame_count:        u32,                      // 帧计数器
    pub render_frame_count:         u32,                      // 帧计数器

    pub render_options:             RenderOptions,
    pub need_update:                bool,

}

impl GraphicsResources {

    // 在导入数据后调用的方法，初始化计算和绘图的资源
    pub fn new(render_state: egui_wgpu::RenderState, model: &mut DataModel) -> Self {

        // 从 Graphics Model 中获取 Node 和 Edge 的数量
        let node_count = model.status.node_count as u32;
        let edge_count = model.status.edge_count as u32;
        let tree_node_count = get_tree_node_count(&node_count);

        // Node 和 Edge 结构体的占内存大小，用于计算 Buffer 长度
        let node_struct_size = mem::size_of::<Node>();

        // 从 render_state 中获取 wgpu 的 device 和 queue
        let device = &render_state.device;
        let _queue = &render_state.queue;

        // 从文件中创建 Shader

        let shader_files = [
            include_str!("../assets/shader/boids/S_node.wgsl"),
            include_str!("../assets/shader/boids/S_edge.wgsl"),
            include_str!("../assets/shader/boids/S_axis.wgsl"),
            include_str!("../assets/shader/boids/CS_boids.wgsl"),
        ];

        let mut shaders = shader_files.par_iter().map(|shader_file| unsafe {

            let shader = device.create_shader_module_unchecked(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_file)),
            });

            shader

        }).collect::<Vec<ShaderModule>>();


        // Compute Shader
        let compute_shader = shaders.pop().unwrap();

        // Node Shader
        let node_shader = &shaders[0];

        // Edge Shader
        let edge_shader = &shaders[1];

        // Axis Shader
        let axis_shader = &shaders[2];

        // Bind Group Layout
        // Bind Group 的布局，允许在布局不变的情况下更换 Bind Group 绑定的 Buffer

        let compute_bind_group_layout =         BindGroupLayout::create_compute_bind_group_layout(device).bing_group_layout;
        let node_render_bind_group_layout =     BindGroupLayout::create_node_render_bind_group_layout(device).bing_group_layout;
        let edge_render_bind_group_layout =     BindGroupLayout::create_edge_render_bind_group_layout(device).bing_group_layout;
        let render_uniform_bind_group_layout =  BindGroupLayout::create_render_uniform_bind_group_layout(device).bing_group_layout;


        // Render Pipeline
        // 渲染管线

        let node_render_pipeline = RenderPipeline::create_node_render_pipeline(
            device,
            &[&render_uniform_bind_group_layout, &node_render_bind_group_layout],
            node_shader
        ).render_pipeline;

        let edge_render_pipeline = RenderPipeline::create_edge_render_pipeline(
            device,
            &[&render_uniform_bind_group_layout, &edge_render_bind_group_layout],
            edge_shader
        ).render_pipeline;

        let axis_render_pipeline = RenderPipeline::create_axis_render_pipeline(
            device,
            &[&render_uniform_bind_group_layout], axis_shader
        ).render_pipeline;

        let mut graph_compute = ComputeShader::create(device.clone(), compute_shader, &[&compute_bind_group_layout]);

        let compute_pipelines = ComputePipelines {
            gen_node:               graph_compute.create_pipeline("gen_node"),
            cal_mass:               graph_compute.create_pipeline("cal_mass"),
            cal_gravity:            graph_compute.create_pipeline("cal_gravity_force"),
            attractive_force:       graph_compute.create_pipeline("attractive_force"),
            reduction_bounding:     graph_compute.create_pipeline("reduction_bounding"),
            bounding_box:           graph_compute.create_pipeline("bounding_box"),
            clear_1:                graph_compute.create_pipeline("clear_1"),
            tree_building:          graph_compute.create_pipeline("tree_building"),
            clear_2:                graph_compute.create_pipeline("clear_2"),
            summarization:          graph_compute.create_pipeline("summarization"),
            sort:                   graph_compute.create_pipeline("sort"),
            electron_force:         graph_compute.create_pipeline("electron_force"),
            compute:                graph_compute.create_pipeline("main"),
            displacement:           graph_compute.create_pipeline("displacement"),
            randomize:              graph_compute.create_pipeline("randomize"),
            copy:                   graph_compute.create_pipeline("copy"),
        };


        // 计算线程组数
        // 线程组数 = 线程数 / 每组线程数（取整）

        let node_work_group_count =
            ((node_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let edge_work_group_count =
            ((edge_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let tree_node_work_group_count = 
            (tree_node_count as f32 / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

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

        let bounding_buffer_size = node_work_group_count * 8 * 4;

        let bounding_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bounding Buffer"),
            size: bounding_buffer_size as wgpu::BufferAddress,
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

        // Tree Buffer
        let tree_buffer_size = pad_size(mem::size_of::<BHTree>(), 1);
        let tree_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tree Buffer"),
            size: tree_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        // Tree Node Buffer
        let tree_node_buffer_size = pad_size(mem::size_of::<BHTreeNode>(), tree_node_count + 1);
        let tree_node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tree Node Buffer"),
            size: tree_node_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        // Tree Child Buffer
        let tree_child_buffer_size = pad_size(mem::size_of::<i32>(), (tree_node_count + 1) * 8);
        let tree_child_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tree Child Buffer"),
            size: tree_child_buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
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
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: edge_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: spring_force_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: bounding_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: tree_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: tree_node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: tree_child_buffer.as_entire_binding(),
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

        let mut boids_resources = GraphicsResources {
            status: model.status.clone(),
            render_state,
            viewport_depth_texture: None,
            output_depth_texture: None,
            viewport_texture: None,
            viewport_msaa_texture: None,
            output_texture: None,
            output_msaa_texture: None,
            viewport_texture_id: Default::default(),
            viewport_size: Default::default(),
            viewport_texture_extent: Default::default(),
            output_texture_extent: Default::default(),
            is_render_output: false,
            camera,
            control,
            uniform_buffer,
            quad_buffer,
            node_buffer,
            edge_buffer,
            render_uniform_buffer,
            bounding_buffer,
            tree_buffer,
            tree_node_buffer,
            tree_child_buffer,
            compute_bind_group,
            node_render_bind_group,
            edge_render_bind_group,
            render_uniform_bind_group,
            node_render_pipeline,
            edge_render_pipeline,
            axis_render_pipeline,
            compute_pipelines,
            node_work_group_count,
            edge_work_group_count,
            tree_node_work_group_count,
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
            cpass.set_pipeline(&self.compute_pipelines.cal_gravity);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.attractive_force);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.edge_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.reduction_bounding);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.bounding_box);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(1, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.clear_1);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.tree_node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.tree_building);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.clear_2);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.tree_node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.summarization);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.sort);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.electron_force);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.compute);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);

            cpass.set_pipeline(&self.compute_pipelines.displacement);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
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
            cpass.set_pipeline(&self.compute_pipelines.gen_node);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
            cpass.set_pipeline(&self.compute_pipelines.cal_mass);
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
            cpass.set_pipeline(&self.compute_pipelines.randomize);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.dispatch_workgroups(self.node_work_group_count, 1, 1);
            cpass.set_pipeline(&self.compute_pipelines.copy);
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

        let is_render_output = self.is_render_output;
        self.is_render_output = false;

        let (resolve_target, view, depth_view) = if !is_render_output {
            (
                Some(&self.viewport_texture.as_ref().unwrap().view),
                &self.viewport_msaa_texture.as_ref().unwrap().view,
                &self.viewport_depth_texture.as_ref().unwrap().view
            )
        } else {
            (
                Some(&self.output_texture.as_ref().unwrap().view),
                &self.output_msaa_texture.as_ref().unwrap().view,
                &self.output_depth_texture.as_ref().unwrap().view
            )
        };

        let color_attachment;

        color_attachment = Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: false,
            },
        });



        let render_pass_descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[color_attachment],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
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

            if !is_render_output {
                if self.render_options.is_rendering_axis {
                    rpass.set_pipeline(&self.axis_render_pipeline);
                    rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                    rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                    rpass.draw(0..4, 0..2);
                }
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
                rpass.draw(0..4, 0..self.status.edge_count as u32);
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

            let texture = Texture::create_texture(device, &texture_extent, 1);
            let msaa_texture = Texture::create_texture(device, &texture_extent, 4);
            let depth_texture = Texture::create_depth_texture(&device, &texture_extent, "depth_texture");

            if self.viewport_texture.is_some() {
                self.render_state.egui_rpass.write().free_texture(&self.viewport_texture_id);
            }
            let texture_id = self.render_state.egui_rpass.write().register_native_texture(device, &texture.view, wgpu::FilterMode::Linear);

            self.viewport_texture_extent = texture_extent;
            self.viewport_texture = Some(texture);
            self.viewport_msaa_texture = Some(msaa_texture);
            self.viewport_depth_texture = Some(depth_texture);
            self.viewport_texture_id = texture_id;

            self.need_update = true;
        }
    }

    pub fn prepare_output(&mut self) {

        self.is_render_output = true;

        let device = &self.render_state.device;
        let _queue = &self.render_state.queue;

        let output_texture_extent = wgpu::Extent3d {
            width: (self.viewport_texture_extent.width * 2) as u32,
            height: (self.viewport_texture_extent.height * 2) as u32,
            depth_or_array_layers: 1,
        };

        let output_texture = Texture::create_texture(device, &output_texture_extent, 1);
        let msaa_output_texture = Texture::create_texture(device, &output_texture_extent, 4);
        let output_depth_texture = Texture::create_depth_texture(&device, &output_texture_extent, "depth_texture");

        self.output_texture_extent = output_texture_extent;
        self.output_texture = Some(output_texture);
        self.output_msaa_texture = Some(msaa_output_texture);
        self.output_depth_texture = Some(output_depth_texture);

    }

    pub fn output_png_after_render(&mut self, out_folder: String) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let buffer_dimensions = BufferDimensions::new(self.output_texture_extent.width as usize, self.output_texture_extent.height as usize);

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let command_buffer = {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            encoder.copy_texture_to_buffer(
                self.output_texture.as_ref().unwrap().texture.as_image_copy(),
                wgpu::ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(buffer_dimensions.padded_bytes_per_row as u32)
                                .unwrap(),
                        ),
                        rows_per_image: None,
                    },
                },
                self.output_texture_extent,
            );

            encoder.finish()
        };

        let index = queue.submit(Some(command_buffer));

        let mut output_path = out_folder;
        let formatted_time = Local::now().format("%Y.%m.%d_%H.%M.%S");
        let png_name = format!("/graphpu_render_{}_{}.png", formatted_time, self.render_frame_count);
        output_path += &png_name;
        pollster::block_on(create_png(output_path, device, output_buffer, &buffer_dimensions, index));
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
        self.bounding_buffer.destroy();
        self.tree_buffer.destroy();
        self.tree_node_buffer.destroy();
        self.tree_child_buffer.destroy();
    }


}

fn update_transform_matrix(queue: &Queue, camera: &mut Camera, render_uniform_buffer: &wgpu::Buffer, _viewport_size: glam::Vec2) {

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

fn get_tree_node_count(node_count: &u32) -> u32 {
    let mut tree_node_count = node_count * 2;
    while tree_node_count & (PARTICLES_PER_GROUP - 1) != 0 {
        tree_node_count += 1;
    }
    tree_node_count - 1
}