struct Node {
    position: vec3<f32>,
    force: vec3<f32>,
    prev_force: vec3<f32>,
    mass: atomic<u32>,
};

struct Uniforms {
    frame_num: u32,
};

struct Bound {
    bound_min: vec3<f32>,
    bound_max: vec3<f32>,
}

struct BHTree {
    max_depth: atomic<u32>,
    bottom: atomic<u32>,
    radius: f32,
}

struct BHTreeNode {
    position: vec3<f32>,
    _empty: i32,
    mass: atomic<i32>,
    count: i32,
    start: atomic<i32>,
    sort: i32,
}

struct Kvp {
    sort_key: f32,
    index: u32,
}

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    time: vec4<f32>,
    screen: vec4<f32>,
    camera: vec4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var<storage, read_write> nodeSrc: array<Node>;
@group(0) @binding(2) var<storage, read> edgeSrc: array<vec2<u32>>;
@group(0) @binding(3) var<storage, read_write> springForceSrc: array<atomic<i32>>;
@group(0) @binding(4) var<storage, read_write> bounding: array<Bound>;
@group(0) @binding(5) var<storage, read_write> bhTree: BHTree;
@group(0) @binding(6) var<storage, read_write> treeNode: array<BHTreeNode>;
@group(0) @binding(7) var<storage, read_write> treeChild: array<atomic<i32>>;
@group(0) @binding(8) var<storage, read_write> kvps: array<Kvp>;
@group(0) @binding(9) var<uniform> transform: Transform;

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
@workgroup_size(256)
fn gen_node(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;

    vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.z = random_xy(index, 2u + 3u * uniforms.frame_num) * 2.0 - 1.0;

    // Write back
    nodeSrc[index].position = vPos;
    nodeSrc[index].force = vec3<f32>(0.0);
    nodeSrc[index].prev_force = vec3<f32>(0.0);
    nodeSrc[index].mass = 1u;
    springForceSrc[index * 3u + 0u] = 0;
    springForceSrc[index * 3u + 1u] = 0;
    springForceSrc[index * 3u + 2u] = 0;

}

@compute
@workgroup_size(256)
fn cal_mass(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
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
@workgroup_size(256)
fn cal_gravity_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    // TODO: Global Param
    let strong_gravity = true;
    let k_gravity = 1.0;

    let pos = nodeSrc[index].position;
    let mass = f32(atomicLoad(&nodeSrc[index].mass));
    var gravity_force: f32;
    if (strong_gravity) {
        gravity_force =  k_gravity * mass;
    } else {
        if (pos.x != 0.0 || pos.y != 0.0 || pos.z != 0.0) {
            gravity_force = k_gravity * mass * inverseSqrt(dot(pos, pos));
        }
        else {
            gravity_force = 0.0;
        }
    }
//    nodeSrc[index].force +=  -pos * gravity_force;
//    nodeSrc[index].force +=  -pos * min(gravity_force, 1.0);
    nodeSrc[index].force +=  -pos * 0.5;
}

@compute
@workgroup_size(256)
fn attractive_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
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

var<workgroup> smin: array<vec3<f32>, 256>;
var<workgroup> smax: array<vec3<f32>, 256>;

@compute
@workgroup_size(256)
fn reduction_bounding(
    @builtin(local_invocation_index) local_index: u32,
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {

    var index = global_id.x;
    let total = arrayLength(&nodeSrc);
    if (index >= total) {
        index = total - 1u;
    }

    smin[local_index] = nodeSrc[index].position;
    smax[local_index] = nodeSrc[index].position;
    workgroupBarrier();

    for (var s = 256u / 2u; s > 0u; s >>= 1u) {
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
    let node_group_count = u32(ceil(f32(arrayLength(&nodeSrc)) / 256.0));
    for (var i = 0u; i < node_group_count; i++) {
        bound_min_min = min(bound_min_min, bounding[i].bound_min);
        bound_max_max = max(bound_max_max, bounding[i].bound_max);
    }

    bounding[0].bound_min = bound_min_min;
    bounding[0].bound_max = bound_max_max;

    let box = bound_max_max - bound_min_min;
    let tree_node_count = arrayLength(&treeNode) - 1u;
    bhTree.radius = max(max(box.x, box.y), box.z) * 0.5;
    atomicStore(&bhTree.bottom, tree_node_count);
    atomicStore(&bhTree.max_depth, 0u);
    atomicStore(&treeNode[tree_node_count].mass, -1);
    atomicStore(&treeNode[tree_node_count].start, 0);
    treeNode[tree_node_count].position = (bound_min_min + bound_max_max) * 0.5;
    treeNode[tree_node_count].count = -1;
    treeNode[tree_node_count].sort = 0;
}

@compute
@workgroup_size(256)
fn clear_1(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&treeNode);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    for (var i = 0u; i < 8u; i++) {
        atomicStore(&treeChild[index * 8u + i], -1);
    }
}

@compute
@workgroup_size(256)
fn tree_building(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    var index = global_invocation_id.x;
    let node_count = arrayLength(&nodeSrc);
    let tree_node_count = arrayLength(&treeNode) - 1u;
    let root_pos = treeNode[tree_node_count].position;
    let inc = min(node_count, 16384u); // should change

    var skip = 1;
    var pos: vec3<f32>;
    var dp: vec3<f32>;
    var rdp: vec3<f32>;
    var n = tree_node_count;
    var depth = 1u;
    var local_max_depth = 1u;
    var j = 0u;
    var root_r = bhTree.radius;
    var r = root_r * 0.5;

    var limit = 10000;

    while (index < node_count) {

        limit--;
        if (limit < 0) {
            return;
        }

        if (skip != 0) {
            skip = 0;
            pos = nodeSrc[index].position;

            n = tree_node_count;
            r = root_r * 0.5;
            depth = 1u;

            let compare = step(root_pos, pos);
            j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u); // 八个象限
            dp = -r + compare * (2.0 * r);
            rdp = root_pos + dp; // 所在象限的原点
        }

        // atomicAdd(&treeChild[n * 8u + j], 0); // ...
        var ch = atomicLoad(&treeChild[n * 8u + j]);

        // 迭代至叶节点
        while (ch >= i32(node_count)) {
            n = u32(ch);
            depth++;
            r *= 0.5;

            let compare = step(rdp, pos);
            j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u);
            dp = -r + compare * (2.0 * r);

            rdp += dp;
            ch = atomicLoad(&treeChild[n * 8u + j]);
        }

        let locked = n * 8u + j;
        var locked_ch = -1;

        // 非 lock 状态
        if (ch != -2) {
            if (ch == -1) {
                var v = -1;
                let origin = atomicCompareExchangeWeak(&treeChild[locked], v, i32(index));
                if (origin == -1) {
                    local_max_depth = max(depth, local_max_depth);
                    index += inc;
                    skip = 1;
                } else {
                     skip = 0;
                 }
            } else {
                // 格子已被占用，将其设置为 lock 状态
                var v = ch;
                let origin = atomicCompareExchangeWeak(&treeChild[locked], v, -2);
                if (ch == origin) {
                    // lock 成功，如果两个点的位置相同，做一点微小偏移就行了
                    if (all(nodeSrc[ch].position == pos)) {
                        nodeSrc[index].position += vec3<f32>(0.1, -0.05, 0.1);
                        skip = 0;
                        atomicStore(&treeChild[locked], ch);
                        break;
                    }

                    // 两个点位置不同，则开始分裂
                    locked_ch = -1;
                    loop {

                        // 1. create new cell
                        let cell = atomicSub(&bhTree.bottom, 1u) - 1u;
                        if (cell <= node_count) {
                            return;
                        }

                        if (locked_ch != -1) {
                            atomicStore(&treeChild[n * 8u + j], i32(cell));
                        }
                        locked_ch = max(locked_ch, i32(cell));

                        // 2. make newly created cell current
                        depth++;
                        n = cell;
                        r *= 0.5;

                        // 3. insert old body into current quadrant
                        let compare = step(rdp, nodeSrc[ch].position);
                        j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u);

                        atomicStore(&treeChild[cell * 8u + j], ch);

                        // 4. determin center + quadrant for cell of new body
                        let compare = step(rdp, pos);
                        j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u);
                        dp = -r + compare * (2.0 * r);

                        rdp += dp;

                        // 5. visit this cell/chec if in use (possibly by old body)
                        ch = atomicLoad(&treeChild[n * 8u + j]);

                        if (ch < 0) {
                            break;
                        }

                    };
                    atomicStore(&treeChild[n * 8u + j], i32(index));
                    local_max_depth = max(depth, local_max_depth);
                    index += inc;
                    skip = 2;
                }
            }
        }
        workgroupBarrier();
        if (skip == 2) {
            atomicStore(&treeChild[locked], locked_ch);
        }
    }
    atomicMax(&bhTree.max_depth, local_max_depth);
}

@compute
@workgroup_size(256)
fn clear_2(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&treeNode) - 1u;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }
    treeNode[index].position = vec3<f32>(0.0);
    treeNode[index].count = -1;
    treeNode[index].sort = 0;
    atomicStore(&treeNode[index].start, -1);
    atomicStore(&treeNode[index].mass, -1);
}

@compute
@workgroup_size(256)
fn summarization(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let bottom = atomicLoad(&bhTree.bottom);
    let tree_node_count = arrayLength(&treeNode) - 1u;
    let node_count = arrayLength(&nodeSrc);
    let inc = min(node_count, 16384u);
    var index = u32((i32(bottom) & -32) + i32(global_invocation_id.x));
    if (index < bottom) {
        index += inc;
    }

    // TODO: ch bounds check
    var schild: array<u32, 8>;
    var smass: array<i32, 8>;
    let restart = index;
    for (var j = 0; j < 5; j++) {
        while (index <= tree_node_count) {
            if (atomicLoad(&treeNode[index].mass) < 0) {
                var ch = 0u;
                var i = 0u;
                for (i = 0u; i < 8u; i++) {
                    ch = u32(atomicLoad(&treeChild[index * 8u + i]));
                    schild[i] = ch;
                    // atomicAdd(&treeNode[ch].mass, 0);
                    smass[i] = atomicLoad(&treeNode[ch].mass);
                    if (ch >= node_count && smass[i] < 0) {
                        break;
                    }
                }
                if (i == 8u) {
                    var cm = 0;
                    var pos = vec3<f32>(0.0);
                    var cnt = 0;
                    for (i = 0u; i < 8u; i++) {
                        ch = schild[i];
                        if (ch >= node_count) {
                            let m = smass[i];
                            cnt += treeNode[ch].count;
                            pos += treeNode[ch].position * f32(m);
                            cm += m;
                        } else {
                            let m = i32(atomicLoad(&nodeSrc[ch].mass));
                            cnt += 1;
                            pos += nodeSrc[ch].position * f32(m);
                            cm += m;
                        }
                    }
                    treeNode[index].count = cnt;
                    treeNode[index].position = pos / f32(cm);
                    // workgroupBarrier();
                    atomicStore(&treeNode[index].mass, cm);
                }
            }
            index += inc;
        }
        index = restart;
    }

    var j = 0;
    var flag = false;
    while (index <= tree_node_count) {
        var cm = 0;
        if (index < node_count) {
            index += inc;
        } else if (index >= node_count && atomicLoad(&treeNode[index].mass) >= 0) {
            index += inc;
        } else {
            if (j == 0) {
                j = 8;
                for (var i = 0u; i < 8u; i++) {
                    let ch = u32(atomicLoad(&treeChild[index * 8u + i]));
                    schild[i] = ch;
                    smass[i] = atomicLoad(&treeNode[ch].mass);
                    if (ch < node_count || smass[i] >= 0) {
                        j--;
                    }
                }
            } else {
                j = 8;
                for (var i = 0u; i < 8u; i++) {
                    let ch = schild[i];
                    let old_mass = smass[i];
                    smass[i] = atomicLoad(&treeNode[ch].mass);
                    if (ch < node_count || old_mass >= 0 || smass[i] >= 0) {
                        j--;
                    }
                }
            }

            if (j == 0) {
                cm = 0;
                var pos = vec3<f32>(0.0);
                var cnt = 0;
                for (var i = 0u; i < 8u; i++) {
                    let ch = schild[i];
                    if (ch >= node_count) {
                        let m = smass[i];
                        cnt += treeNode[ch].count;
                        pos += treeNode[ch].position * f32(m);
                        cm += m;
                    } else {
                        let m = i32(atomicLoad(&nodeSrc[ch].mass));
                        cnt += 1;
                        pos += nodeSrc[ch].position * f32(m);
                        cm += m;
                    }
                }
                treeNode[index].count = cnt;
                treeNode[index].position = pos / f32(cm);
                flag = true;
            }
        }
        // workgroupBarrier();
        if (flag) {
            if (index < node_count) {
                atomicStore(&nodeSrc[index].mass, u32(cm));
            } else {
                atomicStore(&treeNode[index].mass, cm);
            }
            index += inc;
            flag = false;
        }
    }
}

@compute
@workgroup_size(256)
fn sort(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let tree_node_count = arrayLength(&treeNode) - 1u;
    let bottom = atomicLoad(&bhTree.bottom);
    let node_count = arrayLength(&nodeSrc);
    let inc = min(node_count, 16384u);
    var index = tree_node_count + 1u - inc + global_invocation_id.x;

    var limit = 1000;

    while (index >= bottom) {

        limit--;
        if (limit < 0) {
            treeChild[index] = 1000;
            treeChild[0] = 1000;
            return;
        }
        workgroupBarrier();
        var start = atomicLoad(&treeNode[index].start);

        if (start >= 0) {
            var j = 0u;
            for (var i = 0u; i < 8u; i++) {
                let ch = atomicLoad(&treeChild[index * 8u + i]);
                if (ch >= 0) {
                    // 把子节点集中到开头
                    if (i != j) {
                        atomicStore(&treeChild[index * 8u + i], -1);
                        atomicStore(&treeChild[index * 8u + j], ch);
                    }
                    j++;
                    if (ch >= i32(node_count)) {
                        atomicStore(&treeNode[ch].start, start);
                        start += treeNode[ch].count;
                    } else {
                        treeNode[start].sort = ch;
                        start++;
                    }
                }
            }
            if (index < inc) {
                break;
            }
            index -= inc;
        }
    }
}

@compute
@workgroup_size(256)
fn electron_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let tree_node_count = arrayLength(&treeNode) - 1u;
    let node_count = arrayLength(&nodeSrc);
    let inc = min(node_count, 16384u);

    // TODO: Global Param
    let scale = 0.0003;

    var spos: array<u32, 48>;
    var snode: array<u32, 48>;
    var sdq: array<f32, 48>;

    let itolsq = 1.0;
    let epssq = 0.05 * 0.05;
    let diameter = bhTree.radius * 2.0;
    let max_depth = atomicLoad(&bhTree.max_depth);
    sdq[0] = diameter * diameter * itolsq;
    for (var j = 1u; j < max_depth; j++) {
        sdq[j] = sdq[j - 1u] * 0.25;
        sdq[j - 1u] += epssq;
    }
    sdq[max_depth - 1u] += epssq;

    if (max_depth < 48u) {
        for (var index = global_invocation_id.x; index < node_count; index += inc) {
            let order = treeNode[index].sort;
//            if (order == 0) { continue; }
            let pos = nodeSrc[order].position;
            var af = vec3<f32>(0.0);

            var depth = 0u;
            spos[0] = 0u;
            snode[0] = tree_node_count;

            loop {
                var pd = spos[depth];
                var nd = snode[depth];
                while (pd < 8u) {
                    let n_i32 = atomicLoad(&treeChild[nd * 8u + pd]);
                    pd++;

                    if (n_i32 >= 0) {
                        let n = u32(n_i32);
                        var dp: vec3<f32>;
                        if (n < node_count) {
                            dp = pos - nodeSrc[n].position;
                        } else {
                            dp = pos - treeNode[n].position;
                        }
                        let dist2 = dot(dp, dp);

                        if (n < node_count) {
                            if (dist2 > 0.0) {
                                let factor = scale * f32(atomicLoad(&nodeSrc[order].mass)) * f32(atomicLoad(&nodeSrc[n].mass)) / dist2;
                                af += dp * factor;
                            }
                        } else if (dist2 >= sdq[depth]) {
                            if (dist2 > 0.0) {
                                let factor = scale * f32(atomicLoad(&nodeSrc[order].mass)) * f32(atomicLoad(&treeNode[n].mass)) / dist2;
                                af += dp * factor;
                            }
                        } else {
                            spos[depth] = pd;
                            snode[depth] = nd;
                            depth++;
                            pd = 0u;
                            nd = n;
                        }
                    } else {
                        pd = 8u;
                    }
                }
                if (depth == 0u) {
                    break;
                }
                depth--;
            }
            nodeSrc[order].force += af * 0.25;
        }
    }
}

@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    let vPos: vec3<f32> = nodeSrc[index].position;
    let mass = f32(atomicLoad(&nodeSrc[index].mass));

    // TODO: Global Param
    let scaling_ratio = 0.0002;

    var spring_force = vec3<f32>(0.0);
    spring_force.x = bitcast<f32>(atomicLoad(&springForceSrc[index * 3u + 0u]));
    spring_force.y = bitcast<f32>(atomicLoad(&springForceSrc[index * 3u + 1u]));
    spring_force.z = bitcast<f32>(atomicLoad(&springForceSrc[index * 3u + 2u]));

    atomicStore(&springForceSrc[index * 3u + 0u], 0);
    atomicStore(&springForceSrc[index * 3u + 1u], 0);
    atomicStore(&springForceSrc[index * 3u + 2u], 0);
    spring_force *= 100.0;

    nodeSrc[index].force += spring_force;
}

@compute
@workgroup_size(256)
fn displacement(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    // TODO: Global Param
    let global_speed = 1.0;

    let d_force = nodeSrc[index].force - nodeSrc[index].prev_force;
    let swg = sqrt(dot(d_force, d_force));
    let factor = global_speed / (1.0 + sqrt(global_speed * swg)) / f32(nodeSrc[index].mass);

    let force = nodeSrc[index].force;
    nodeSrc[index].force = vec3<f32>(0.0);
    nodeSrc[index].prev_force = force;

//    if (index == 0u) {
//        return;
//    }

    nodeSrc[index].position += force * factor * 0.01;
}

@compute
@workgroup_size(256)
fn randomize(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;

    vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.z = random_xy(index, 2u + 3u * uniforms.frame_num) * 2.0 - 1.0;

    // Write back
    nodeSrc[index].position = vPos;
    nodeSrc[index].force = vec3<f32>(0.0);
    nodeSrc[index].prev_force = vec3<f32>(0.0);
}

@compute
@workgroup_size(256)
fn copy(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;
    // var vVel : vec3<f32> = nodeSrc[index].velocity;

  // Write back
    //  nodeSrc[index] = Node(vPos, vVel);
}

@compute
@workgroup_size(256)
fn cal_depth(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let total = arrayLength(&nodeSrc);
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = nodeSrc[index].position;

    var clip_pos = transform.projection * transform.view * vec4<f32>(vPos, 1.0);

    kvps[index].index = index;
    kvps[index].sort_key = clip_pos.z;

}