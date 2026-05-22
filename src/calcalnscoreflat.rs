// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Maximum-expected-accuracy DP score from a dense posterior matrix, without
/// keeping the traceback. Uses two rolling DP rows in `dp_rows`.
#[track_caller]
pub fn calc_aln_score_flat(post: &[f32], lx: uint, ly: uint, dp_rows: &mut [f32]) -> f32 {
    let row_len = (ly + 1) as usize;
    assert!(dp_rows.len() >= row_len);
    assert!(post.len() >= (lx * ly) as usize);

    for j in 0..=ly as usize {
        dp_rows[j] = 0.0;
    }

    let mut post_ix = 0usize;
    for _i in 1..=lx as usize {
        let mut curr_j1 = 0.0_f32;
        let mut prev_j1 = dp_rows[0];
        dp_rows[0] = 0.0;
        for j in 1..=ly as usize {
            let prev_j = dp_rows[j];
            let p = post[post_ix];
            post_ix += 1;
            let b_score = prev_j1 + p;
            let x_score = prev_j;
            let y_score = curr_j1;

            prev_j1 = dp_rows[j];
            curr_j1 = b_score.max(x_score).max(y_score);
            dp_rows[j] = curr_j1;
        }
    }
    dp_rows[ly as usize]
}
