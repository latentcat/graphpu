

struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Node {
    position: vec3<f32>,
    velocity: vec3<f32>,
};

@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0) var<storage, read> nodeSrc : array<Node>;
@group(1) @binding(1) var<storage, read> edgeSrc : array<vec2<u32>>;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var edge = edgeSrc[i.instance_index];
    var node_id = edge[i.vertex_index];
    var node = nodeSrc[node_id];

    var v: Varing;
    v.position = vec4<f32>(node.position.xyz, 1.0);
    v.position = projection * v.position;
    v.tex_coords = vec2<f32>(0.0);

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {


    var out_color = vec4<f32>(0.05);

    return out_color;
}
