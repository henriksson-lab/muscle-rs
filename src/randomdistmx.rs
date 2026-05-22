// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Build a symmetric SeqCount x SeqCount matrix of random distances in [0.001, 1.001).
#[track_caller]
pub fn get_random_dist_mx(seq_count: uint) -> Vec<Vec<f32>> {
    let mut dist_mx = vec![vec![0.0_f32; seq_count as usize]; seq_count as usize];
    for i in 0..seq_count as usize {
        dist_mx[i][i] = 0.0;
        for j in 0..i {
            let d = (randu32() % 100) as f32 / 100.0 + 0.001;
            dist_mx[i][j] = d;
            dist_mx[j][i] = d;
        }
    }
    dist_mx
}
