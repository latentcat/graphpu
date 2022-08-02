struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Node {
    pos : vec3<f32>,
    vel : vec3<f32>,
};

struct Edge {
    source_id: u32,
    target_id: u32,
}

@group(0) @binding(0) var<storage, read> nodeSrc : array<Node>;
@group(0) @binding(1) var<storage, read> edgeSrc : array<Edge>;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var edge = edgeSrc[i.instance_index];
    var node_id = vec2<u32>(edge.source_id, edge.target_id)[i.vertex_index];
    var node = nodeSrc[node_id];

    var v: Varing;
    v.position = vec4<f32>(node.pos.xy, 0.0, 1.0);
    v.tex_coords = vec2<f32>(0.0);

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {


    var out_color = vec4<f32>(0.05);

    return out_color;
}
