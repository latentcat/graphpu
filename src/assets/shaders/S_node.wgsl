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
    v.position += vec4<f32>(quad_pos * 0.0075, 0.0, 0.0);
    v.position = transform.projection * v.position;
    var quad_pos_ratio = quad_pos;
    quad_pos_ratio.x /= transform.camera.x;
    v.position += vec4<f32>(quad_pos_ratio * (2.0 / transform.screen.y) * v.position.w, 0.0, 0.0);
    v.tex_coords = quad_pos;
    v.color = mix(vec3<f32>(0.0, 1.0, 0.0), vec3<f32>(1.0, 0.0, 0.0), f32(i.instance_index) / f32(arrayLength(&nodeSrc)));
    if (i.instance_index == 0u) {
        v.color = vec3<f32>(1.0);
    }

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    let sdf = dot(v.tex_coords, v.tex_coords);
    let clip = step(sdf, 1.0);

    var out_color = vec4<f32>(v.color, 1.0);

//    let alpha = 1.0;
//
//    out_color.r *= alpha;
//    out_color.g *= alpha;
//    out_color.b *= alpha;

    if clip < 0.5 {
        discard;
    }

    return out_color;
}
