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

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<uniform> uniforms: Uniforms;
@group(0) @binding(2) var<storage, read_write> nodeSrc: array<Node>;
@group(0) @binding(3) var<storage, read> edgeSrc: array<vec2<u32>>;
@group(0) @binding(4) var<storage, read_write> springForceSrc: array<atomic<u32>>;


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
    atomicStore(&springForceSrc[index * 3u + 0u], 0u);
    atomicStore(&springForceSrc[index * 3u + 1u], 0u);
    atomicStore(&springForceSrc[index * 3u + 2u], 0u);
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
//
//    interlocked_add_float(&nodeSrc[source_node].spring_force_x, dir.x);
//    interlocked_add_float(&nodeSrc[source_node].spring_force_y, dir.y);
//    interlocked_add_float(&nodeSrc[source_node].spring_force_z, dir.z);
//    interlocked_add_float(&nodeSrc[target_node].spring_force_x, -dir.x);
//    interlocked_add_float(&nodeSrc[target_node].spring_force_y, -dir.y);
//    interlocked_add_float(&nodeSrc[target_node].spring_force_z, -dir.x);
    atomicExchange(&springForceSrc[source_node * 3u + 2u], 0u);

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

        electron_force += -dir / (dot(dir, dir) * dot(dir, dir));

        continuing {
            i = i + 1u;
        }
    }

    var gravaty_force = -vPos;

//    var spring_force = bitcast<vec3<f32>>();

    var vForce: vec3<f32> = electron_force * 0.25 + gravaty_force * 10.0;

    vVel = vVel + vForce;

    // clamp velocity for a more pleasing simulation
    if (dot(vVel, vVel) > 0.0) {
        vVel = normalize(vVel) * clamp(length(vVel), 0.0, 1.0);
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