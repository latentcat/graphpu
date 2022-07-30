struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main_vs(
    @location(0) particle_pos: vec2<f32>,
    @location(1) particle_vel: vec2<f32>,
    @location(2) quad_pos: vec2<f32>,
) -> Varing {
    var v: Varing;
    v.position = vec4<f32>(particle_pos + quad_pos * 0.01, 0.0, 1.0);
    v.tex_coords = quad_pos;
    return v;
}

@fragment
fn main_fs(v: Varing) -> @location(0) vec4<f32> {

    let sdf = dot(v.tex_coords, v.tex_coords);
    let alpha = step(sdf, 1.0);

    var out_color = vec4<f32>(alpha * 0.4);

    return out_color;
}
