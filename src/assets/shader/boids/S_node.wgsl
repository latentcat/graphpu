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
    mass: u32,
    spring_force_x: u32,
    spring_force_y: u32,
    spring_force_z: u32,
};

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> transform: Transform;

@group(1) @binding(0) var<storage, read> nodeSrc : array<Node>;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var node = nodeSrc[i.instance_index];

    var v: Varing;
    v.position = vec4<f32>(node.position.xyz, 1.0);
    v.position = transform.view * v.position;
    v.position += vec4<f32>(quad_pos * 0.01 * (1.0 + f32(node.mass) * 0.1), 0.0, 0.0);
    v.position = transform.projection * v.position;
    v.tex_coords = quad_pos;

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    let sdf = dot(v.tex_coords, v.tex_coords);
    let clip = step(sdf, 1.0);

    var out_color = vec4<f32>(1.0);

    let alpha = 0.5;

    out_color.r *= alpha;
    out_color.g *= alpha;
    out_color.b *= alpha;

    if clip < 0.5 {
        discard;
    }

    return out_color;
}
