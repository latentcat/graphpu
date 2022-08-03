struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Node {
  position : vec3<f32>,
  velocity : vec3<f32>,
};

@group(0) @binding(0) var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0) var<storage, read> nodeSrc : array<Node>;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var node = nodeSrc[i.instance_index];

    var v: Varing;
    v.position = vec4<f32>(node.position.xy + quad_pos * 0.0075, node.position.z, 1.0);
    v.position = projection * v.position;
    v.tex_coords = quad_pos;

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    let sdf = dot(v.tex_coords, v.tex_coords);
    let alpha = step(sdf, 1.0);

    var out_color = vec4<f32>(alpha * 0.25);

    return out_color;
}
