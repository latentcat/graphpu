

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

@group(0) @binding(0) var<uniform> transform: Transform;

@group(1) @binding(0) var<storage, read> node_src : array<Node>;
@group(1) @binding(1) var<storage, read> edge_src : array<vec2<u32>>;

fn vs_transform(
    node_pos_a: vec3<f32>,
    node_pos_b: vec3<f32>,
    node_pos: vec3<f32>,
    quad_pos: vec2<f32>
) -> vec4<f32> {
    let node_a = transform.projection * transform.view * vec4<f32>(node_pos_a, 1.0);
    let node_b = transform.projection * transform.view * vec4<f32>(node_pos_b, 1.0);
    var dir = node_a / node_a.w - node_b / node_b.w;
    dir.y = dir.y / transform.camera.x;
    let quad_dir = normalize(vec2<f32>(dir.y, -dir.x));

    var position = vec4<f32>(node_pos, 1.0);
    position = transform.view * position;
    position += vec4<f32>(quad_pos.y * quad_dir * 0.01 * 0.03, 0.0, 0.0);
    position = transform.projection * position;

    var quad_pos_ratio = quad_pos.y * quad_dir;
    quad_pos_ratio.x /= transform.camera.x;
    position += vec4<f32>(quad_pos_ratio * (0.5 / transform.screen.y) * position.w, 0.0, 0.0);
    return position;
}

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    let edge = edge_src[i.instance_index];

    var v: Varing;

    v.position = vs_transform(
        node_src[edge[0]].position,
        node_src[edge[1]].position,
        node_src[edge[i.vertex_index % 2u]].position,
        quad_pos
    );

    v.tex_coords = vec2<f32>(0.0);

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    let out_color = vec4<f32>(1.0, 1.0, 1.0, 0.02);

    return out_color;
}