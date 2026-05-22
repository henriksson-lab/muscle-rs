// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Posterior-based pair alignment for two globally-labelled sequences; optionally returns the
/// sparse posterior. Returns `(expected_accuracy, path)`.
#[track_caller]
pub fn align_pair_flat_sparse_post(
    label1: &str,
    label2: &str,
    sparse_post: Option<&mut MySparseMx>,
) -> (f32, String) {
    let l1 = get_seq_length_by_global_label(label1);
    let l2 = get_seq_length_by_global_label(label2);
    let post = calc_post(label1, label2);
    let mut dp_rows = alloc_dp_rows(l1, l2);
    let mut tb = alloc_tb(l1, l2);
    let (score, path) = calc_aln_flat(&post, l1, l2, &mut dp_rows, &mut tb);
    if let Some(sparse) = sparse_post {
        my_sparse_mx_from_post(sparse, &post, l1, l2);
    }

    assert!(l1 > 0 && l2 > 0);
    let ea = score / l1.min(l2) as f32;
    (ea, path)
}

/// Aligns a pair of globally-labelled sequences; returns `(expected_accuracy, path)`.
#[track_caller]
pub fn align_pair_flat(label1: &str, label2: &str) -> (f32, String) {
    align_pair_flat_sparse_post(label1, label2, None)
}
