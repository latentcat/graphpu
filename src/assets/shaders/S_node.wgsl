struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
    @location(1) color: vec3<f32>,
};

struct Node {
    position: vec3<f32>,
    force: vec3<f32>,
    prev_force: vec3<f32>,
    mass: u32,
};

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    time: vec4<f32>,
    screen: vec4<f32>,
    camera: vec4<f32>,
}

struct Kvp {
    sort_key: f32,
    index: u32,
}

@group(0) @binding(0) var<uniform> transform: Transform;

@group(1) @binding(0) var<storage, read> node_src : array<Node>;
@group(1) @binding(1) var<storage, read> kvps: array<Kvp>;

fn vs_transform(
    position: ptr<function,vec4<f32>>,
    node_position: vec3<f32>,
    quad_pos: vec2<f32>
) {
    *position = vec4<f32>(node_position, 1.0);

    *position = transform.view * *position;
    *position += vec4<f32>(quad_pos * 0.0025, 0.0, 0.0);

    *position = transform.projection * *position;
    var quad_pos_ratio = quad_pos;
    quad_pos_ratio.x /= transform.camera.x;
    *position += vec4<f32>(quad_pos_ratio * (1.5 / transform.screen.y) * (*position).w, 0.0, 0.0);
}

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var node = node_src[kvps[i.instance_index].index];
    var kvp = kvps[i.instance_index];

    var v: Varing;

    vs_transform(&v.position, node.position, quad_pos);

    v.tex_coords = quad_pos;

    v.color = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), f32(i.instance_index) / f32(arrayLength(&node_src)));
    if (kvp.index == 0u) { v.color = vec3<f32>(1.0); }

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    let sdf = dot(v.tex_coords, v.tex_coords);
    let clip = step(sdf, 1.0);

    var out_color = vec4<f32>(v.color, 1.0);

    if clip < 0.5 {
        discard;
    }

    return out_color;
}