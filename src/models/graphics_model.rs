#![allow(dead_code)]
#![allow(unused_variables)]

use std::borrow::{Cow};
use std::mem;
use chrono::{Local, Utc};
use egui::{Ui, Vec2};
use glam::Vec3;
use wgpu::{Queue, ShaderModule, ComputePass};
use wgpu::util::DeviceExt;
use crate::models::data_model::GraphicsStatus;
use crate::models::graphics_lib::{BufferDimensions, Camera, Controls, RenderPipeline, Texture};

use rayon::prelude::*;
use crate::models::graphics_lib::bind_group_layout::BindGroupLayout;
use crate::models::graphics_lib::compute_shader::{ComputeBuffer, ComputeBufferType, ComputeKernel, ComputeShader};
use crate::models::graphics_lib::unifrom::{generate_uniforms, Uniforms};
use crate::utils::file::create_png;

use super::data_model::DataModel;
use bytemuck::{Pod, Zeroable};
use crate::utils::message::{message_error, message_warning};

// 须同步修改 Compute WGSL 中每一个函数的 @workgroup_size
// WGSL 中没有宏，不能使用类似 HLSL 的 #define 方法
const PARTICLES_PER_GROUP: u32 = 256;


// 须同步修改各个 WGSL 中的 Node struct
#[repr(C)]
pub struct Node {
    _position:      [f32; 3],
    _force:         [f32; 3],
    _prev_force:    [f32; 3],
    _mass:          u32,
}

#[repr(C)]
pub struct Bound {
    _bound_min: [f32; 3],
    _bound_max: [f32; 3],
}

#[repr(C)]
pub struct BHTree {
    _max_depth: u32,
    _bottom:    u32,
    _radius:    f32,
    _empty:     i32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct BHTreeNode {
    _position:  [f32; 4],
    _mass:      i32,
    _count:     i32,
    _start:     i32,
    _sort:      i32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct ComputeUniforms {
    frame_num:          u32,
    node_count:         u32,
    edge_count:         u32,
    edge_sort_count:    u32,
    tree_node_count:    u32,
    bounding_count:     u32,
    kernel_status_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct EdgeSort {
    _node:      [u32; 4],
    _force:     [f32; 4],
}

pub const KERNEL_STATUS_COUNT: usize = 5;
pub const KERNEL_NAMES: [&str; KERNEL_STATUS_COUNT] = [
    "attractive_force",
    "tree_building",
    "summarization",
    "sort",
    "electron_force",
];

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
    pub is_hover_graphics_view: bool,
    pub compute_render_state: egui_wgpu::RenderState,
    pub graphics_resources: GraphicsResources,
}

impl GraphicsModel {

    // 初始化计算 Model，传入 eframe 初始化中的 render_state
    // 仅启动时调用一次
    pub fn init(cc: &eframe::CreationContext) -> Self {
        Self {
            is_computing: false,
            is_dispatching: false,
            is_hover_graphics_view: false,
            compute_render_state: cc.wgpu_render_state.as_ref().unwrap().clone(),
            graphics_resources: GraphicsResources::new(cc.wgpu_render_state.as_ref().unwrap().clone()),
        }
    }

    // 重置计算 Model，dispose 并删除计算资源
    pub fn reset(&mut self) {
        self.graphics_resources.dispose();
    }
}

impl GraphicsModel {

    // 切换是否持续计算
    // 仅对 ComputeMethodType::Continuous 生效
    pub fn switch_computing(&mut self) {
        self.is_computing = !self.is_computing;
        if self.is_computing { self.cancel_error_state() }
    }

    // 设置是否持续计算
    // 仅对 ComputeMethodType::Continuous 生效
    pub fn set_computing(&mut self, state: bool) {
        self.is_computing = state;
        if state { self.cancel_error_state() }
    }

    // 设置下一帧是否 Dispatch
    // 仅对 ComputeMethodType::OneStep 生效
    pub fn set_dispatching(&mut self, state: bool) {
        self.is_dispatching = state;
    }

    pub fn cancel_error_state(&mut self) {
        if self.graphics_resources.graph_resources.is_some() {
            self.graphics_resources.graph_resources.as_mut().unwrap().is_kernel_error = false
        }
    }

    pub fn render_output(&mut self, out_folder:String) {
        let graphics_resources = &mut self.graphics_resources;

        graphics_resources.prepare_output();
        graphics_resources.render();
        graphics_resources.output_png_after_render(out_folder.to_owned());

    }
}

pub struct RenderOptions {
    pub is_rendering_node: bool,
    pub is_rendering_edge: bool,
    pub is_rendering_axis: bool,
    pub is_rendering_bounding_box: bool,
    pub is_showing_debug:  bool,
}

pub struct ComputePipelines {
    gen_node:              ComputeKernel,
    cal_mass:              ComputeKernel,
    cal_gravity:           ComputeKernel,
    attractive_force:      ComputeKernel,
    reduction_bounding:    ComputeKernel,
    reduction_bounding_2:  ComputeKernel,
    bounding_box:          ComputeKernel,
    clear_1:               ComputeKernel,
    tree_building:         ComputeKernel,
    clear_2:               ComputeKernel,
    summarization:         ComputeKernel,
    sort:                  ComputeKernel,
    electron_force:        ComputeKernel,
    compute:               ComputeKernel,
    displacement:          ComputeKernel,
    randomize:             ComputeKernel,
    copy:                  ComputeKernel,
    cal_depth:             ComputeKernel,
    sort_by_depth:         ComputeKernel,
}

pub enum CastType {
    Node,
    Edge
}

// 绘图资源 Model，存放和计算和绘图相关的一切资源
pub struct GraphicsResources {

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

    cast_texture_extent:          wgpu::Extent3d,
    cast_texture:                 Option<Texture>,
    cast_depth_texture:           Option<Texture>,

    axis_render_pipeline:           wgpu::RenderPipeline,

    // 相机
    camera:                         Camera,
    pub control:                    Controls,

    // Buffers
    quad_buffer:                    wgpu::Buffer,
    render_uniform_buffer:          wgpu::Buffer,
    bounding_box_vertex_buffer:     wgpu::Buffer,
    bounding_box_index_buffer:      wgpu::Buffer,

    render_uniform_bind_group_layout: wgpu::BindGroupLayout,
    render_uniform_bind_group:      wgpu::BindGroup,

    pub compute_frame_count:        u32,                      // 帧计数器
    pub render_frame_count:         u32,                      // 帧计数器
    last_time:                      i64,                      // 上次记录时间
    last_frame:                     u32,                      // 上次记录帧数
    pub frames_per_second:          f64,                      // FPS

    pub render_options:             RenderOptions,
    pub need_update:                bool,

    pub cast_type:                  Option<CastType>,
    pub cast_value:                 u32,

    pub shaders:                    Vec<ShaderModule>,
    pub compute_shader:             ComputeShader,

    pub graph_resources:            Option<GraphResources>,
}

pub struct GraphResources {

    // 包含 Node / Edge Count
    pub status:                     GraphicsStatus,

    uniform_buffer:                 wgpu::Buffer,
    node_buffer:                    wgpu::Buffer,
    node_copy_buffer:               wgpu::Buffer,
    node_edge_sort_range_buffer:    wgpu::Buffer,
    edge_buffer:                    wgpu::Buffer,
    edge_sort_node_buffer:          wgpu::Buffer,
    edge_sort_dir_buffer:           wgpu::Buffer,
    bounding_buffer:                wgpu::Buffer,
    tree_buffer:                    wgpu::Buffer,
    tree_node_buffer:               wgpu::Buffer,
    tree_child_buffer:              wgpu::Buffer,
    depth_sort_buffer:              wgpu::Buffer,
    depth_sort_param_buffer:        wgpu::Buffer,
    kernel_status_buffer:           wgpu::Buffer,

    // Bind Group
    node_render_bind_group:         wgpu::BindGroup,
    edge_render_bind_group:         wgpu::BindGroup,
    bounding_box_render_bind_group: wgpu::BindGroup,


    pub debugger:                   GraphicsDebugger,
    pub buffer_bytes:               Option<Vec<u8>>,

    node_render_pipeline:           wgpu::RenderPipeline,
    node_cast_render_pipeline:      wgpu::RenderPipeline,
    edge_render_pipeline:           wgpu::RenderPipeline,
    edge_cast_render_pipeline:      wgpu::RenderPipeline,
    bounding_box_render_pipeline:   wgpu::RenderPipeline,

    // 计算管线
    pub kernel_status_codes:        Vec<i32>,
    pub is_kernel_error:            bool,

    // 线程组数 = 线程数 / 每组线程数
    node_work_group_count:          u32,
    edge_work_group_count:          u32,
    edge_sort_work_group_count:     u32,
    tree_node_work_group_count:     u32,
    step_work_group_count:          u32,
    bb_work_group_count:            u32,
}

pub struct GraphicsDebugger {
    debug_buffer:    wgpu::Buffer,
    buffer_size:     u32,
}

impl GraphicsResources {

    // 在导入数据后调用的方法，初始化计算和绘图的资源
    pub fn new(render_state: egui_wgpu::RenderState) -> Self {


        // 从 render_state 中获取 wgpu 的 device 和 queue
        let device = &render_state.device;
        let _queue = &render_state.queue;

        // 从文件中创建 Shader

        let shader_files = [
            include_str!("../assets/shaders/S_node.wgsl"),
            include_str!("../assets/shaders/S_edge.wgsl"),
            include_str!("../assets/shaders/S_axis.wgsl"),
            include_str!("../assets/shaders/S_bounding_box.wgsl"),
            include_str!("../assets/shaders/CS_graph_solver.wgsl"),
        ];

        let mut shaders = shader_files.par_iter().map(|shader_file| unsafe {

            let shader = device.create_shader_module_unchecked(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(shader_file)),
            });

            shader

        }).collect::<Vec<ShaderModule>>();

        let compute_shader = shaders.pop().unwrap();

        let render_uniform_bind_group_layout =      BindGroupLayout::create_render_uniform_bind_group_layout(device).bing_group_layout;

        let axis_shader = &shaders[2];

        let axis_render_pipeline = RenderPipeline::create_axis_render_pipeline(
            device,
            &[&render_uniform_bind_group_layout],
            axis_shader
        ).render_pipeline;



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

        let vertex_buffer_data: &[f32] = &[
                -1.0, -1.0, -1.0,
                 1.0, -1.0, -1.0,
                -1.0,  1.0, -1.0,
                 1.0,  1.0, -1.0,
                -1.0, -1.0,  1.0,
                 1.0, -1.0,  1.0,
                -1.0,  1.0,  1.0,
                 1.0,  1.0,  1.0,
            ];

        let index_buffer_data: &[u16] = &[
            0, 1, 1, 3, 3, 2, 2, 0,
            0, 4, 1, 5, 2, 6, 3, 7,
            4, 5, 5, 7, 7, 6, 6, 4,
        ];

        let bounding_box_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_buffer_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bounding_box_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_buffer_data),
            usage: wgpu::BufferUsages::INDEX,
        });


        let camera = Camera::from(Vec3::new(3.0, 3.0, 6.0));
        let control = Controls::new();

        //
        let render_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Render Uniform Buffer"),
            size: mem::size_of::<Uniforms>() as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
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

        let compute_shader = ComputeShader {
            shader: compute_shader,
            device: device.clone(),
            kernels: Default::default()
        };


        let graphics_resources = GraphicsResources {
            render_state,
            viewport_depth_texture: None,
            output_depth_texture: None,
            cast_texture_extent: Default::default(),
            cast_texture: None,
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
            quad_buffer,
            bounding_box_vertex_buffer,
            bounding_box_index_buffer,
            render_uniform_buffer,
            render_uniform_bind_group,
            axis_render_pipeline,
            compute_frame_count: 0,
            render_frame_count: 0,
            last_time: 0,
            last_frame: 0,
            frames_per_second: 0.0,
            render_options: RenderOptions {
                is_rendering_node: true,
                is_rendering_edge: true,
                is_rendering_axis: false,
                is_rendering_bounding_box: false,
                is_showing_debug: false
            },
            need_update: true,
            cast_depth_texture: None,
            cast_type: None,
            cast_value: 0,
            shaders,
            compute_shader,
            graph_resources: None,
            render_uniform_bind_group_layout
        };

        graphics_resources

    }

    pub fn init_data(&mut self, render_state: egui_wgpu::RenderState, model: &mut DataModel) {

        let device = &render_state.device;
        let _queue = &render_state.queue;

        let status = model.status.clone();


        // 从 Graphics Model 中获取 Node 和 Edge 的数量
        let node_count = model.status.node_count as u32;
        let edge_count = model.status.edge_count as u32;
        let edge_sort_count = (model.status.edge_count * 2) as u32;
        let tree_node_count = get_tree_node_count(&node_count);

        // Node 和 Edge 结构体的占内存大小，用于计算 Buffer 长度
        let node_struct_size = mem::size_of::<Node>();

        let shaders = &self.shaders;

        let node_shader = &shaders[0];
        let edge_shader = &shaders[1];
        let axis_shader = &shaders[2];
        let bounding_box_shader = &shaders[3];


        // Bind Group Layout
        // Bind Group 的布局，允许在布局不变的情况下更换 Bind Group 绑定的 Buffer

        let node_render_bind_group_layout =         BindGroupLayout::create_node_render_bind_group_layout(device).bing_group_layout;
        let edge_render_bind_group_layout =         BindGroupLayout::create_edge_render_bind_group_layout(device).bing_group_layout;
        let bounding_box_render_bind_group_layout = BindGroupLayout::create_bounding_box_render_bind_group_layout(device).bing_group_layout;


        // Render Pipeline
        // 渲染管线

        let node_render_pipeline = RenderPipeline::create_node_render_pipeline(
            device,
            &[&self.render_uniform_bind_group_layout, &node_render_bind_group_layout],
            node_shader,
            false,
        ).render_pipeline;

        let node_cast_render_pipeline = RenderPipeline::create_node_render_pipeline(
            device,
            &[&self.render_uniform_bind_group_layout, &node_render_bind_group_layout],
            node_shader,
            true,
        ).render_pipeline;

        let edge_render_pipeline = RenderPipeline::create_edge_render_pipeline(
            device,
            &[&self.render_uniform_bind_group_layout, &edge_render_bind_group_layout],
            edge_shader,
            false,
        ).render_pipeline;

        let edge_cast_render_pipeline = RenderPipeline::create_edge_render_pipeline(
            device,
            &[&self.render_uniform_bind_group_layout, &edge_render_bind_group_layout],
            edge_shader,
            true,
        ).render_pipeline;

        let bounding_box_render_pipeline = RenderPipeline::create_bounding_box_render_pipeline(
            device,
            &[&self.render_uniform_bind_group_layout, &bounding_box_render_bind_group_layout],
            bounding_box_shader
        ).render_pipeline;


        // 计算线程组数
        // 线程组数 = 线程数 / 每组线程数（取整）

        let node_work_group_count =
            ((node_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let edge_work_group_count =
            ((edge_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let edge_sort_work_group_count =
            ((edge_sort_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let tree_node_work_group_count =
            (tree_node_count as f32 / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        let step_work_group_count =
            (std::cmp::min(node_count, 16384) as f32 / PARTICLES_PER_GROUP as f32).ceil() as u32;

        let bb_work_group_count =
            ((node_work_group_count as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        // Buffer 创建

        let compute_uniform = ComputeUniforms {
            frame_num: 0,
            node_count,
            edge_count,
            edge_sort_count,
            tree_node_count,
            bounding_count: node_work_group_count,
            kernel_status_count: KERNEL_NAMES.len() as u32,
        };

        // 创建 Uniform Buffer
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[compute_uniform]),
            usage: wgpu::BufferUsages::UNIFORM |
                wgpu::BufferUsages::COPY_DST,
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

        // 计算对齐后的 Node Buffer 大小
        let node_copy_buffer_size = node_count * 3 * 4;

        // 指定大小，创建 Node Buffer，不初始化数据
        let node_copy_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Copy Buffer"),
            size: node_copy_buffer_size as _,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });

        let node_edge_sort_range_buffer_size = node_count * 2 * 4;

        let node_edge_sort_range_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Node Edge Sort Range Buffer"),
            size: node_edge_sort_range_buffer_size as _,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });

        let spring_force_buffer_size = node_count * 4 * 4;

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

        let depth_sort_buffer_size = node_count * 2 * 4;

        let depth_sort_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Depth Sort Buffer"),
            size: depth_sort_buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let depth_sort_param_buffer_size = 2 * 4;

        let depth_sort_param_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Depth Sort Param Buffer"),
            size: depth_sort_param_buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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

        let edge_sort_node_buffer_size = edge_sort_count * 2 * 4;

        let edge_sort_node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Edge Sort Node Buffer"),
            size: edge_sort_node_buffer_size as _,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });

        let edge_sort_dir_buffer_size = edge_sort_count * 4 * 4;

        let edge_sort_dir_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Edge Sort Dir Buffer"),
            size: edge_sort_dir_buffer_size as _,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
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
        let tree_node_buffer_size = 4 * ((tree_node_count + 1) * 8);
        let tree_node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tree Node Buffer"),
            size: tree_node_buffer_size as _,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });

        // Tree Child Buffer
        let tree_child_buffer_size = 4 * ((tree_node_count + 1) * 8);
        let tree_child_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Tree Child Buffer"),
            size: tree_child_buffer_size as _,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });

        // 清空 Model 中的 Source Target List
        model.clear_source_target_list();


        // Bind Group
        // 用于绑定 Buffer 与 Bind Group Layout，后连接 Pipeline Layout 和 Pipeline
        // 需与 Bind Group Layout 保持索引和容量一致

        // Node Render Bind Group
        let node_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &node_render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: node_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: depth_sort_buffer.as_entire_binding(),
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

        // Edge Render Bind Group
        let bounding_box_render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bounding_box_render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bounding_buffer.as_entire_binding(),
                },
            ],
            label: None,
        });


        let kernel_status_buffer_size = KERNEL_STATUS_COUNT * 4;
        let kernel_status_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Kernel Status Buffer"),
            size: kernel_status_buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false
        });


        let debug_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Buffer"),
            size: kernel_status_buffer_size as _,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let debugger = GraphicsDebugger {
            debug_buffer,
            buffer_size: kernel_status_buffer_size as _,
        };

        let graph_compute = &mut self.compute_shader;

        graph_compute.create_compute_kernel("init_kernel_status", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("gen_node", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 3,
                buffer_type: ComputeBufferType::Storage,
                buffer: spring_force_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_edge_sort_range_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 14,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_edge_sort_range_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("cal_mass", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 2,
                buffer_type: ComputeBufferType::StorageReadOnly,
                buffer: edge_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("cal_gravity_force", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
        ]);

        graph_compute.create_compute_kernel("prepare_edge_sort", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 12,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_dir_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 2,
                buffer_type: ComputeBufferType::StorageReadOnly,
                buffer: edge_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("sort_edge", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 12,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_dir_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 9,
                buffer_type: ComputeBufferType::Uniform,
                buffer: depth_sort_param_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("compute_node_edge_sort_range", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 12,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_dir_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 14,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_edge_sort_range_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("compute_node_edge_sort_range_2", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 12,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_dir_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 14,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_edge_sort_range_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("spring_force_reduction", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 3,
                buffer_type: ComputeBufferType::Storage,
                buffer: spring_force_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 12,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_dir_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 14,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_edge_sort_range_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("spring_force", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 3,
                buffer_type: ComputeBufferType::Storage,
                buffer: spring_force_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 13,
                buffer_type: ComputeBufferType::Storage,
                buffer: edge_sort_dir_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 14,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_edge_sort_range_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("reduction_bounding", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 4,
                buffer_type: ComputeBufferType::Storage,
                buffer: bounding_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("reduction_bounding_2", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 4,
                buffer_type: ComputeBufferType::Storage,
                buffer: bounding_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("bounding_box", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 4,
                buffer_type: ComputeBufferType::Storage,
                buffer: bounding_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 5,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 6,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_node_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("clear_1", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 7,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_child_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("tree_building", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 5,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 6,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 7,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_child_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("clear_2", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 6,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_node_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("summarization", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 5,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 6,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 7,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_child_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("sort", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 5,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 6,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 7,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_child_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("electron_force", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 5,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 6,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 7,
                buffer_type: ComputeBufferType::Storage,
                buffer: tree_child_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("main", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 3,
                buffer_type: ComputeBufferType::Storage,
                buffer: spring_force_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("displacement", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("randomize", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("copy", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 15,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_copy_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("cal_depth", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 1,
                buffer_type: ComputeBufferType::Storage,
                buffer: node_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 8,
                buffer_type: ComputeBufferType::Storage,
                buffer: depth_sort_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 10,
                buffer_type: ComputeBufferType::Uniform,
                buffer: self.render_uniform_buffer.as_entire_binding(),
            },
        ]);
        graph_compute.create_compute_kernel("sort_by_depth", vec![
            ComputeBuffer {
                binding: 0,
                buffer_type: ComputeBufferType::Uniform,
                buffer: uniform_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 8,
                buffer_type: ComputeBufferType::Storage,
                buffer: depth_sort_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 9,
                buffer_type: ComputeBufferType::Uniform,
                buffer: depth_sort_param_buffer.as_entire_binding(),
            },
            ComputeBuffer {
                binding: 11,
                buffer_type: ComputeBufferType::Storage,
                buffer: kernel_status_buffer.as_entire_binding(),
            },
        ]);

        let graph_resources = GraphResources {
            status,
            uniform_buffer,
            node_buffer,
            node_copy_buffer,
            node_edge_sort_range_buffer,
            edge_buffer,
            edge_sort_node_buffer,
            edge_sort_dir_buffer,
            bounding_buffer,
            tree_buffer,
            tree_node_buffer,
            tree_child_buffer,
            depth_sort_buffer,
            depth_sort_param_buffer,
            kernel_status_buffer,
            node_render_bind_group,
            edge_render_bind_group,
            bounding_box_render_bind_group,
            debugger,
            buffer_bytes: None,
            node_render_pipeline,
            node_cast_render_pipeline,
            edge_render_pipeline,
            edge_cast_render_pipeline,
            bounding_box_render_pipeline,
            kernel_status_codes: vec![-1; KERNEL_STATUS_COUNT],
            is_kernel_error: false,
            node_work_group_count,
            edge_work_group_count,
            edge_sort_work_group_count,
            tree_node_work_group_count,
            step_work_group_count,
            bb_work_group_count
        };

        self.graph_resources = Some(graph_resources);

        self.gen_node();
        self.prepare_cast();
        self.need_update = true;

    }

    pub fn compute(&mut self) {

        if self.graph_resources.is_none() { return; }
        let graph_resources = self.graph_resources.as_ref().unwrap();

        let device = self.render_state.device.clone();
        let queue = self.render_state.queue.clone();

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.push_debug_group("compute render movement");
        {
            // compute pass
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            Self::dispatch_compute_kernel(&self, &mut cpass, "init_kernel_status", 1);

            Self::dispatch_compute_kernel(&self, &mut cpass, "cal_gravity_force", graph_resources.node_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "spring_force_reduction", graph_resources.edge_sort_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "spring_force", graph_resources.node_work_group_count);

            Self::calc_bounding_box(&self, &mut cpass);

            Self::dispatch_compute_kernel(&self, &mut cpass, "bounding_box", 1);

            Self::dispatch_compute_kernel(&self, &mut cpass, "clear_1", graph_resources.tree_node_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "tree_building", graph_resources.step_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "clear_2", graph_resources.tree_node_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "summarization", graph_resources.step_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "sort", graph_resources.step_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "electron_force", graph_resources.step_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "main", graph_resources.node_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "displacement", graph_resources.node_work_group_count);

        }

        // Self::debug(self);

        // queue.submit(Some(command_encoder.finish()));
        self.compute_frame_count += 1;
        // device.poll(wgpu::Maintain::Wait);

        let graph_resources = self.graph_resources.as_mut().unwrap();

        let debugger = &mut graph_resources.debugger;
        command_encoder.copy_buffer_to_buffer(&graph_resources.kernel_status_buffer, 0, &debugger.debug_buffer, 0, debugger.buffer_size as _);
        queue.submit(Some(command_encoder.finish()));

        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        let buffer_slice = debugger.debug_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::Wait);

        pollster::block_on(async {
            if let Some(Ok(())) = receiver.receive().await {
                let data = buffer_slice.get_mapped_range();
                let result: Vec<i32> = bytemuck::cast_slice(&data).to_vec();

                let content = format!("{:?}", &result);

                if *result.par_iter().max().unwrap() > 0 {
                    message_error("Kernel Error", content.as_str());
                    graph_resources.is_kernel_error = true;
                } else {
                    graph_resources.is_kernel_error = false;
                    if *result.par_iter().min().unwrap() < 0 {
                        message_warning("Kernel Warning", content.as_str());
                    }
                }

                if result.len() == KERNEL_STATUS_COUNT {
                    graph_resources.kernel_status_codes = result;
                }

                drop(data);
                debugger.debug_buffer.unmap();


            } else {
                panic!("failed to run compute on gpu!")
            }
        });

    }

    pub fn debug<'a>(&'a mut self) {


        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let graph_resources = self.graph_resources.as_ref().unwrap();

        let debug_buffer_size = graph_resources.status.node_count * 3 * 4;
        let debug_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Debug Buffer"),
            size: debug_buffer_size as _,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut cpass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            Self::dispatch_compute_kernel(&self, &mut cpass, "copy", graph_resources.node_work_group_count);
        }
        queue.submit(Some(command_encoder.finish()));

        let mut command_encoder_2 =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        command_encoder_2.copy_buffer_to_buffer(&graph_resources.node_copy_buffer, 0, &debug_buffer, 0, debug_buffer_size as _);
        queue.submit(Some(command_encoder_2.finish()));

        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        let buffer_slice = debug_buffer.slice(..);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        device.poll(wgpu::Maintain::Wait);

        let graph_resources = self.graph_resources.as_mut().unwrap();
        pollster::block_on(async {
            if let Some(Ok(())) = receiver.receive().await {
                let data = buffer_slice.get_mapped_range();
                let result: Vec<u8> = bytemuck::cast_slice(&data).to_vec();

                // let content = format!("{:?}", &result);
                // println!("{}", content);
                // println!("{}", result.len());
                graph_resources.buffer_bytes = Some(result);

            } else {
                panic!("failed to run compute on gpu!")
            }
        });

    }

    pub fn calc_bounding_box<'a>(&'a self, cpass: &mut ComputePass<'a>) {
        let graph_resources = self.graph_resources.as_ref().unwrap();
        Self::dispatch_compute_kernel(&self, cpass, "reduction_bounding", graph_resources.node_work_group_count);


        let mut bound_range = graph_resources.bb_work_group_count;

        loop {
            Self::dispatch_compute_kernel(&self, cpass, "reduction_bounding_2", bound_range);

            if bound_range <= 1 { break; }
            bound_range = ((bound_range as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;

        }
    }

    pub fn dispatch_compute_kernel<'a>(&'a self, cpass: &mut ComputePass<'a>, kernel_name: &'a str, work_group_count: u32) {
        cpass.set_pipeline(&self.compute_shader.kernels.get(kernel_name).unwrap().compute_pipeline);
        cpass.set_bind_group(0, &self.compute_shader.kernels.get(kernel_name).unwrap().bind_group, &[]);
        cpass.dispatch_workgroups(work_group_count, 1, 1);
    }


    pub fn gen_node(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let graph_resources = &self.graph_resources.as_ref().unwrap();

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.push_debug_group("Gen Node");
        {
            // compute pass
            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            Self::dispatch_compute_kernel(&self, &mut cpass, "gen_node", graph_resources.node_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "cal_mass", graph_resources.edge_work_group_count);

            Self::calc_bounding_box(&self, &mut cpass);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));


        // calc depth
        let command_buffer = {
            let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

                Self::dispatch_compute_kernel(&self, &mut cpass, "prepare_edge_sort", graph_resources.edge_work_group_count);

            }
            command_encoder.finish()
        };
        queue.submit(Some(command_buffer));

        // sort
        {
            let mut dim = 2;
            while dim < (graph_resources.status.edge_count * 2) * 2 {
                let mut block_count = dim >> 1;
                queue.write_buffer(&graph_resources.depth_sort_param_buffer, 0, bytemuck::cast_slice(&[dim as u32]));
                while block_count > 0 {
                    queue.write_buffer(&graph_resources.depth_sort_param_buffer, 4, bytemuck::cast_slice(&[block_count as u32]));
                    let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                    {
                        let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                        Self::dispatch_compute_kernel(&self, &mut cpass, "sort_edge", graph_resources.edge_sort_work_group_count);
                    }
                    queue.submit(Some(command_encoder.finish()));
                    block_count = block_count >> 1;
                }
                dim = dim << 1;
            }
        };

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        command_encoder.push_debug_group("compute_node_edge_sort_range");
        {
            // compute pass
            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            Self::dispatch_compute_kernel(&self, &mut cpass, "compute_node_edge_sort_range", graph_resources.edge_sort_work_group_count);
            Self::dispatch_compute_kernel(&self, &mut cpass, "compute_node_edge_sort_range_2", graph_resources.edge_sort_work_group_count);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));

        self.compute_frame_count += 1;
    }

    pub fn randomize(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let graph_resources = self.graph_resources.as_ref().unwrap();

        let mut command_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        queue.write_buffer(&graph_resources.uniform_buffer, 0, bytemuck::cast_slice(&[self.compute_frame_count as u32]));
        command_encoder.push_debug_group("randomize render position");
        {
            // compute pass
            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

            Self::dispatch_compute_kernel(&self, &mut cpass, "randomize", graph_resources.node_work_group_count);

            Self::dispatch_compute_kernel(&self, &mut cpass, "copy", graph_resources.node_work_group_count);

            Self::calc_bounding_box(&self, &mut cpass);
        }
        command_encoder.pop_debug_group();
        queue.submit(Some(command_encoder.finish()));
        self.compute_frame_count += 1;
    }

    pub fn render(&mut self) {

        let is_graph_resources = self.graph_resources.is_some();

        let new_time = Utc::now().timestamp_millis();
        let delta_time = new_time - self.last_time;
        if delta_time >= 300 {
            self.frames_per_second = (1000 * (self.render_frame_count - self.last_frame)) as f64 / delta_time as f64;
            self.last_time = new_time;
            self.last_frame = self.render_frame_count;
        }

        self.render_frame_count += 1u32;

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let is_render_output = self.is_render_output;
        self.is_render_output = false;

        let
            (
                resolve_target,
                view,
                depth_view
            )
            = if !is_render_output
        {
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
                load: wgpu::LoadOp::Clear(if !is_render_output { wgpu::Color::TRANSPARENT } else { wgpu::Color::BLACK }),
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
        };

        if self.render_options.is_rendering_node && is_graph_resources {
            let graph_resources = self.graph_resources.as_ref().unwrap();

            // calc depth
            let command_buffer = {
                let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });

                    Self::dispatch_compute_kernel(&self, &mut cpass, "cal_depth", graph_resources.node_work_group_count);

                }
                command_encoder.finish()
            };
            queue.submit(Some(command_buffer));

            // sort
            {
                let mut dim = 2;
                while dim < graph_resources.status.node_count * 2 {
                    let mut block_count = dim >> 1;
                    queue.write_buffer(&graph_resources.depth_sort_param_buffer, 0, bytemuck::cast_slice(&[dim as u32]));
                    while block_count > 0 {
                        queue.write_buffer(&graph_resources.depth_sort_param_buffer, 4, bytemuck::cast_slice(&[block_count as u32]));
                        let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                        {
                            let mut cpass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                            Self::dispatch_compute_kernel(&self, &mut cpass, "sort_by_depth", graph_resources.node_work_group_count);
                        }
                        queue.submit(Some(command_encoder.finish()));
                        block_count = block_count >> 1;
                    }
                    dim = dim << 1;
                }
            };

        }

        // render
        let command_buffer = {
            let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);
                if !is_render_output {
                    if self.render_options.is_rendering_axis {
                        rpass.set_pipeline(&self.axis_render_pipeline);
                        rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                        rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                        rpass.draw(0..4, 0..2);
                    }
                }
                if is_graph_resources {
                    let graph_resources = self.graph_resources.as_ref().unwrap();

                    if !is_render_output {

                        if self.render_options.is_rendering_bounding_box {
                            rpass.set_pipeline(&graph_resources.bounding_box_render_pipeline);
                            rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                            rpass.set_bind_group(1, &graph_resources.bounding_box_render_bind_group, &[]);
                            rpass.set_index_buffer(self.bounding_box_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                            rpass.set_vertex_buffer(0, self.bounding_box_vertex_buffer.slice(..));
                            rpass.draw_indexed(0..24, 0, 0..1);
                        }
                    }

                    if self.render_options.is_rendering_node {
                        rpass.set_pipeline(&graph_resources.node_render_pipeline);
                        rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                        rpass.set_bind_group(1, &graph_resources.node_render_bind_group, &[]);
                        rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                        rpass.draw(0..4, 0..graph_resources.status.node_count as u32);
                    }


                    if self.render_options.is_rendering_edge {
                        rpass.set_pipeline(&graph_resources.edge_render_pipeline);
                        rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                        rpass.set_bind_group(1, &graph_resources.edge_render_bind_group, &[]);
                        rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                        rpass.draw(0..4, 0..graph_resources.status.edge_count as u32);
                    }
                }
            }
            command_encoder.finish()
        };
        queue.submit(Some(command_buffer));

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

            let texture = Texture::create_texture(device, &texture_extent, 1, false);
            let msaa_texture = Texture::create_texture(device, &texture_extent, 4, false);
            let depth_texture = Texture::create_depth_texture(&device, &texture_extent, "depth_texture", true);

            if self.viewport_texture.is_some() {
                self.render_state.renderer.write().free_texture(&self.viewport_texture_id);
            }
            let texture_id = self.render_state.renderer.write().register_native_texture(device, &texture.view, wgpu::FilterMode::Linear);

            self.viewport_texture_extent = texture_extent;
            self.viewport_texture = Some(texture);
            self.viewport_msaa_texture = Some(msaa_texture);
            self.viewport_depth_texture = Some(depth_texture);
            self.viewport_texture_id = texture_id;

            self.need_update = true;
        }
    }


    pub fn prepare_cast(&mut self) {

        let device = &self.render_state.device;
        let _queue = &self.render_state.queue;

        let cast_texture_extent = wgpu::Extent3d {
            width: 1 as _,
            height: 1 as _,
            depth_or_array_layers: 1,
        };

        let cast_texture = Texture::create_texture(device, &cast_texture_extent, 1, true);
        let cast_depth_texture = Texture::create_depth_texture(&device, &cast_texture_extent, "cast_depth_texture", false);

        self.cast_texture_extent = cast_texture_extent;
        self.cast_texture = Some(cast_texture);
        self.cast_depth_texture = Some(cast_depth_texture);

    }


    pub fn render_cast(&mut self) {

        if self.graph_resources.is_none() { return; }
        let graph_resources = self.graph_resources.as_ref().unwrap();

        if self.control.pointer_pos.is_none() { return; }

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let view = &self.cast_texture.as_ref().unwrap().view;
        let depth_view = &self.cast_depth_texture.as_ref().unwrap().view;

        let color_attachment = Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: true,
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
        };

        // render
        let command_buffer = {
            let mut command_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut rpass = command_encoder.begin_render_pass(&render_pass_descriptor);

                if self.render_options.is_rendering_node {
                    rpass.set_pipeline(&graph_resources.node_cast_render_pipeline);
                    rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                    rpass.set_bind_group(1, &graph_resources.node_render_bind_group, &[]);
                    rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                    rpass.draw(0..4, 0..graph_resources.status.node_count as u32);
                }

                if self.render_options.is_rendering_edge {
                    rpass.set_pipeline(&graph_resources.edge_cast_render_pipeline);
                    rpass.set_bind_group(0, &self.render_uniform_bind_group, &[]);
                    rpass.set_bind_group(1, &graph_resources.edge_render_bind_group, &[]);
                    rpass.set_vertex_buffer(0, self.quad_buffer.slice(..));
                    rpass.draw(0..4, 0..graph_resources.status.edge_count as u32);
                }
            }
            command_encoder.finish()
        };
        queue.submit(Some(command_buffer));


        self.copy_cast_data();
    }


    pub fn copy_cast_data(&mut self) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let buffer_dimensions = BufferDimensions::new(
            self.cast_texture_extent.width as usize,
            self.cast_texture_extent.height as usize,
            16
        );

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (buffer_dimensions.padded_bytes_per_row * buffer_dimensions.height) as _,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let command_buffer = {
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            encoder.copy_texture_to_buffer(
                self.cast_texture.as_ref().unwrap().texture.as_image_copy(),
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
                self.cast_texture_extent,
            );

            encoder.finish()
        };

        let submission_index = queue.submit(Some(command_buffer));

        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        let buffer_slice = output_buffer.slice(..16);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

        device.poll(wgpu::Maintain::WaitForSubmissionIndex(submission_index));

        pollster::block_on(async {
            if let Some(Ok(())) = receiver.receive().await {
                let data = buffer_slice.get_mapped_range();
                let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();

                match result[0] {
                    1 => {
                        self.cast_type = Some(CastType::Node);
                        self.cast_value = result[1];
                    }
                    2 => {
                        self.cast_type = Some(CastType::Edge);
                        self.cast_value = result[1];
                    }
                    _ => {
                        self.cast_type = None;
                        self.cast_value = 0;
                    }
                }

                let content = format!("{:?}", &result);
                // println!("{}", content);
                // println!("{}", result.len());

            } else {
                panic!("failed to run compute on gpu!")
            }
        });

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

        let output_texture = Texture::create_texture(device, &output_texture_extent, 1, false);
        let msaa_output_texture = Texture::create_texture(device, &output_texture_extent, 4, false);
        let output_depth_texture = Texture::create_depth_texture(&device, &output_texture_extent, "depth_texture", true);

        self.output_texture_extent = output_texture_extent;
        self.output_texture = Some(output_texture);
        self.output_msaa_texture = Some(msaa_output_texture);
        self.output_depth_texture = Some(output_depth_texture);

    }

    pub fn output_png_after_render(&mut self, out_folder: String) {

        let device = &self.render_state.device;
        let queue = &self.render_state.queue;

        let buffer_dimensions = BufferDimensions::new(
            self.output_texture_extent.width as usize,
            self.output_texture_extent.height as usize,
            4
        );

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

    pub fn update_control(&mut self, ui: &mut Ui, is_hover: bool, scale: f32) {

        self.control.update_interaction(ui, is_hover);
        self.control.update_camera(ui, &mut self.camera);

        if self.control.is_update {
            self.need_update = true;
        }

        if self.need_update || self.control.is_pointer_update {

            let pointer_pos = if let Some(pos) = self.control.pointer_pos {
                Some(glam::Vec2::new(pos.x * scale, pos.y * scale) )
            } else {
                None
            };
            update_render_uniforms(
                &self.render_state.queue,
                &mut self.camera,
                &self.render_uniform_buffer,
                glam::Vec2::new(self.viewport_size.x, self.viewport_size.y),
                pointer_pos,
                self.control.is_pointer_update
            );
        }

        self.control.is_update = false;

    }

    pub fn dispose(&mut self) {

        let graph_resources = self.graph_resources.as_mut().unwrap();
        graph_resources.node_buffer.destroy();
        graph_resources.edge_buffer.destroy();
        graph_resources.bounding_buffer.destroy();
        graph_resources.tree_buffer.destroy();
        graph_resources.tree_node_buffer.destroy();
        graph_resources.tree_child_buffer.destroy();
        graph_resources.depth_sort_buffer.destroy();
        graph_resources.depth_sort_param_buffer.destroy();

        self.graph_resources = None;
        self.need_update = true;
        // self.render();
    }


}

fn update_render_uniforms(
    queue: &Queue,
    camera: &mut Camera,
    render_uniform_buffer: &wgpu::Buffer,
    viewport_size: glam::Vec2,
    pointer_pos: Option<glam::Vec2>,
    is_pointer_update: bool,
) {

    let pos = if let Some(pos) = pointer_pos {
        pos
    } else {
        glam::Vec2::ZERO
    };

    if camera.is_updated {

        camera.update_projection_matrix();

        let uniform = generate_uniforms(camera, viewport_size, pos);

        queue.write_buffer(&render_uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));
        camera.is_updated = false;

    } else if pointer_pos.is_some() && is_pointer_update {

        let uniform = generate_uniforms(camera, viewport_size, pos);
        queue.write_buffer(&render_uniform_buffer, 0, bytemuck::cast_slice(&[uniform]));

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

fn get_tree_node_count(node_count: &u32) -> u32 {
    let mut tree_node_count = node_count * 2;
    // println!("{}", node_count);
    while tree_node_count & (PARTICLES_PER_GROUP - 1) != 0 {
        tree_node_count += 1;
    }
    // println!("{}", tree_node_count - 1);
    tree_node_count - 1
}

// fn dispatch_compute_kernel(cpass: &mut wgpu::ComputePass, compute_kernel: &ComputeKernel, work_group_count: u32) {
//
//     cpass.set_pipeline(&compute_kernel.compute_pipeline);
//     cpass.set_bind_group(0, &compute_kernel.bind_group, &[]);
//     cpass.dispatch_workgroups(work_group_count, 1, 1);
// }