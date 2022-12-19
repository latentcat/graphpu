// language: metal2.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size1;
    uint size2;
    uint size3;
    uint size4;
    uint size6;
    uint size7;
};

struct Node {
    metal::float3 position;
    metal::float3 force;
    metal::packed_float3 prev_force;
    metal::atomic_uint mass;
};
struct Uniforms {
    uint frame_num;
};
struct Bound {
    metal::float3 bound_min;
    metal::float3 bound_max;
};
struct BHTree {
    metal::atomic_uint max_depth;
    metal::atomic_uint bottom;
    float radius;
};
struct BHTreeNode {
    metal::packed_float3 position;
    int _empty;
    metal::atomic_int mass;
    int count;
    metal::atomic_int start;
    int sort;
};
typedef Node type_6[1];
typedef metal::uint2 type_8[1];
typedef metal::atomic_int type_9[1];
typedef Bound type_10[1];
typedef BHTreeNode type_11[1];
struct type_13 {
    metal::float3 inner[256];
};
struct type_14 {
    uint inner[8];
};
struct type_15 {
    int inner[8];
};
struct type_17 {
    uint inner[48];
};
struct type_18 {
    float inner[48];
};

uint hash(
    uint s_1
) {
    uint t = {};
    t = s_1;
    uint _e10 = t;
    t = _e10 ^ 2747636419u;
    uint _e13 = t;
    t = _e13 * 2654435769u;
    uint _e16 = t;
    uint _e17 = t;
    t = _e16 ^ (_e17 >> 16u);
    uint _e21 = t;
    t = _e21 * 2654435769u;
    uint _e24 = t;
    uint _e25 = t;
    t = _e24 ^ (_e25 >> 16u);
    uint _e29 = t;
    t = _e29 * 2654435769u;
    uint _e32 = t;
    return _e32;
}

float random(
    uint seed
) {
    uint _e9 = hash(seed);
    return static_cast<float>(_e9) / 4294967296.0;
}

float random_xy(
    uint seed_x,
    uint seed_y
) {
    uint _e10 = hash(seed_x);
    uint _e12 = hash(_e10 + seed_y);
    return static_cast<float>(_e12) / 4294967296.0;
}

void atomic_add_f32_(
    uint springIndex,
    float updateValue,
    device type_9& springForceSrc,
    constant _mslBufferSizes& _buffer_sizes
) {
    int new_u32_ = {};
    int assumed = 0;
    int origin = {};
    new_u32_ = as_type<int>(updateValue);
    while(true) {
        if (true) {
        } else {
            break;
        }
        int _e17 = assumed;
        int _e18 = new_u32_;
        metal::atomic_compare_exchange_weak_explicit(&springForceSrc[springIndex], &_e17, _e18, metal::memory_order_relaxed, metal::memory_order_relaxed);
        origin = _e17;
        int _e20 = origin;
        int _e21 = assumed;
        if (_e20 == _e21) {
            break;
        }
        int _e23 = origin;
        assumed = _e23;
        int _e24 = origin;
        new_u32_ = as_type<int>(as_type<float>(_e24) + updateValue);
    }
    return;
}

struct gen_nodeInput {
};
kernel void gen_node(
  metal::uint3 global_invocation_id [[thread_position_in_grid]]
, constant Uniforms& uniforms [[user(fake0)]]
, device type_6& nodeSrc [[user(fake0)]]
, device type_9& springForceSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::float3 vPos = {};
    uint total = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint index_5 = global_invocation_id.x;
    if (index_5 >= total) {
        return;
    }
    metal::float3 _e14 = nodeSrc[index_5].position;
    vPos = _e14;
    uint _e20 = uniforms.frame_num;
    float _e23 = random_xy(index_5, 0u + (3u * _e20));
    vPos.x = (_e23 * 2.0) - 1.0;
    uint _e32 = uniforms.frame_num;
    float _e35 = random_xy(index_5, 1u + (3u * _e32));
    vPos.y = (_e35 * 2.0) - 1.0;
    uint _e44 = uniforms.frame_num;
    float _e47 = random_xy(index_5, 2u + (3u * _e44));
    vPos.z = (_e47 * 2.0) - 1.0;
    metal::float3 _e54 = vPos;
    nodeSrc[index_5].position = _e54;
    nodeSrc[index_5].force = metal::float3(0.0);
    nodeSrc[index_5].prev_force = metal::float3(0.0);
    metal::atomic_store_explicit(&nodeSrc[index_5].mass, 1u, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&springForceSrc[(index_5 * 3u) + 0u], 0, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&springForceSrc[(index_5 * 3u) + 1u], 0, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&springForceSrc[(index_5 * 3u) + 2u], 0, metal::memory_order_relaxed);
    return;
}


struct cal_massInput {
};
kernel void cal_mass(
  metal::uint3 global_invocation_id_1 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, device type_8 const& edgeSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::uint2 edge = {};
    uint total_1 = 1 + (_buffer_sizes.size2 - 0 - 8) / 8;
    uint index_6 = global_invocation_id_1.x;
    if (index_6 >= total_1) {
        return;
    }
    metal::uint2 _e13 = edgeSrc[index_6];
    edge = _e13;
    uint source_node = edge.x;
    uint target_node = edge.y;
    uint _e24 = metal::atomic_fetch_add_explicit(&nodeSrc[source_node].mass, 1u, metal::memory_order_relaxed);
    uint _e28 = metal::atomic_fetch_add_explicit(&nodeSrc[target_node].mass, 1u, metal::memory_order_relaxed);
    return;
}


struct cal_gravity_forceInput {
};
kernel void cal_gravity_force(
  metal::uint3 global_invocation_id_2 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    float gravity_force = {};
    uint total_2 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint index_7 = global_invocation_id_2.x;
    if (index_7 >= total_2) {
        return;
    }
    metal::float3 pos_3 = nodeSrc[index_7].position;
    uint _e19 = metal::atomic_load_explicit(&nodeSrc[index_7].mass, metal::memory_order_relaxed);
    float mass = static_cast<float>(_e19);
    if (true) {
        gravity_force = 0.25 * mass;
    } else {
        if (((pos_3.x != 0.0) || (pos_3.y != 0.0)) || (pos_3.z != 0.0)) {
            gravity_force = (0.25 * mass) * metal::rsqrt(metal::dot(pos_3, pos_3));
        } else {
            gravity_force = 0.0;
        }
    }
    metal::float3 _e41 = nodeSrc[index_7].force;
    nodeSrc[index_7].force = _e41 + (-pos_3 * 0.25);
    return;
}


struct attractive_forceInput {
};
kernel void attractive_force(
  metal::uint3 global_invocation_id_3 [[thread_position_in_grid]]
, device type_6 const& nodeSrc [[user(fake0)]]
, device type_8 const& edgeSrc [[user(fake0)]]
, device type_9& springForceSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::uint2 edge_1 = {};
    metal::float3 dir = {};
    uint total_3 = 1 + (_buffer_sizes.size2 - 0 - 8) / 8;
    uint index_8 = global_invocation_id_3.x;
    if (index_8 >= total_3) {
        return;
    }
    metal::uint2 _e13 = edgeSrc[index_8];
    edge_1 = _e13;
    uint source_node_1 = edge_1.x;
    uint target_node_1 = edge_1.y;
    metal::float3 _e23 = nodeSrc[target_node_1].position;
    metal::float3 _e26 = nodeSrc[source_node_1].position;
    dir = _e23 - _e26;
    metal::float3 _e29 = dir;
    dir = _e29 * 0.009999999776482582;
    float _e37 = dir.x;
    atomic_add_f32_((source_node_1 * 3u) + 0u, _e37, springForceSrc, _buffer_sizes);
    float _e43 = dir.y;
    atomic_add_f32_((source_node_1 * 3u) + 1u, _e43, springForceSrc, _buffer_sizes);
    float _e49 = dir.z;
    atomic_add_f32_((source_node_1 * 3u) + 2u, _e49, springForceSrc, _buffer_sizes);
    float _e55 = dir.x;
    atomic_add_f32_((target_node_1 * 3u) + 0u, -_e55, springForceSrc, _buffer_sizes);
    float _e62 = dir.y;
    atomic_add_f32_((target_node_1 * 3u) + 1u, -_e62, springForceSrc, _buffer_sizes);
    float _e69 = dir.z;
    atomic_add_f32_((target_node_1 * 3u) + 2u, -_e69, springForceSrc, _buffer_sizes);
    return;
}


struct reduction_boundingInput {
};
kernel void reduction_bounding(
  uint local_index [[thread_index_in_threadgroup]]
, metal::uint3 global_id [[thread_position_in_grid]]
, metal::uint3 group_id [[threadgroup_position_in_grid]]
, device type_6 const& nodeSrc [[user(fake0)]]
, device type_10& bounding [[user(fake0)]]
, threadgroup type_13& smin
, threadgroup type_13& smax
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint index = {};
    uint s = {};
    index = global_id.x;
    uint total_4 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint _e16 = index;
    if (_e16 >= total_4) {
        index = total_4 - 1u;
    }
    uint _e21 = index;
    metal::float3 _e24 = nodeSrc[_e21].position;
    smin.inner[local_index] = _e24;
    uint _e26 = index;
    metal::float3 _e29 = nodeSrc[_e26].position;
    smax.inner[local_index] = _e29;
    metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    s = 256u / 2u;
    bool loop_init = true;
    while(true) {
        if (!loop_init) {
            uint _e37 = s;
            s = _e37 >> 1u;
        }
        loop_init = false;
        uint _e34 = s;
        if (_e34 > 0u) {
        } else {
            break;
        }
        uint _e40 = s;
        if (local_index < _e40) {
            uint _e42 = s;
            uint k = local_index + _e42;
            metal::float3 _e46 = smin.inner[local_index];
            metal::float3 _e48 = smin.inner[k];
            smin.inner[local_index] = metal::min(_e46, _e48);
            metal::float3 _e52 = smax.inner[local_index];
            metal::float3 _e54 = smax.inner[k];
            smax.inner[local_index] = metal::max(_e52, _e54);
        }
        metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
    }
    if (local_index == 0u) {
        metal::float3 _e63 = smin.inner[0];
        bounding[group_id.x].bound_min = _e63;
        metal::float3 _e69 = smax.inner[0];
        bounding[group_id.x].bound_max = _e69;
        return;
    } else {
        return;
    }
}


kernel void bounding_box(
  device type_6 const& nodeSrc [[user(fake0)]]
, device type_10& bounding [[user(fake0)]]
, device BHTree& bhTree [[user(fake0)]]
, device type_11& treeNode [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::float3 bound_min_min = {};
    metal::float3 bound_max_max = {};
    uint i = 0u;
    metal::float3 _e13 = bounding[0].bound_min;
    bound_min_min = _e13;
    metal::float3 _e18 = bounding[0].bound_max;
    bound_max_max = _e18;
    uint node_group_count = static_cast<uint>(metal::ceil(static_cast<float>(1 + (_buffer_sizes.size1 - 0 - 48) / 48) / 256.0));
    bool loop_init_1 = true;
    while(true) {
        if (!loop_init_1) {
            uint _e30 = i;
            i = _e30 + 1u;
        }
        loop_init_1 = false;
        uint _e28 = i;
        if (_e28 < node_group_count) {
        } else {
            break;
        }
        metal::float3 _e33 = bound_min_min;
        uint _e34 = i;
        metal::float3 _e37 = bounding[_e34].bound_min;
        bound_min_min = metal::min(_e33, _e37);
        metal::float3 _e39 = bound_max_max;
        uint _e40 = i;
        metal::float3 _e43 = bounding[_e40].bound_max;
        bound_max_max = metal::max(_e39, _e43);
    }
    metal::float3 _e48 = bound_min_min;
    bounding[0].bound_min = _e48;
    metal::float3 _e52 = bound_max_max;
    bounding[0].bound_max = _e52;
    metal::float3 _e53 = bound_max_max;
    metal::float3 _e54 = bound_min_min;
    metal::float3 box = _e53 - _e54;
    uint tree_node_count = (1 + (_buffer_sizes.size6 - 0 - 32) / 32) - 1u;
    bhTree.radius = metal::max(metal::max(box.x, box.y), box.z) * 0.5;
    metal::atomic_store_explicit(&bhTree.bottom, tree_node_count, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&bhTree.max_depth, 0u, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&treeNode[tree_node_count].mass, -1, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&treeNode[tree_node_count].start, 0, metal::memory_order_relaxed);
    metal::float3 _e78 = bound_min_min;
    metal::float3 _e79 = bound_max_max;
    treeNode[tree_node_count].position = (_e78 + _e79) * 0.5;
    treeNode[tree_node_count].count = -1;
    treeNode[tree_node_count].sort = 0;
    return;
}


struct clear_1_Input {
};
kernel void clear_1_(
  metal::uint3 global_invocation_id_4 [[thread_position_in_grid]]
, device type_11 const& treeNode [[user(fake0)]]
, device type_9& treeChild [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint i_1 = 0u;
    uint total_5 = 1 + (_buffer_sizes.size6 - 0 - 32) / 32;
    uint index_9 = global_invocation_id_4.x;
    if (index_9 >= total_5) {
        return;
    }
    bool loop_init_2 = true;
    while(true) {
        if (!loop_init_2) {
            uint _e19 = i_1;
            i_1 = _e19 + 1u;
        }
        loop_init_2 = false;
        uint _e16 = i_1;
        if (_e16 < 8u) {
        } else {
            break;
        }
        uint _e24 = i_1;
        metal::atomic_store_explicit(&treeChild[(index_9 * 8u) + _e24], -1, metal::memory_order_relaxed);
    }
    return;
}


struct tree_buildingInput {
};
kernel void tree_building(
  metal::uint3 global_invocation_id_5 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, device BHTree& bhTree [[user(fake0)]]
, device type_11 const& treeNode [[user(fake0)]]
, device type_9& treeChild [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint index_1 = {};
    int skip = 1;
    metal::float3 pos = {};
    metal::float3 dp = {};
    metal::float3 rdp = {};
    uint n = {};
    uint depth = 1u;
    uint local_max_depth = 1u;
    uint j = 0u;
    float root_r = {};
    float r = {};
    int limit = 10000;
    int ch = {};
    int locked_ch = {};
    int v = {};
    int v_1 = {};
    index_1 = global_invocation_id_5.x;
    uint node_count = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint tree_node_count_1 = (1 + (_buffer_sizes.size6 - 0 - 32) / 32) - 1u;
    metal::float3 root_pos = treeNode[tree_node_count_1].position;
    uint inc = metal::min(node_count, 16384u);
    n = tree_node_count_1;
    float _e35 = bhTree.radius;
    root_r = _e35;
    float _e37 = root_r;
    r = _e37 * 0.5;
    while(true) {
        uint _e43 = index_1;
        if (_e43 < node_count) {
        } else {
            break;
        }
        int _e45 = limit;
        limit = _e45 - 1;
        int _e48 = limit;
        if (_e48 < 0) {
            return;
        }
        int _e51 = skip;
        if (_e51 != 0) {
            skip = 0;
            uint _e55 = index_1;
            metal::float3 _e58 = nodeSrc[_e55].position;
            pos = _e58;
            n = tree_node_count_1;
            float _e59 = root_r;
            r = _e59 * 0.5;
            depth = 1u;
            metal::float3 _e63 = pos;
            metal::float3 compare = metal::step(root_pos, _e63);
            j = (static_cast<uint>(compare.x) << 0u) | ((static_cast<uint>(compare.y) << 1u) + (static_cast<uint>(compare.z) << 2u));
            float _e79 = r;
            float _e82 = r;
            dp = metal::float3(-_e79) + (compare * (2.0 * _e82));
            metal::float3 _e87 = dp;
            rdp = root_pos + _e87;
        }
        uint _e89 = n;
        uint _e92 = j;
        int _e95 = metal::atomic_load_explicit(&treeChild[(_e89 * 8u) + _e92], metal::memory_order_relaxed);
        ch = _e95;
        while(true) {
            int _e97 = ch;
            if (_e97 >= static_cast<int>(node_count)) {
            } else {
                break;
            }
            int _e100 = ch;
            n = static_cast<uint>(_e100);
            uint _e102 = depth;
            depth = _e102 + 1u;
            float _e105 = r;
            r = _e105 * 0.5;
            metal::float3 _e108 = rdp;
            metal::float3 _e109 = pos;
            metal::float3 compare_1 = metal::step(_e108, _e109);
            j = (static_cast<uint>(compare_1.x) << 0u) | ((static_cast<uint>(compare_1.y) << 1u) + (static_cast<uint>(compare_1.z) << 2u));
            float _e125 = r;
            float _e128 = r;
            dp = metal::float3(-_e125) + (compare_1 * (2.0 * _e128));
            metal::float3 _e133 = rdp;
            metal::float3 _e134 = dp;
            rdp = _e133 + _e134;
            uint _e136 = n;
            uint _e139 = j;
            int _e142 = metal::atomic_load_explicit(&treeChild[(_e136 * 8u) + _e139], metal::memory_order_relaxed);
            ch = _e142;
        }
        uint _e143 = n;
        uint _e146 = j;
        uint locked = (_e143 * 8u) + _e146;
        locked_ch = -1;
        int _e150 = ch;
        if (_e150 != -2) {
            int _e153 = ch;
            if (_e153 == -1) {
                v = -1;
                int _e159 = v;
                uint _e160 = index_1;
                metal::atomic_compare_exchange_weak_explicit(&treeChild[locked], &_e159, static_cast<int>(_e160), metal::memory_order_relaxed, metal::memory_order_relaxed);
                if (_e159 == -1) {
                    uint _e165 = depth;
                    uint _e166 = local_max_depth;
                    local_max_depth = metal::max(_e165, _e166);
                    uint _e168 = index_1;
                    index_1 = _e168 + inc;
                    skip = 1;
                } else {
                    skip = 0;
                }
            } else {
                int _e172 = ch;
                v_1 = _e172;
                int _e175 = v_1;
                metal::atomic_compare_exchange_weak_explicit(&treeChild[locked], &_e175, -2, metal::memory_order_relaxed, metal::memory_order_relaxed);
                int _e178 = ch;
                if (_e178 == _e175) {
                    int _e180 = ch;
                    metal::float3 _e183 = nodeSrc[_e180].position;
                    metal::float3 _e184 = pos;
                    if (metal::all(_e183 == _e184)) {
                        uint _e187 = index_1;
                        metal::float3 _e190 = nodeSrc[_e187].position;
                        nodeSrc[_e187].position = _e190 + metal::float3(0.10000000149011612, -0.05000000074505806, 0.10000000149011612);
                        skip = 0;
                        int _e198 = ch;
                        metal::atomic_store_explicit(&treeChild[locked], _e198, metal::memory_order_relaxed);
                        break;
                    }
                    locked_ch = -1;
                    while(true) {
                        uint _e202 = metal::atomic_fetch_sub_explicit(&bhTree.bottom, 1u, metal::memory_order_relaxed);
                        uint cell = _e202 - 1u;
                        if (cell <= node_count) {
                            return;
                        }
                        int _e206 = locked_ch;
                        if (_e206 != -1) {
                            uint _e209 = n;
                            uint _e212 = j;
                            metal::atomic_store_explicit(&treeChild[(_e209 * 8u) + _e212], static_cast<int>(cell), metal::memory_order_relaxed);
                        }
                        int _e216 = locked_ch;
                        locked_ch = metal::max(_e216, static_cast<int>(cell));
                        uint _e219 = depth;
                        depth = _e219 + 1u;
                        n = cell;
                        float _e222 = r;
                        r = _e222 * 0.5;
                        metal::float3 _e225 = rdp;
                        int _e226 = ch;
                        metal::float3 _e229 = nodeSrc[_e226].position;
                        metal::float3 compare_2 = metal::step(_e225, _e229);
                        j = (static_cast<uint>(compare_2.x) << 0u) | ((static_cast<uint>(compare_2.y) << 1u) + (static_cast<uint>(compare_2.z) << 2u));
                        uint _e247 = j;
                        int _e250 = ch;
                        metal::atomic_store_explicit(&treeChild[(cell * 8u) + _e247], _e250, metal::memory_order_relaxed);
                        metal::float3 _e251 = rdp;
                        metal::float3 _e252 = pos;
                        metal::float3 compare_3 = metal::step(_e251, _e252);
                        j = (static_cast<uint>(compare_3.x) << 0u) | ((static_cast<uint>(compare_3.y) << 1u) + (static_cast<uint>(compare_3.z) << 2u));
                        float _e268 = r;
                        float _e271 = r;
                        dp = metal::float3(-_e268) + (compare_3 * (2.0 * _e271));
                        metal::float3 _e276 = rdp;
                        metal::float3 _e277 = dp;
                        rdp = _e276 + _e277;
                        uint _e279 = n;
                        uint _e282 = j;
                        int _e285 = metal::atomic_load_explicit(&treeChild[(_e279 * 8u) + _e282], metal::memory_order_relaxed);
                        ch = _e285;
                        int _e286 = ch;
                        if (_e286 < 0) {
                            break;
                        }
                    }
                    uint _e289 = n;
                    uint _e292 = j;
                    uint _e295 = index_1;
                    metal::atomic_store_explicit(&treeChild[(_e289 * 8u) + _e292], static_cast<int>(_e295), metal::memory_order_relaxed);
                    uint _e297 = depth;
                    uint _e298 = local_max_depth;
                    local_max_depth = metal::max(_e297, _e298);
                    uint _e300 = index_1;
                    index_1 = _e300 + inc;
                    skip = 2;
                }
            }
        }
        metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
        int _e303 = skip;
        if (_e303 == 2) {
            int _e307 = locked_ch;
            metal::atomic_store_explicit(&treeChild[locked], _e307, metal::memory_order_relaxed);
        }
    }
    uint _e309 = local_max_depth;
    uint _e310 = metal::atomic_fetch_max_explicit(&bhTree.max_depth, _e309, metal::memory_order_relaxed);
    return;
}


struct clear_2_Input {
};
kernel void clear_2_(
  metal::uint3 global_invocation_id_6 [[thread_position_in_grid]]
, device type_11& treeNode [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint total_6 = (1 + (_buffer_sizes.size6 - 0 - 32) / 32) - 1u;
    uint index_10 = global_invocation_id_6.x;
    if (index_10 >= total_6) {
        return;
    }
    treeNode[index_10].position = metal::float3(0.0);
    treeNode[index_10].count = -1;
    treeNode[index_10].sort = 0;
    metal::atomic_store_explicit(&treeNode[index_10].start, -1, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&treeNode[index_10].mass, -1, metal::memory_order_relaxed);
    return;
}


struct summarizationInput {
};
kernel void summarization(
  metal::uint3 global_invocation_id_7 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, device BHTree const& bhTree [[user(fake0)]]
, device type_11& treeNode [[user(fake0)]]
, device type_9 const& treeChild [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint index_2 = {};
    int cm = 0;
    type_14 schild = {};
    type_15 smass = {};
    int j_1 = 0;
    uint ch_1 = {};
    uint i_2 = {};
    metal::float3 pos_1 = {};
    int cnt = {};
    int j_2 = 0;
    bool flag = false;
    uint i_3 = {};
    uint i_4 = {};
    metal::float3 pos_2 = {};
    int cnt_1 = {};
    uint i_5 = {};
    uint bottom = metal::atomic_load_explicit(&bhTree.bottom, metal::memory_order_relaxed);
    uint tree_node_count_2 = (1 + (_buffer_sizes.size6 - 0 - 32) / 32) - 1u;
    uint node_count_1 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint inc_1 = metal::min(node_count_1, 16384u);
    index_2 = static_cast<uint>((static_cast<int>(bottom) & -32) + static_cast<int>(global_invocation_id_7.x));
    uint _e27 = index_2;
    if (_e27 < bottom) {
        uint _e29 = index_2;
        index_2 = _e29 + inc_1;
    }
    uint restart = index_2;
    bool loop_init_3 = true;
    while(true) {
        if (!loop_init_3) {
            int _e41 = j_1;
            j_1 = _e41 + 1;
        }
        loop_init_3 = false;
        int _e38 = j_1;
        if (_e38 < 5) {
        } else {
            break;
        }
        while(true) {
            uint _e44 = index_2;
            if (_e44 <= tree_node_count_2) {
            } else {
                break;
            }
            uint _e46 = index_2;
            int _e49 = metal::atomic_load_explicit(&treeNode[_e46].mass, metal::memory_order_relaxed);
            if (_e49 < 0) {
                ch_1 = 0u;
                i_2 = 0u;
                i_2 = 0u;
                bool loop_init_4 = true;
                while(true) {
                    if (!loop_init_4) {
                        uint _e60 = i_2;
                        i_2 = _e60 + 1u;
                    }
                    loop_init_4 = false;
                    uint _e57 = i_2;
                    if (_e57 < 8u) {
                    } else {
                        break;
                    }
                    uint _e63 = index_2;
                    uint _e66 = i_2;
                    int _e69 = metal::atomic_load_explicit(&treeChild[(_e63 * 8u) + _e66], metal::memory_order_relaxed);
                    ch_1 = static_cast<uint>(_e69);
                    uint _e71 = i_2;
                    uint _e73 = ch_1;
                    schild.inner[_e71] = _e73;
                    uint _e74 = i_2;
                    uint _e76 = ch_1;
                    int _e79 = metal::atomic_load_explicit(&treeNode[_e76].mass, metal::memory_order_relaxed);
                    smass.inner[_e74] = _e79;
                    uint _e80 = ch_1;
                    uint _e82 = i_2;
                    int _e84 = smass.inner[_e82];
                    if ((_e80 >= node_count_1) && (_e84 < 0)) {
                        break;
                    }
                }
                uint _e88 = i_2;
                if (_e88 == 8u) {
                    pos_1 = metal::float3(0.0);
                    cnt = 0;
                    i_2 = 0u;
                    bool loop_init_5 = true;
                    while(true) {
                        if (!loop_init_5) {
                            uint _e100 = i_2;
                            i_2 = _e100 + 1u;
                        }
                        loop_init_5 = false;
                        uint _e97 = i_2;
                        if (_e97 < 8u) {
                        } else {
                            break;
                        }
                        uint _e103 = i_2;
                        uint _e105 = schild.inner[_e103];
                        ch_1 = _e105;
                        uint _e106 = ch_1;
                        if (_e106 >= node_count_1) {
                            uint _e108 = i_2;
                            int m = smass.inner[_e108];
                            int _e111 = cnt;
                            uint _e112 = ch_1;
                            int _e115 = treeNode[_e112].count;
                            cnt = _e111 + _e115;
                            metal::float3 _e117 = pos_1;
                            uint _e118 = ch_1;
                            metal::float3 _e121 = treeNode[_e118].position;
                            pos_1 = _e117 + (_e121 * static_cast<float>(m));
                            int _e125 = cm;
                            cm = _e125 + m;
                        } else {
                            uint _e127 = ch_1;
                            uint _e130 = metal::atomic_load_explicit(&nodeSrc[_e127].mass, metal::memory_order_relaxed);
                            int m_1 = static_cast<int>(_e130);
                            int _e132 = cnt;
                            cnt = _e132 + 1;
                            metal::float3 _e135 = pos_1;
                            uint _e136 = ch_1;
                            metal::float3 _e139 = nodeSrc[_e136].position;
                            pos_1 = _e135 + (_e139 * static_cast<float>(m_1));
                            int _e143 = cm;
                            cm = _e143 + m_1;
                        }
                    }
                    uint _e145 = index_2;
                    int _e148 = cnt;
                    treeNode[_e145].count = _e148;
                    uint _e149 = index_2;
                    metal::float3 _e152 = pos_1;
                    int _e153 = cm;
                    treeNode[_e149].position = _e152 / metal::float3(static_cast<float>(_e153));
                    uint _e157 = index_2;
                    int _e160 = cm;
                    metal::atomic_store_explicit(&treeNode[_e157].mass, _e160, metal::memory_order_relaxed);
                }
            }
            uint _e161 = index_2;
            index_2 = _e161 + inc_1;
        }
        index_2 = restart;
    }
    while(true) {
        uint _e167 = index_2;
        if (_e167 <= tree_node_count_2) {
        } else {
            break;
        }
        uint _e169 = index_2;
        uint _e173 = index_2;
        uint _e175 = index_2;
        int _e178 = metal::atomic_load_explicit(&treeNode[_e175].mass, metal::memory_order_relaxed);
        if (_e169 < node_count_1) {
            uint _e171 = index_2;
            index_2 = _e171 + inc_1;
        } else {
            if ((_e173 >= node_count_1) && (_e178 >= 0)) {
                uint _e182 = index_2;
                index_2 = _e182 + inc_1;
            } else {
                int _e184 = j_2;
                if (_e184 == 0) {
                    j_2 = 8;
                    i_3 = 0u;
                    bool loop_init_6 = true;
                    while(true) {
                        if (!loop_init_6) {
                            uint _e193 = i_3;
                            i_3 = _e193 + 1u;
                        }
                        loop_init_6 = false;
                        uint _e190 = i_3;
                        if (_e190 < 8u) {
                        } else {
                            break;
                        }
                        uint _e196 = index_2;
                        uint _e199 = i_3;
                        int _e202 = metal::atomic_load_explicit(&treeChild[(_e196 * 8u) + _e199], metal::memory_order_relaxed);
                        uint ch_2 = static_cast<uint>(_e202);
                        uint _e204 = i_3;
                        schild.inner[_e204] = ch_2;
                        uint _e206 = i_3;
                        int _e210 = metal::atomic_load_explicit(&treeNode[ch_2].mass, metal::memory_order_relaxed);
                        smass.inner[_e206] = _e210;
                        uint _e212 = i_3;
                        int _e214 = smass.inner[_e212];
                        if ((ch_2 < node_count_1) || (_e214 >= 0)) {
                            int _e218 = j_2;
                            j_2 = _e218 - 1;
                        }
                    }
                } else {
                    j_2 = 8;
                    i_4 = 0u;
                    bool loop_init_7 = true;
                    while(true) {
                        if (!loop_init_7) {
                            uint _e227 = i_4;
                            i_4 = _e227 + 1u;
                        }
                        loop_init_7 = false;
                        uint _e224 = i_4;
                        if (_e224 < 8u) {
                        } else {
                            break;
                        }
                        uint _e230 = i_4;
                        uint ch_3 = schild.inner[_e230];
                        uint _e233 = i_4;
                        int old_mass = smass.inner[_e233];
                        uint _e236 = i_4;
                        int _e240 = metal::atomic_load_explicit(&treeNode[ch_3].mass, metal::memory_order_relaxed);
                        smass.inner[_e236] = _e240;
                        uint _e245 = i_4;
                        int _e247 = smass.inner[_e245];
                        if (((ch_3 < node_count_1) || (old_mass >= 0)) || (_e247 >= 0)) {
                            int _e251 = j_2;
                            j_2 = _e251 - 1;
                        }
                    }
                }
                int _e254 = j_2;
                if (_e254 == 0) {
                    cm = 0;
                    pos_2 = metal::float3(0.0);
                    cnt_1 = 0;
                    i_5 = 0u;
                    bool loop_init_8 = true;
                    while(true) {
                        if (!loop_init_8) {
                            uint _e268 = i_5;
                            i_5 = _e268 + 1u;
                        }
                        loop_init_8 = false;
                        uint _e265 = i_5;
                        if (_e265 < 8u) {
                        } else {
                            break;
                        }
                        uint _e271 = i_5;
                        uint ch_4 = schild.inner[_e271];
                        if (ch_4 >= node_count_1) {
                            uint _e275 = i_5;
                            int m_2 = smass.inner[_e275];
                            int _e278 = cnt_1;
                            int _e281 = treeNode[ch_4].count;
                            cnt_1 = _e278 + _e281;
                            metal::float3 _e283 = pos_2;
                            metal::float3 _e286 = treeNode[ch_4].position;
                            pos_2 = _e283 + (_e286 * static_cast<float>(m_2));
                            int _e290 = cm;
                            cm = _e290 + m_2;
                        } else {
                            uint _e294 = metal::atomic_load_explicit(&nodeSrc[ch_4].mass, metal::memory_order_relaxed);
                            int m_3 = static_cast<int>(_e294);
                            int _e296 = cnt_1;
                            cnt_1 = _e296 + 1;
                            metal::float3 _e299 = pos_2;
                            metal::float3 _e302 = nodeSrc[ch_4].position;
                            pos_2 = _e299 + (_e302 * static_cast<float>(m_3));
                            int _e306 = cm;
                            cm = _e306 + m_3;
                        }
                    }
                    uint _e308 = index_2;
                    int _e311 = cnt_1;
                    treeNode[_e308].count = _e311;
                    uint _e312 = index_2;
                    metal::float3 _e315 = pos_2;
                    int _e316 = cm;
                    treeNode[_e312].position = _e315 / metal::float3(static_cast<float>(_e316));
                    flag = true;
                }
            }
        }
        bool _e321 = flag;
        if (_e321) {
            uint _e322 = index_2;
            if (_e322 < node_count_1) {
                uint _e324 = index_2;
                int _e327 = cm;
                metal::atomic_store_explicit(&nodeSrc[_e324].mass, static_cast<uint>(_e327), metal::memory_order_relaxed);
            } else {
                uint _e329 = index_2;
                int _e332 = cm;
                metal::atomic_store_explicit(&treeNode[_e329].mass, _e332, metal::memory_order_relaxed);
            }
            uint _e333 = index_2;
            index_2 = _e333 + inc_1;
            flag = false;
        }
    }
    return;
}


struct sortInput {
};
kernel void sort(
  metal::uint3 global_invocation_id_8 [[thread_position_in_grid]]
, device type_6 const& nodeSrc [[user(fake0)]]
, device BHTree const& bhTree [[user(fake0)]]
, device type_11& treeNode [[user(fake0)]]
, device type_9& treeChild [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint index_3 = {};
    int limit_1 = 1000;
    int start = {};
    uint j_3 = {};
    uint i_6 = {};
    uint tree_node_count_3 = (1 + (_buffer_sizes.size6 - 0 - 32) / 32) - 1u;
    uint bottom_1 = metal::atomic_load_explicit(&bhTree.bottom, metal::memory_order_relaxed);
    uint node_count_2 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint inc_2 = metal::min(node_count_2, 16384u);
    index_3 = ((tree_node_count_3 + 1u) - inc_2) + global_invocation_id_8.x;
    while(true) {
        uint _e27 = index_3;
        if (_e27 >= bottom_1) {
        } else {
            break;
        }
        int _e29 = limit_1;
        limit_1 = _e29 - 1;
        int _e32 = limit_1;
        if (_e32 < 0) {
            uint _e35 = index_3;
            metal::atomic_store_explicit(&treeChild[_e35], 1000, metal::memory_order_relaxed);
            metal::atomic_store_explicit(&treeChild[0], 1000, metal::memory_order_relaxed);
            return;
        }
        metal::threadgroup_barrier(metal::mem_flags::mem_threadgroup);
        uint _e41 = index_3;
        int _e44 = metal::atomic_load_explicit(&treeNode[_e41].start, metal::memory_order_relaxed);
        start = _e44;
        int _e46 = start;
        if (_e46 >= 0) {
            j_3 = 0u;
            i_6 = 0u;
            bool loop_init_9 = true;
            while(true) {
                if (!loop_init_9) {
                    uint _e56 = i_6;
                    i_6 = _e56 + 1u;
                }
                loop_init_9 = false;
                uint _e53 = i_6;
                if (_e53 < 8u) {
                } else {
                    break;
                }
                uint _e59 = index_3;
                uint _e62 = i_6;
                int ch_5 = metal::atomic_load_explicit(&treeChild[(_e59 * 8u) + _e62], metal::memory_order_relaxed);
                if (ch_5 >= 0) {
                    uint _e68 = i_6;
                    uint _e69 = j_3;
                    if (_e68 != _e69) {
                        uint _e71 = index_3;
                        uint _e74 = i_6;
                        metal::atomic_store_explicit(&treeChild[(_e71 * 8u) + _e74], -1, metal::memory_order_relaxed);
                        uint _e78 = index_3;
                        uint _e81 = j_3;
                        metal::atomic_store_explicit(&treeChild[(_e78 * 8u) + _e81], ch_5, metal::memory_order_relaxed);
                    }
                    uint _e84 = j_3;
                    j_3 = _e84 + 1u;
                    if (ch_5 >= static_cast<int>(node_count_2)) {
                        int _e91 = start;
                        metal::atomic_store_explicit(&treeNode[ch_5].start, _e91, metal::memory_order_relaxed);
                        int _e92 = start;
                        int _e95 = treeNode[ch_5].count;
                        start = _e92 + _e95;
                    } else {
                        int _e97 = start;
                        treeNode[_e97].sort = ch_5;
                        int _e100 = start;
                        start = _e100 + 1;
                    }
                }
            }
            uint _e103 = index_3;
            if (_e103 < inc_2) {
                break;
            }
            uint _e105 = index_3;
            index_3 = _e105 - inc_2;
        }
    }
    return;
}


struct electron_forceInput {
};
kernel void electron_force(
  metal::uint3 global_invocation_id_9 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, device BHTree const& bhTree [[user(fake0)]]
, device type_11 const& treeNode [[user(fake0)]]
, device type_9 const& treeChild [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    type_17 spos = {};
    type_17 snode = {};
    type_18 sdq = {};
    uint j_4 = 1u;
    uint index_4 = {};
    metal::float3 af = {};
    uint depth_1 = {};
    uint pd = {};
    uint nd = {};
    metal::float3 dp_1 = {};
    uint tree_node_count_4 = (1 + (_buffer_sizes.size6 - 0 - 32) / 32) - 1u;
    uint node_count_3 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint inc_3 = metal::min(node_count_3, 16384u);
    float epssq = 0.05000000074505806 * 0.05000000074505806;
    float _e26 = bhTree.radius;
    float diameter = _e26 * 2.0;
    uint max_depth = metal::atomic_load_explicit(&bhTree.max_depth, metal::memory_order_relaxed);
    sdq.inner[0] = (diameter * diameter) * 1.0;
    bool loop_init_10 = true;
    while(true) {
        if (!loop_init_10) {
            uint _e39 = j_4;
            j_4 = _e39 + 1u;
        }
        loop_init_10 = false;
        uint _e37 = j_4;
        if (_e37 < max_depth) {
        } else {
            break;
        }
        uint _e42 = j_4;
        uint _e44 = j_4;
        float _e48 = sdq.inner[_e44 - 1u];
        sdq.inner[_e42] = _e48 * 0.25;
        uint _e51 = j_4;
        float _e55 = sdq.inner[_e51 - 1u];
        sdq.inner[_e51 - 1u] = _e55 + epssq;
    }
    float _e60 = sdq.inner[max_depth - 1u];
    sdq.inner[max_depth - 1u] = _e60 + epssq;
    if (max_depth < 48u) {
        index_4 = global_invocation_id_9.x;
        bool loop_init_11 = true;
        while(true) {
            if (!loop_init_11) {
                uint _e68 = index_4;
                index_4 = _e68 + inc_3;
            }
            loop_init_11 = false;
            uint _e66 = index_4;
            if (_e66 < node_count_3) {
            } else {
                break;
            }
            uint _e70 = index_4;
            int order = treeNode[_e70].sort;
            metal::float3 pos_4 = nodeSrc[order].position;
            af = metal::float3(0.0);
            depth_1 = 0u;
            spos.inner[0] = 0u;
            snode.inner[0] = tree_node_count_4;
            while(true) {
                uint _e87 = depth_1;
                uint _e89 = spos.inner[_e87];
                pd = _e89;
                uint _e91 = depth_1;
                uint _e93 = snode.inner[_e91];
                nd = _e93;
                while(true) {
                    uint _e95 = pd;
                    if (_e95 < 8u) {
                    } else {
                        break;
                    }
                    uint _e98 = nd;
                    uint _e101 = pd;
                    int n_i32_ = metal::atomic_load_explicit(&treeChild[(_e98 * 8u) + _e101], metal::memory_order_relaxed);
                    uint _e105 = pd;
                    pd = _e105 + 1u;
                    if (n_i32_ >= 0) {
                        uint n_1 = static_cast<uint>(n_i32_);
                        if (n_1 < node_count_3) {
                            metal::float3 _e115 = nodeSrc[n_1].position;
                            dp_1 = pos_4 - _e115;
                        } else {
                            metal::float3 _e119 = treeNode[n_1].position;
                            dp_1 = pos_4 - _e119;
                        }
                        metal::float3 _e121 = dp_1;
                        metal::float3 _e122 = dp_1;
                        float dist2_ = metal::dot(_e121, _e122);
                        if (n_1 < node_count_3) {
                            if (dist2_ > 0.0) {
                                uint _e129 = metal::atomic_load_explicit(&nodeSrc[order].mass, metal::memory_order_relaxed);
                                uint _e134 = metal::atomic_load_explicit(&nodeSrc[n_1].mass, metal::memory_order_relaxed);
                                float factor = ((0.0003000000142492354 * static_cast<float>(_e129)) * static_cast<float>(_e134)) / dist2_;
                                metal::float3 _e138 = af;
                                metal::float3 _e139 = dp_1;
                                af = _e138 + (_e139 * factor);
                            }
                        } else {
                            uint _e142 = depth_1;
                            float _e144 = sdq.inner[_e142];
                            if (dist2_ >= _e144) {
                                if (dist2_ > 0.0) {
                                    uint _e150 = metal::atomic_load_explicit(&nodeSrc[order].mass, metal::memory_order_relaxed);
                                    int _e155 = metal::atomic_load_explicit(&treeNode[n_1].mass, metal::memory_order_relaxed);
                                    float factor_1 = ((0.0003000000142492354 * static_cast<float>(_e150)) * static_cast<float>(_e155)) / dist2_;
                                    metal::float3 _e159 = af;
                                    metal::float3 _e160 = dp_1;
                                    af = _e159 + (_e160 * factor_1);
                                }
                            } else {
                                uint _e163 = depth_1;
                                uint _e165 = pd;
                                spos.inner[_e163] = _e165;
                                uint _e166 = depth_1;
                                uint _e168 = nd;
                                snode.inner[_e166] = _e168;
                                uint _e169 = depth_1;
                                depth_1 = _e169 + 1u;
                                pd = 0u;
                                nd = n_1;
                            }
                        }
                    } else {
                        pd = 8u;
                    }
                }
                uint _e174 = depth_1;
                if (_e174 == 0u) {
                    break;
                }
                uint _e177 = depth_1;
                depth_1 = _e177 - 1u;
            }
            metal::float3 _e182 = nodeSrc[order].force;
            metal::float3 _e183 = af;
            nodeSrc[order].force = _e182 + (_e183 * 0.10000000149011612);
        }
        return;
    } else {
        return;
    }
}


struct main_Input {
};
kernel void main_(
  metal::uint3 global_invocation_id_10 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, device type_9& springForceSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::float3 spring_force = {};
    uint total_7 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint index_11 = global_invocation_id_10.x;
    if (index_11 >= total_7) {
        return;
    }
    metal::float3 vPos_3 = nodeSrc[index_11].position;
    uint _e19 = metal::atomic_load_explicit(&nodeSrc[index_11].mass, metal::memory_order_relaxed);
    float mass_1 = static_cast<float>(_e19);
    spring_force = metal::float3(0.0);
    int _e31 = metal::atomic_load_explicit(&springForceSrc[(index_11 * 3u) + 0u], metal::memory_order_relaxed);
    spring_force.x = as_type<float>(_e31);
    int _e39 = metal::atomic_load_explicit(&springForceSrc[(index_11 * 3u) + 1u], metal::memory_order_relaxed);
    spring_force.y = as_type<float>(_e39);
    int _e47 = metal::atomic_load_explicit(&springForceSrc[(index_11 * 3u) + 2u], metal::memory_order_relaxed);
    spring_force.z = as_type<float>(_e47);
    metal::atomic_store_explicit(&springForceSrc[(index_11 * 3u) + 0u], 0, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&springForceSrc[(index_11 * 3u) + 1u], 0, metal::memory_order_relaxed);
    metal::atomic_store_explicit(&springForceSrc[(index_11 * 3u) + 2u], 0, metal::memory_order_relaxed);
    metal::float3 _e67 = spring_force;
    spring_force = _e67 * 100.0;
    metal::float3 _e72 = nodeSrc[index_11].force;
    metal::float3 _e73 = spring_force;
    nodeSrc[index_11].force = _e72 + _e73;
    return;
}


struct displacementInput {
};
kernel void displacement(
  metal::uint3 global_invocation_id_11 [[thread_position_in_grid]]
, device type_6& nodeSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    uint total_8 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint index_12 = global_invocation_id_11.x;
    if (index_12 >= total_8) {
        return;
    }
    metal::float3 _e17 = nodeSrc[index_12].force;
    metal::float3 _e20 = nodeSrc[index_12].prev_force;
    metal::float3 d_force = _e17 - _e20;
    float swg = metal::sqrt(metal::dot(d_force, d_force));
    uint _e31 = metal::atomic_load_explicit(&nodeSrc[index_12].mass, metal::memory_order_relaxed);
    float factor_2 = (1.0 / (1.0 + metal::sqrt(1.0 * swg))) / static_cast<float>(_e31);
    metal::float3 force = nodeSrc[index_12].force;
    nodeSrc[index_12].force = metal::float3(0.0);
    nodeSrc[index_12].prev_force = force;
    metal::float3 _e45 = nodeSrc[index_12].position;
    nodeSrc[index_12].position = _e45 + ((force * factor_2) * 0.009999999776482582);
    return;
}


struct randomizeInput {
};
kernel void randomize(
  metal::uint3 global_invocation_id_12 [[thread_position_in_grid]]
, constant Uniforms& uniforms [[user(fake0)]]
, device type_6& nodeSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::float3 vPos_1 = {};
    uint total_9 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint index_13 = global_invocation_id_12.x;
    if (index_13 >= total_9) {
        return;
    }
    metal::float3 _e16 = nodeSrc[index_13].position;
    vPos_1 = _e16;
    uint _e22 = uniforms.frame_num;
    float _e25 = random_xy(index_13, 0u + (3u * _e22));
    vPos_1.x = (_e25 * 2.0) - 1.0;
    uint _e34 = uniforms.frame_num;
    float _e37 = random_xy(index_13, 1u + (3u * _e34));
    vPos_1.y = (_e37 * 2.0) - 1.0;
    uint _e46 = uniforms.frame_num;
    float _e49 = random_xy(index_13, 2u + (3u * _e46));
    vPos_1.z = (_e49 * 2.0) - 1.0;
    metal::float3 _e56 = vPos_1;
    nodeSrc[index_13].position = _e56;
    nodeSrc[index_13].force = metal::float3(0.0);
    nodeSrc[index_13].prev_force = metal::float3(0.0);
    return;
}


struct copyInput {
};
kernel void copy(
  metal::uint3 global_invocation_id_13 [[thread_position_in_grid]]
, device type_6 const& nodeSrc [[user(fake0)]]
, constant _mslBufferSizes& _buffer_sizes [[user(fake0)]]
) {
    metal::float3 vPos_2 = {};
    uint total_10 = 1 + (_buffer_sizes.size1 - 0 - 48) / 48;
    uint index_14 = global_invocation_id_13.x;
    if (index_14 >= total_10) {
        return;
    }
    metal::float3 _e16 = nodeSrc[index_14].position;
    vPos_2 = _e16;
    return;
}
