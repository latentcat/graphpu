// language: metal2.0
#include <metal_stdlib>
#include <simd/simd.h>

using metal::uint;

struct _mslBufferSizes {
    uint size0;
};

typedef metal::atomic_int type_1[1];

void atomic_add_f32_(
    uint springIndex,
    float updateValue,
    device type_1& arr,
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
        int _e10 = assumed;
        int _e11 = new_u32_;
        metal::atomic_compare_exchange_weak_explicit(&arr[springIndex], &_e10, _e11, metal::memory_order_relaxed, metal::memory_order_relaxed);
        origin = _e10;
        int _e13 = origin;
        int _e14 = assumed;
        if (_e13 == _e14) {
            break;
        }
        int _e16 = origin;
        assumed = _e16;
        int _e17 = origin;
        new_u32_ = as_type<int>(as_type<float>(_e17) + updateValue);
    }
    return;
}

void cas(
    device type_1& arr,
    constant _mslBufferSizes& _buffer_sizes
) {
    int v = -1;
    metal::atomic_compare_exchange_weak_explicit(&arr[0], &-1, 1, metal::memory_order_relaxed, metal::memory_order_relaxed);
    int _e10 = v;
    metal::atomic_compare_exchange_weak_explicit(&arr[0], &_e10, 1, metal::memory_order_relaxed, metal::memory_order_relaxed);
    return;
}
