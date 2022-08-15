struct Node {
    position: vec3<f32>,
    velocity: vec3<f32>,
    mass: atomic<u32>,
};

struct SimParams {
    deltaT : f32,
    rule1Distance : f32,
    rule2Distance : f32,
    rule3Distance : f32,
    rule1Scale : f32,
    rule2Scale : f32,
    rule3Scale : f32,
};

struct Uniforms {
    frame_num: u32,
};

struct Bound {
    bound_min: vec3<f32>,
    bound_max: vec3<f32>,
}

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;
@group(0) @binding(2) var<storage, read_write> nodeSrc: array<Node>;
@group(0) @binding(3) var<storage, read> edgeSrc: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read_write> springForceSrc: array<atomic<i32>>;
@group(0) @binding(5) var<storage, read_write> bounding: array<Bound>;


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

//fn interlocked_add_float(spring_force: ptr<function, atomic<u32>>, value: f32) {
//    var i_val: u32 = bitcast<u32>(value);
//    var tmp0: u32 = 0u;
//    var tmp1: u32;
//    while (true)
//    {
//
////    var result = atomicCompareExchangeWeak(&nodeSrc[source_node].spring_force_x, 0u, 1u);
//        var tmp1 = atomicLoad(spring_force);
//        if (tmp1 == tmp0) {
//            break;
//        }
//        tmp0 = tmp1;
//        i_val = bitcast<u32>(value + bitcast<f32>(tmp1));
//    }
//}



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

    var dir_i32 = vec3<i32>(dir * 10000.0);

    atomicAdd(&springForceSrc[source_node * 3u + 0u], dir_i32.x);
    atomicAdd(&springForceSrc[source_node * 3u + 1u], dir_i32.y);
    atomicAdd(&springForceSrc[source_node * 3u + 2u], dir_i32.z);
    atomicAdd(&springForceSrc[target_node * 3u + 0u], -dir_i32.x);
    atomicAdd(&springForceSrc[target_node * 3u + 1u], -dir_i32.y);
    atomicAdd(&springForceSrc[target_node * 3u + 2u], -dir_i32.z);

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

    spring_force.x = f32( atomicLoad(&springForceSrc[index * 3u + 0u]) );
    spring_force.y = f32( atomicLoad(&springForceSrc[index * 3u + 1u]) );
    spring_force.z = f32( atomicLoad(&springForceSrc[index * 3u + 2u]) );

    atomicStore(&springForceSrc[index * 3u + 0u], 0);
    atomicStore(&springForceSrc[index * 3u + 1u], 0);
    atomicStore(&springForceSrc[index * 3u + 2u], 0);

    spring_force *= 0.001;

//    var spring_force = bitcast<vec3<f32>>();

    var vForce: vec3<f32> = electron_force * 0.05 + gravaty_force * 10. + spring_force * 100.0;

    vVel = vVel + vForce * params.deltaT;

    // clamp velocity for a more pleasing simulation
    if (dot(vVel, vVel) > 0.0) {
        vVel = normalize(vVel) * clamp(length(vVel) * 0.1, 0.0, .5);
    }

    // kinematic update
    vPos += vVel * params.deltaT;

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