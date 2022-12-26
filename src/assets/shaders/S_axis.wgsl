struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
};

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    time: vec4<f32>,
    screen: vec4<f32>,
    camera: vec4<f32>,
}

@group(0) @binding(0) var<uniform> transform: Transform;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {

    var v: Varing;

    var has_x = f32(i.instance_index);
    var has_z = 1.0 - f32(i.instance_index);

    var length = quad_pos.x * 10000000.0;

    v.position = vec4<f32>(vec3<f32>(length * has_x, 0.0, length * has_z), 1.0);
    v.position = transform.projection * transform.view * v.position;

    var dir = transform.projection * transform.view * vec4<f32>(0.01 * has_x, 0.0, 0.01 * has_z, 1.0);
    var center = transform.projection * transform.view * vec4<f32>(0.0, 0.0, 0.0, 1.0);

    var cast_position = center + (dir - center) / (dir.z - center.z) * (transform.camera.z * 0.001 - center.z);

    if v.position.w < transform.camera.z {
        v.position = cast_position;
    }

    var ratio = transform.camera.x;
    var dir_normal = normalize(vec2<f32>(dir.y / ratio / ratio, -dir.x));

    v.position = v.position / abs(v.position.w);
    v.position += vec4<f32>(dir_normal * quad_pos.y * (1.5 / transform.screen.y), 0.0, 0.0);

    v.tex_coords = quad_pos;

    // Yellow / Blue
    var colors = array<vec3<f32>, 2>(vec3<f32>(.98, .47, .08), vec3<f32>(.272, .866, .855));
    v.color = colors[i.instance_index];

    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    var color = v.color;
    var x = v.position.z * 0.5 + 0.5;
    var lerp = (1.0 - pow(x, 10000.0)) * 1.2;
    color *= lerp;

    let alpha = 1.0;

    return vec4<f32>(color, alpha);
}
