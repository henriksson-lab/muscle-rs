// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Convenience wrapper: expands a sparse posterior into a dense matrix and
/// returns the maximum-expected-accuracy alignment score.
#[track_caller]
pub fn calc_aln_score_sparse(mx: &MySparseMx) -> f32 {
    let lx = mx.lx;
    let ly = mx.ly;
    let post = my_sparse_mx_to_post(mx);
    let mut dp_rows = alloc_dp_rows(lx, ly);
    calc_aln_score_flat(&post, lx, ly, &mut dp_rows)
}
