struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @builtin(position) position: vec4<f32>,
};

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    time: vec4<f32>,
    screen: vec4<f32>,
    camera: vec4<f32>,
}

struct Bound {
    bound_min: vec3<f32>,
    bound_max: vec3<f32>,
}

@group(0) @binding(0) var<uniform> transform: Transform;

@group(1) @binding(0) var<storage, read> boundSrc : array<Bound>;

@vertex
fn main_vs(
    @location(0) bound_pos: vec3<f32>,
    i: Input
) -> Varing {
    var bound = boundSrc[0];

    var bound_center = (bound.bound_min + bound.bound_max) / 2.0;
    var bound_radius = (bound.bound_max - bound.bound_min) / 2.0;

    var v: Varing;
    v.position = vec4<f32>(bound_center + bound_pos * bound_radius, 1.0);
    v.position = transform.projection * transform.view * v.position;

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    var out_color = vec4<f32>(1.0, 1.0, 1.0, 1.0);

    return out_color;
}
