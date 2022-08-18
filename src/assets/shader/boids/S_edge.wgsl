

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
    mass: u32,
};

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> transform: Transform;

@group(1) @binding(0) var<storage, read> nodeSrc : array<Node>;
@group(1) @binding(1) var<storage, read> edgeSrc : array<vec2<u32>>;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var edge = edgeSrc[i.instance_index];
    var node_id = edge[i.vertex_index % 2u];
    var node = nodeSrc[node_id];

    var node_dir = nodeSrc[edge[1]].position - nodeSrc[edge[0]].position;
    var dir = transform.view * vec4<f32>(node_dir, 1.0);
    dir = dir / dir.w;
    var quad_dir = normalize(vec2<f32>(dir.y, -dir.x));

    var v: Varing;
    v.position = vec4<f32>(node.position.xyz, 1.0);
    v.position = transform.view * v.position;
    v.position += vec4<f32>(quad_pos.y * quad_dir * 0.0075 * 0.25, 0.0, 0.0);
    v.position = transform.projection * v.position;
    v.tex_coords = vec2<f32>(0.0);

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    var out_color = vec4<f32>(1.0, 1.0, 1.0, 0.1);

    return out_color;
}
