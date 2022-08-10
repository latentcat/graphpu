struct Input {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct Varing {
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Node {
    position : vec3<f32>,
    velocity : vec3<f32>,
    mass: u32,
};

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
}

fn hash(s: u32) -> u32 {
    var t : u32 = s;
    t ^= 2747636419u;
    t *= 2654435769u;
    t ^= t >> 16u;
    t *= 2654435769u;
    t ^= t >> 16u;
    t *= 2654435769u;
    return t;
}

fn random(seed: u32) -> f32 {
    return f32(hash(seed)) / 4294967295.0; // 2^32-1
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
    v.position += vec4<f32>(quad_pos * 0.0075 * (1.0 + f32(node.mass) * 0.05), 0.0, 0.0);
    v.position = transform.projection * v.position;
    v.tex_coords = quad_pos;
    v.color = vec4<f32>(
        random(i.instance_index * 4u + 0u),
        random(i.instance_index * 4u + 1u),
        random(i.instance_index * 4u + 2u),
        1.0
    );

    return v;
}

struct Output {
  @builtin(frag_depth) depth: f32,
  @location(0) color: vec4<f32>
}

@fragment
fn main_fs(v: Varing) -> Output {

    var o: Output;
    o.depth = v.position.z;

    var sdf = dot(v.tex_coords, v.tex_coords) - 1.0 + 0.1;
    sdf = max(0.0, sdf) * 30.0;
    sdf = exp(-sdf * sdf);

    var out_color = vec3<f32>(1.0);

    out_color = v.color.xyz;

    let alpha = 0.8 * sdf;

//    out_color *= alpha;

    if sdf < 0.99 {
        o.depth = 0.9999999;
    }

    if sdf < 0.01 {
        discard;
    }

    o.color = vec4<f32>(out_color, sdf);

    return o;
}
