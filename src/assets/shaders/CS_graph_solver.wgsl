struct Node {
    position: vec3<f32>,
    _empty_1: i32,
    force: vec3<f32>,
    _empty_2: i32,
    prev_force: vec3<f32>,
    mass: atomic<u32>,
};

struct Uniforms {
    frame_num: u32,
    node_count: u32,
    edge_count: u32,
    edge_sort_count: u32,
    tree_node_count: u32,
    bounding_count: u32,
    kernel_status_count: u32,
};

struct Bound {
    bound_min: vec3<f32>,
    bound_max: vec3<f32>,
}

struct BHTree {
    max_depth: atomic<u32>,
    bottom: atomic<u32>,
    radius: f32,
    _empty: i32,
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

struct KvpParam {
    dim: u32,
    block_count: u32,
}

struct Transform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    time: vec4<f32>,
    screen: vec4<f32>,
    camera: vec4<f32>,
}

struct NodeEdgeSortRange {
    min: atomic<u32>,
    max: atomic<u32>,
}

@group(0) @binding(0)  var<uniform>             uniforms:               Uniforms;
@group(0) @binding(1)  var<storage, read_write> node_src:               array<Node>;
@group(0) @binding(2)  var<storage, read>       edge_src:               array<vec2<u32>>;
@group(0) @binding(3)  var<storage, read_write> spring_force_src:       array<vec3<f32>>;
@group(0) @binding(4)  var<storage, read_write> bounding:               array<Bound>;
@group(0) @binding(5)  var<storage, read_write> bhTree:                 BHTree;
@group(0) @binding(6)  var<storage, read_write> tree_node_src:          array<BHTreeNode>;
@group(0) @binding(7)  var<storage, read_write> tree_child_src:         array<atomic<i32>>;
@group(0) @binding(8)  var<storage, read_write> kvps:                   array<Kvp>;
@group(0) @binding(9)  var<uniform>             kvps_param:             KvpParam;
@group(0) @binding(10) var<uniform>             transform:              Transform;
@group(0) @binding(11) var<storage, read_write> kernel_status:          array<i32>;
@group(0) @binding(12) var<storage, read_write> edge_sort_node:         array<vec2<u32>>;
@group(0) @binding(13) var<storage, read_write> edge_sort_dir:          array<vec3<f32>>;
@group(0) @binding(14) var<storage, read_write> node_edge_sort_range:   array<NodeEdgeSortRange>;
@group(0) @binding(15) var<storage, read_write> node_copy_src:          array<f32>;

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


@compute
@workgroup_size(256)
fn init_kernel_status(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let total = uniforms.kernel_status_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    kernel_status[index] = 0;

}


@compute
@workgroup_size(256)
fn gen_node(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = node_src[index].position;

    vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.z = random_xy(index, 2u + 3u * uniforms.frame_num) * 2.0 - 1.0;
//    vPos.x = 0.0;
//    vPos.y = 0.0;
//    vPos.z = 0.0;

    // Write back
    node_src[index].position = vPos;
    node_src[index].force = vec3<f32>(0.0);
    node_src[index].prev_force = vec3<f32>(0.0);
    node_src[index].mass = 1u;
    spring_force_src[index] = vec3<f32>(0.0);

    atomicStore(&node_edge_sort_range[index].min, 0u);
    atomicStore(&node_edge_sort_range[index].max, 0u);

}


@compute
@workgroup_size(256)
fn cal_mass(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.edge_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var edge = edge_src[index];
    let source_node: u32 = edge[0];
    let target_node: u32 = edge[1];

    atomicAdd(&node_src[source_node].mass, 1u);
    atomicAdd(&node_src[target_node].mass, 1u);
}


@compute
@workgroup_size(256)
fn cal_gravity_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    // TODO: Global Param
    let strong_gravity = true;
    let k_gravity = 1.0;

    let pos = node_src[index].position;
    let mass = f32(atomicLoad(&node_src[index].mass));
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
//    node_src[index].force +=  -pos * gravity_force;
//    node_src[index].force +=  -pos * min(gravity_force, 1.0);

    node_src[index].force +=  -pos * 0.5;
}

@compute
@workgroup_size(256)
fn prepare_edge_sort(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.edge_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var edge = edge_src[index];

    edge_sort_node[index * 2u] = edge;
    edge_sort_node[index * 2u + 1u] = vec2<u32>(edge[1], edge[0]);
}

@compute
@workgroup_size(256)
fn sort_edge(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let i = global_invocation_id.x;
    var j = i ^ kvps_param.block_count;

    if (kvps_param.block_count == kvps_param.dim >> 1u) {
        j = i ^ (kvps_param.block_count * 2u - 1u);
    }

    let total = uniforms.edge_sort_count;
    if (j < i || i >= total || j >= total) {
        return;
    }

    let edge_i = edge_sort_node[i];
    let edge_j = edge_sort_node[j];

    if (edge_j[0] < edge_i[0]) {
        edge_sort_node[i] = edge_j;
        edge_sort_node[j] = edge_i;
    }
}

@compute
@workgroup_size(256)
fn compute_node_edge_sort_range(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let i = global_invocation_id.x;

    let total = uniforms.edge_sort_count;
    if (i >= total) {
        return;
    }

    let node_index = edge_sort_node[i][0];

    atomicStore(&node_edge_sort_range[node_index].min, i);
    atomicStore(&node_edge_sort_range[node_index].max, i + 1u);

}
@compute
@workgroup_size(256)
fn compute_node_edge_sort_range_2(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let i = global_invocation_id.x;

    let total = uniforms.edge_sort_count;
    if (i >= total) {
        return;
    }

    let node_index = edge_sort_node[i][0];

    atomicMin(&node_edge_sort_range[node_index].min, i);
    atomicMax(&node_edge_sort_range[node_index].max, i + 1u);

}


var<workgroup> local_sum: array<vec3<f32>, 256>;

@compute
@workgroup_size(256)
fn spring_force_reduction(
    @builtin(local_invocation_index) local_index: u32,
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {

    var skip = false;

    var index = global_id.x;
    let total = uniforms.edge_sort_count;
    if (index >= total) {
        index = total - 1u;
    }

    var edge = edge_sort_node[index];
    let source_node: u32 = edge[0];
    let target_node: u32 = edge[1];
    var dir = node_src[target_node].position - node_src[source_node].position;
    local_sum[local_index] = dir;

    if (index >= total) {
        skip = true;
        local_sum[local_index] = vec3<f32>(0.0);
    }

    let range_min = atomicLoad(&node_edge_sort_range[source_node].min);
    let range_max = atomicLoad(&node_edge_sort_range[source_node].max);

    if (range_min >= range_max) {
        skip = true;
    }

    var node_relative_index: i32 = i32(index) - i32(range_min);
    let min_relative_index:  i32 = i32(local_index) - node_relative_index;
    let max_relative_index:  i32 = min_relative_index - i32(range_min) + i32(range_max);

    workgroupBarrier();

    var start = u32(max(min_relative_index, 0));
    var end = u32(min(max_relative_index, 256));

    for (var s = 256u / 2u; s > 0u; s >>= 1u) {

        if (!skip && local_index < start + s) {
            let k = local_index + s;
            if (k < end) {
                local_sum[local_index] += local_sum[k];
            }
        }
        workgroupBarrier();
    }

    if (skip) {
        return;
    }

    if (local_index == start) {
//        node_src[source_node].position += vec3<f32>(0.005, 0.0, 0.0);
//        let dir_sum = local_sum[local_index];
//        edge_sort_src[index].dir = dir_sum;
    }
        let dir_sum = local_sum[local_index];
        edge_sort_dir[index] = dir_sum;
}

@compute
@workgroup_size(256)
fn spring_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    let range_min = atomicLoad(&node_edge_sort_range[index].min);
    let range_max = atomicLoad(&node_edge_sort_range[index].max);

    if (range_min >= range_max) {
        return;
    }

    spring_force_src[index] += edge_sort_dir[range_min];
    for (var i = u32(range_min) - (u32(range_min) % 256u) + 256u; i < u32(range_max); i += 256u) {
        spring_force_src[index] += edge_sort_dir[i];
    }

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
    let total = uniforms.node_count;
    if (index >= total) {
        index = total - 1u;
    }

    smin[local_index] = node_src[index].position;
    smax[local_index] = node_src[index].position;
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

// 5
@compute
@workgroup_size(256)
fn reduction_bounding_2(
    @builtin(local_invocation_index) local_index: u32,
    @builtin(global_invocation_id) global_id: vec3<u32>,
    @builtin(workgroup_id) group_id: vec3<u32>,
) {

    var index = global_id.x;
    let total = uniforms.bounding_count;
    if (index >= total) {
        index = total - 1u;
    }

    smin[local_index] = bounding[index].bound_min;
    smax[local_index] = bounding[index].bound_max;
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

// 6
@compute
@workgroup_size(1)
fn bounding_box() {
    var bound_min_min = bounding[0].bound_min;
    var bound_max_max = bounding[0].bound_max;

    let box = bound_max_max - bound_min_min;
    let tree_node_count = uniforms.tree_node_count - 1u;
    bhTree.radius = max(max(box.x, box.y), box.z) * 0.5;
    atomicStore(&bhTree.bottom, tree_node_count);
    atomicStore(&bhTree.max_depth, 0u);
    atomicStore(&tree_node_src[tree_node_count].mass, -1);
    atomicStore(&tree_node_src[tree_node_count].start, 0);
    tree_node_src[tree_node_count].position = (bound_min_min + bound_max_max) * 0.5;
    tree_node_src[tree_node_count].count = -1;
    tree_node_src[tree_node_count].sort = -1;
}

// 7
@compute
@workgroup_size(256)
fn clear_1(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.tree_node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    for (var i = 0u; i < 8u; i++) {
        atomicStore(&tree_child_src[index * 8u + i], -1);
    }
}

// 8
@compute
@workgroup_size(256)
fn tree_building(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    var index = global_invocation_id.x;
    let node_count = uniforms.node_count;
    let tree_node_count = uniforms.tree_node_count - 1u;
    let root_pos = tree_node_src[tree_node_count].position;
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

    var loop_limit_count = 1000;

    while (index < node_count) {

        if (loop_limit_count < 0) {
            kernel_status[1] = -101;
            break;
        }
        loop_limit_count--;

        if (skip != 0) {
            skip = 0;
            pos = node_src[index].position;

            n = tree_node_count;
            r = root_r * 0.5;
            depth = 1u;

            let compare = step(root_pos, pos);
            j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u); // 八个象限
            dp = -r + compare * (2.0 * r);
            rdp = root_pos + dp; // 所在象限的原点
        }

        // atomicAdd(&tree_child_src[n * 8u + j], 0); // ...
        var ch = atomicLoad(&tree_child_src[n * 8u + j]);

        // 迭代至叶节点
        while (ch >= i32(node_count)) {
            n = u32(ch);
            depth++;
            r *= 0.5;

            let compare = step(rdp, pos);
            j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u);
            dp = -r + compare * (2.0 * r);

            rdp += dp;
            ch = atomicLoad(&tree_child_src[n * 8u + j]);
        }

        let locked = n * 8u + j;
        var locked_ch = -1;

        // 非 lock 状态
        if (ch != -2) {
            if (ch == -1) {
                var v = -1;
                let origin = atomicCompareExchangeWeak(&tree_child_src[locked], v, i32(index));
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
                let origin = atomicCompareExchangeWeak(&tree_child_src[locked], v, -2);
                if (ch == origin) {
                    // lock 成功，如果两个点的位置相同，做一点微小偏移就行了
                    if (all(node_src[ch].position == pos)) {
                        node_src[index].position += vec3<f32>(random_xy(index, 0u + 3u * uniforms.frame_num), random_xy(index, 1u + 3u * uniforms.frame_num), random_xy(index, 2u + 3u * uniforms.frame_num)) * 0.2 - 0.1;
                        skip = 0;
                        atomicStore(&tree_child_src[locked], ch);
                        kernel_status[0] = -3;
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
                            atomicStore(&tree_child_src[n * 8u + j], i32(cell));
                        }
                        locked_ch = max(locked_ch, i32(cell));

                        // 2. make newly created cell current
                        depth++;
                        n = cell;
                        r *= 0.5;

                        // 3. insert old body into current quadrant
                        let compare = step(rdp, node_src[ch].position);
                        j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u);

                        atomicStore(&tree_child_src[cell * 8u + j], ch);

                        // 4. determin center + quadrant for cell of new body
                        let compare = step(rdp, pos);
                        j = (u32(compare.x) << 0u) | (u32(compare.y) << 1u) + (u32(compare.z) << 2u);
                        dp = -r + compare * (2.0 * r);

                        rdp += dp;

                        // 5. visit this cell/chec if in use (possibly by old body)
                        ch = atomicLoad(&tree_child_src[n * 8u + j]);

                        if (ch < 0) {
                            break;
                        }

                    };
                    atomicStore(&tree_child_src[n * 8u + j], i32(index));
                    local_max_depth = max(depth, local_max_depth);
                    index += inc;
                    skip = 2;
                }
            }
        }
        workgroupBarrier();
        if (skip == 2) {
            atomicStore(&tree_child_src[locked], locked_ch);
        }
    }
    atomicMax(&bhTree.max_depth, local_max_depth);
}

// 9
@compute
@workgroup_size(256)
fn clear_2(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.tree_node_count - 1u;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }
    tree_node_src[index].position = vec3<f32>(0.0);
    tree_node_src[index].count = -1;
    tree_node_src[index].sort = -1;
    atomicStore(&tree_node_src[index].start, -1);
    atomicStore(&tree_node_src[index].mass, -1);
}

// 10
@compute
@workgroup_size(256)
fn summarization(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let bottom = atomicLoad(&bhTree.bottom);
    let tree_node_count = uniforms.tree_node_count - 1u;
    let node_count = uniforms.node_count;
    let inc = min(node_count, 16384u);
    var index = u32((i32(bottom) & -32) + i32(global_invocation_id.x));
    if (index < bottom) {
        index += inc;
    }

    // TODO: ch bounds check
    var schild: array<u32, 8>;
    var smass: array<i32, 8>;
    let restart = index;

    var loop_limit_count = 10000;

    for (var j = 0; j < 5; j++) {

        while (index <= tree_node_count) {

            if (loop_limit_count < 0) {
                kernel_status[2] = 500 + j;
                break;
            }
            loop_limit_count--;

            if (atomicLoad(&tree_node_src[index].mass) < 0) {
                var ch = 0u;
                var i = 0u;
                for (i = 0u; i < 8u; i++) {
                    ch = u32(atomicLoad(&tree_child_src[index * 8u + i]));
                    schild[i] = ch;
                    // atomicAdd(&tree_node_src[ch].mass, 0);
                    smass[i] = atomicLoad(&tree_node_src[ch].mass);
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
                            cnt += tree_node_src[ch].count;
                            pos += tree_node_src[ch].position * f32(m);
                            cm += m;
                        } else {
                            let m = i32(atomicLoad(&node_src[ch].mass));
                            cnt += 1;
                            pos += node_src[ch].position * f32(m);
                            cm += m;
                        }
                    }
                    tree_node_src[index].count = cnt;
                    tree_node_src[index].position = pos / f32(cm);
                    // workgroupBarrier();
                    atomicStore(&tree_node_src[index].mass, cm);
                }
            }
            index += inc;
        }
        index = restart;
    }

    var j = 0;
    var flag = false;
    while (index <= tree_node_count) {

        if (loop_limit_count < 0) {
            kernel_status[2] = 101;
            break;
        }
        loop_limit_count--;

        var cm = 0;
        if (index < node_count) {
            index += inc;
        } else if (index >= node_count && atomicLoad(&tree_node_src[index].mass) >= 0) {
            index += inc;
        } else {
            if (j == 0) {
                j = 8;
                for (var i = 0u; i < 8u; i++) {
                    let ch = u32(atomicLoad(&tree_child_src[index * 8u + i]));
                    schild[i] = ch;
                    smass[i] = atomicLoad(&tree_node_src[ch].mass);
                    if (ch < node_count || smass[i] >= 0) {
                        j--;
                    }
                }
            } else {
                j = 8;
                for (var i = 0u; i < 8u; i++) {
                    let ch = schild[i];
                    let old_mass = smass[i];
                    smass[i] = atomicLoad(&tree_node_src[ch].mass);
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
                        cnt += tree_node_src[ch].count;
                        pos += tree_node_src[ch].position * f32(m);
                        cm += m;
                    } else {
                        let m = i32(atomicLoad(&node_src[ch].mass));
                        cnt += 1;
                        pos += node_src[ch].position * f32(m);
                        cm += m;
                    }
                }
                tree_node_src[index].count = cnt;
                tree_node_src[index].position = pos / f32(cm);
                flag = true;
            }
        }
        // workgroupBarrier();
        if (flag) {
            if (index < node_count) {
                atomicStore(&node_src[index].mass, u32(cm));
            } else {
                atomicStore(&tree_node_src[index].mass, cm);
            }
            index += inc;
            flag = false;
        }
    }
}

// 11
@compute
@workgroup_size(256)
fn sort(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let tree_node_count = uniforms.tree_node_count - 1u;
    let bottom = atomicLoad(&bhTree.bottom);
    let node_count = uniforms.node_count;
    let inc = min(node_count, 16384u);
    var index = tree_node_count + 1u - inc + global_invocation_id.x;

    var loop_limit_count = 1000;

    while (index >= bottom) {

        if (loop_limit_count < 0) {
            kernel_status[3] = -101;
            break;
        }
        loop_limit_count--;

        workgroupBarrier();
        var start = atomicLoad(&tree_node_src[index].start);

        if (start >= 0) {
            var j = 0u;
            for (var i = 0u; i < 8u; i++) {
                let ch = atomicLoad(&tree_child_src[index * 8u + i]);
                if (ch >= 0) {
                    // 把子节点集中到开头
                    if (i != j) {
                        atomicStore(&tree_child_src[index * 8u + i], -1);
                        atomicStore(&tree_child_src[index * 8u + j], ch);
                    }
                    j++;
                    if (ch >= i32(node_count)) {
                        atomicStore(&tree_node_src[ch].start, start);
                        start += tree_node_src[ch].count;
                    } else {
                        tree_node_src[start].sort = ch;
                        start++;
                    }
                }
            }
            if (index < inc) {
                break;
            }
            index -= inc;
        }
//            if (index < inc) {
//                break;
//            }
//            index -= inc;
    }
}

// 12
@compute
@workgroup_size(256)
fn electron_force(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let tree_node_count = uniforms.tree_node_count - 1u;
    let node_count = uniforms.node_count;
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

    var loop_limit_count = 5000 * i32(node_count / inc);

    if (max_depth < 48u) {
        for (var index = global_invocation_id.x; index < node_count; index += inc) {
            var order = tree_node_src[index].sort;
            if (order < 0) { continue; }

            let pos = node_src[order].position;
            var af = vec3<f32>(0.0);

            var depth = 0u;
            spos[0] = 0u;
            snode[0] = tree_node_count;

            loop {

                if (loop_limit_count < 0) {
                    kernel_status[4] = 101;
                    break;
                }
                loop_limit_count--;

                var pd = spos[depth];
                var nd = snode[depth];
                while (pd < 8u) {

                    if (loop_limit_count < 0) {
                        kernel_status[4] = 101;
                        break;
                    }
                    loop_limit_count--;

                    let n_i32 = atomicLoad(&tree_child_src[nd * 8u + pd]);
                    pd++;

                    if (n_i32 >= 0) {
                        let n = u32(n_i32);
                        var dp: vec3<f32>;
                        if (n < node_count) {
                            dp = pos - node_src[n].position;
                        } else {
                            dp = pos - tree_node_src[n].position;
                        }
                        let dist2 = dot(dp, dp);

                        if (n < node_count) {
                            if (dist2 > 0.0) {
                                let factor = scale * f32(atomicLoad(&node_src[order].mass)) * f32(atomicLoad(&node_src[n].mass)) / dist2;
                                af += dp * factor;
                            }
                        } else if (dist2 >= sdq[depth]) {
                            if (dist2 > 0.0) {
                                let factor = scale * f32(atomicLoad(&node_src[order].mass)) * f32(atomicLoad(&tree_node_src[n].mass)) / dist2;
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
            node_src[order].force += af * 0.25;
        }
    }
}

// 13
@compute
@workgroup_size(256)
fn main(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    let vPos: vec3<f32> = node_src[index].position;
    let mass = f32(atomicLoad(&node_src[index].mass));

    // TODO: Global Param
    let scaling_ratio = 0.0002;

    var spring_force = spring_force_src[index];
    spring_force_src[index] = vec3<f32>(0.0);
    spring_force *= 100.0;

    node_src[index].force += spring_force;
}

// 14
@compute
@workgroup_size(256)
fn displacement(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    // TODO: Global Param
    let global_speed = 1.0;

    let d_force = node_src[index].force - node_src[index].prev_force;
    let swg = sqrt(dot(d_force, d_force));
    let factor = global_speed / (1.0 + sqrt(global_speed * swg)) / f32(node_src[index].mass);

    let force = node_src[index].force;
    node_src[index].force = vec3<f32>(0.0);
    node_src[index].prev_force = force;

//    if (index == 0u) {
//        return;
//    }

    node_src[index].position += force * factor * 0.01;
}

// 15
@compute
@workgroup_size(256)
fn randomize(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = node_src[index].position;

    vPos.x = random_xy(index, 0u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.y = random_xy(index, 1u + 3u * uniforms.frame_num) * 2.0 - 1.0;
    vPos.z = random_xy(index, 2u + 3u * uniforms.frame_num) * 2.0 - 1.0;

    // Write back
    node_src[index].position = vPos;
    node_src[index].force = vec3<f32>(0.0);
    node_src[index].prev_force = vec3<f32>(0.0);
}

// 16
@compute
@workgroup_size(256)
fn copy(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = node_src[index].position;

    node_copy_src[3u * index     ] = vPos.x;
    node_copy_src[3u * index + 1u] = vPos.y;
    node_copy_src[3u * index + 2u] = vPos.z;
}

// 17
@compute
@workgroup_size(256)
fn cal_depth(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {

    let total = uniforms.node_count;
    let index = global_invocation_id.x;
    if (index >= total) {
        return;
    }

    var vPos : vec3<f32> = node_src[index].position;

    var clip_pos = transform.projection * transform.view * vec4<f32>(vPos, 1.0);

    kvps[index].index = index;
    kvps[index].sort_key = clip_pos.z;

}

// 18
@compute
@workgroup_size(256)
fn sort_by_depth(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let i = global_invocation_id.x;
    var j = i ^ kvps_param.block_count;

    if (kvps_param.block_count == kvps_param.dim >> 1u) {
        j = i ^ (kvps_param.block_count * 2u - 1u);
    }
    
    let total = uniforms.node_count;
    if (j < i || i >= total || j >= total) {
        return;
    }


    let index_i= kvps[i].index;
    let index_j = kvps[j].index;
    let key_i = kvps[index_i].sort_key;
    let key_j = kvps[index_j].sort_key;
    
    var diff = key_j - key_i;

    if (diff > 0.0) {
        kvps[i].index = index_j;
        kvps[j].index = index_i;
    }
}