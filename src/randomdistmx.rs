// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Build a symmetric SeqCount x SeqCount matrix of random distances in [0.001, 1.001).
#[track_caller]
pub fn get_random_dist_mx_into(seq_count: uint, dist_mx: &mut Vec<Vec<f32>>) {
    dist_mx.clear();
    dist_mx.resize(seq_count as usize, Vec::new());
    for i in 0..seq_count as usize {
        dist_mx[i].resize(seq_count as usize, 0.0);
        dist_mx[i][i] = 0.0;
        for j in 0..i {
            let d = (randu32() % 100) as f32 / 100.0 + 0.001;
            dist_mx[i][j] = d;
            dist_mx[j][i] = d;
        }
    }
}

/// Build and return a symmetric SeqCount x SeqCount matrix of random distances.
#[track_caller]
pub fn get_random_dist_mx(seq_count: uint) -> Vec<Vec<f32>> {
    let mut dist_mx = Vec::new();
    get_random_dist_mx_into(seq_count, &mut dist_mx);
    dist_mx
}
