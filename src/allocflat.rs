// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[inline(always)]
fn uninit_vec<T>(len: usize) -> Vec<T> {
    let mut v = Vec::with_capacity(len);
    // SAFETY: This mirrors the original C++ `myalloc` scratch buffers, which
    // return uninitialized storage. The flat DP/posterior callers overwrite
    // their live ranges before reading them; these allocators must not be used
    // for buffers whose initial value is semantically observed.
    unsafe {
        v.set_len(len);
    }
    v
}

/// Element count for the forward/backward matrix: `(LX+1)*(LY+1)*HMMSTATE_COUNT`.
#[track_caller]
pub fn get_fb_size(lx: uint, ly: uint) -> uint64 {
    uint64::from(lx + 1) * uint64::from(ly + 1) * HMMSTATE_COUNT
}

/// Element count for a posterior matrix: `LX*LY`.
#[track_caller]
pub fn get_post_size(lx: uint, ly: uint) -> uint64 {
    let size64 = uint64::from(lx) * uint64::from(ly);
    assert_eq!(uint64::from(size64 as uint), size64);
    size64
}

/// Element count for the rolling DP rows: `2*(LY+1)`.
#[track_caller]
pub fn get_dp_rows_size(_lx: uint, ly: uint) -> uint64 {
    let size64 = 2 * uint64::from(ly + 1);
    assert_eq!(uint64::from(size64 as uint), size64);
    size64
}

/// Element count for the traceback matrix: `(LX+1)*(LY+1)`.
#[track_caller]
pub fn get_tb_size(lx: uint, ly: uint) -> uint64 {
    let size64 = uint64::from(lx + 1) * uint64::from(ly + 1);
    assert_eq!(uint64::from(size64 as uint), size64);
    size64
}

/// Allocates a zeroed forward/backward matrix; dies if `LX*LY` would overflow HMM buffers.
#[track_caller]
pub fn alloc_fb(lx: uint, ly: uint) -> Vec<f32> {
    if f64::from(lx) * f64::from(ly) * 5.0 + 100.0 > f64::from(i32::MAX) {
        panic!("Sequences length {lx}, {ly} overflow HMM buffers");
    }
    uninit_vec(get_fb_size(lx, ly) as usize)
}

/// Allocates a posterior matrix of size `LX*LY`.
#[track_caller]
pub fn alloc_post(lx: uint, ly: uint) -> Vec<f32> {
    uninit_vec(get_post_size(lx, ly) as usize)
}

/// Allocates the two rolling DP rows used by the Viterbi traceback.
#[track_caller]
pub fn alloc_dp_rows(lx: uint, ly: uint) -> Vec<f32> {
    uninit_vec(get_dp_rows_size(lx, ly) as usize)
}

/// Allocates the traceback matrix used by the pair-HMM Viterbi.
#[track_caller]
pub fn alloc_tb(lx: uint, ly: uint) -> Vec<i8> {
    uninit_vec(get_tb_size(lx, ly) as usize)
}
