struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Particle {
  pos : vec2<f32>,
  vel : vec2<f32>,
};

@group(0) @binding(0) var<storage, read> particlesSrc : array<Particle>;

@vertex
fn main_vs(
    @location(0) quad_pos: vec2<f32>,
    i: Input
) -> Varing {
    var particle = particlesSrc[i.instance_index];

    var v: Varing;
    v.position = vec4<f32>(particle.pos + quad_pos * 0.0075, 0.0, 1.0);
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
