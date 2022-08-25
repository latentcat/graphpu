struct Node {
    position: vec3<f32>,
    velocity: vec3<f32>,
    mass: atomic<u32>,
};

struct Uniforms {
    frame_num: u32,
};

struct Bound {
    bound_min: vec3<f32>,
    bound_max: vec3<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read_write> nodeSrc: array<Node>;
@group(0) @binding(2) var<storage, read> edgeSrc: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read_write> springForceSrc: array<atomic<i32>>;
@group(0) @binding(4) var<storage, read_write> bounding: array<Bound>;


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

fn random_xy(seed_x: u32, seed_y: u32) -> f32 {
    return f32(hash(hash(seed_x) + seed_y)) / 4294967295.0; // 2^32-1
}

fn atomic_add_f32(springIndex: u32, updateValue: f32) {
    let atomic_ptr = &springForceSrc[springIndex];
    var new_u32 = bitcast<i32>(updateValue);
    var assumed: i32 = 0;
    var origin: i32;
    while (true) {
        origin = atomicCompareExchangeWeak(atomic_ptr, assumed, new_u32);
        if (origin == assumed) {
            break;
        }
        assumed = origin;
        new_u32 = bitcast<i32>(bitcast<f32>(origin) + updateValue);
    }
}

@compute
@workgroup_size(128)
fn gen_node(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;
    var vVel : vec3<f32> = nodeSrc[index].velocity;

    vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.z = random_xy(index, 2u + 3u * uniforms.frame_num) * 2.0 - 1.0;

    vVel = vec3<f32>(0.0);

    // Write back
    nodeSrc[index].position = vPos;
    nodeSrc[index].velocity = vVel;
    atomicStore(&nodeSrc[index].mass, 0u);
    atomicStore(&springForceSrc[index * 3u + 0u], 0);
    atomicStore(&springForceSrc[index * 3u + 1u], 0);
    atomicStore(&springForceSrc[index * 3u + 2u], 0);
    let target_node: u32 = index * 3u + 2u;
    let aa = atomicExchange(&springForceSrc[target_node], 0);
}

@compute
@workgroup_size(128)
fn cal_mass(@builtin(global_invocation_id) global_invocation_id: vec3<u32>)
{
    let total = arrayLength(&edgeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var edge = edgeSrc[index];
    let source_node: u32 = edge[0];
    let target_node: u32 = edge[1];

    atomicAdd(&nodeSrc[source_node].mass, 1u);
    atomicAdd(&nodeSrc[target_node].mass, 1u);

}


@compute
@workgroup_size(128)
fn attractive_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>)
{
    let total = arrayLength(&edgeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var edge = edgeSrc[index];
    let source_node: u32 = edge[0];
    let target_node: u32 = edge[1];

    var dir = nodeSrc[target_node].position - nodeSrc[source_node].position;

    atomic_add_f32(source_node * 3u + 0u, dir.x);
    atomic_add_f32(source_node * 3u + 1u, dir.y);
    atomic_add_f32(source_node * 3u + 2u, dir.z);
    atomic_add_f32(target_node * 3u + 0u, -dir.x);
    atomic_add_f32(target_node * 3u + 1u, -dir.y);
    atomic_add_f32(target_node * 3u + 2u, -dir.z);
}

var<workgroup> smin: array<vec3<f32>, 128>;
var<workgroup> smax: array<vec3<f32>, 128>;

@compute
@workgroup_size(128)
fn reduction_bounding(
    @builtin(local_invocation_id) local_id: vec3<u32>,
    @builtin(local_invocation_index) local_index: u32,
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {

    let index = global_id.x;

    smin[local_index] = nodeSrc[index].position;
    smax[local_index] = nodeSrc[index].position;
    workgroupBarrier();

    for (var s = 64u; s > 0u; s >>= 1u) {
        if (local_index < s) {
            let k = local_index + s;
            smin[local_index] = min(smin[local_index], smin[k]);
            smax[local_index] = max(smax[local_index], smax[k]);
        }
        workgroupBarrier();
    }

    if (local_index == 0u) {
        bounding[group_id.x].bound_min = smin[0];
        bounding[group_id.x].bound_max = smax[0];
    }
}

@compute
@workgroup_size(1)
fn bounding_box() {
    var bound_min_min = bounding[0].bound_min;
    var bound_max_max = bounding[0].bound_max;
    let node_group_count = arrayLength(&nodeSrc) / 128u;
    for (var i = 0u; i < node_group_count; i++) {
        bound_min_min = min(bound_min_min, bounding[i].bound_min);
        bound_max_max = max(bound_max_max, bounding[i].bound_max);
    }
    bounding[0].bound_min = bound_min_min;
    bounding[0].bound_max = bound_max_max;
}

@compute
@workgroup_size(128)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos: vec3<f32> = nodeSrc[index].position;
    var vVel: vec3<f32> = nodeSrc[index].velocity;

    var i : u32 = 0u;
    var electron_force = vec3<f32>(0.0);
    loop {
        if (i >= total) { break; }
        if (i == index) { continue; }

        let pos = nodeSrc[i].position;
        let dir = pos - vPos;

        electron_force += -dir / dot(dir, dir);

        continuing {
            i = i + 1u;
        }
    }

    var gravaty_force = -vPos;

    var spring_force = vec3<f32>(0.0);

    spring_force.x = bitcast<f32>(atomicLoad(&springForceSrc[index * 3u + 0u]));
    spring_force.y = bitcast<f32>(atomicLoad(&springForceSrc[index * 3u + 1u]));
    spring_force.z = bitcast<f32>(atomicLoad(&springForceSrc[index * 3u + 2u]));

    atomicStore(&springForceSrc[index * 3u + 0u], 0);
    atomicStore(&springForceSrc[index * 3u + 1u], 0);
    atomicStore(&springForceSrc[index * 3u + 2u], 0);

    var vForce: vec3<f32> = electron_force * 0.05 + gravaty_force * 10. + spring_force * 1000.0;

    vVel = vVel + vForce * 0.04;

    // clamp velocity for a more pleasing simulation
    if (dot(vVel, vVel) > 0.0) {
        vVel = normalize(vVel) * clamp(length(vVel) * 0.1, 0.0, .5);
    }

    // kinematic update
    vPos += vVel * 0.04;

    // Write back
    nodeSrc[index].position = vPos;
    nodeSrc[index].velocity = vVel;
}


@compute
@workgroup_size(128)
fn randomize(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;
    var vVel : vec3<f32> = nodeSrc[index].velocity;

    vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.z = random_xy(index, 2u + 3u * uniforms.frame_num) * 2.0 - 1.0;

    vVel = vec3<f32>(0.0);

    // Write back
    nodeSrc[index].position = vPos;
    nodeSrc[index].velocity = vVel;
}

@compute
@workgroup_size(128)
fn copy(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;
    var vVel : vec3<f32> = nodeSrc[index].velocity;

  // Write back
//  nodeSrc[index] = Node(vPos, vVel);
}