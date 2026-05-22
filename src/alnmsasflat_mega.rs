// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Mega variant of flat MSA alignment using the threaded `_mega` posterior-pair routine.
#[track_caller]
pub fn align_ms_as_flat_mega(
    progress_str: &str,
    msa1: &MultiSequence,
    msa2: &MultiSequence,
    target_pair_count: uint,
    path: &mut String,
) -> f32 {
    let seq_count1 = msa1.seqs.len() as uint;
    let seq_count2 = msa2.seqs.len() as uint;
    assert!(seq_count1 > 0);
    assert!(seq_count2 > 0);

    let col_count1 = multi_sequence_get_col_count(msa1);
    let col_count2 = multi_sequence_get_col_count(msa2);
    let (seq_indexes1, seq_indexes2) = get_pairs(seq_count1, seq_count2, target_pair_count);
    assert_eq!(seq_indexes1.len(), seq_indexes2.len());

    let mut sparse_mxs = Vec::new();
    let avg_ea = get_post_pairs_aligned_flat_mega(
        progress_str,
        msa1,
        msa2,
        &seq_indexes1,
        &seq_indexes2,
        &mut sparse_mxs,
    );

    let mut post = alloc_post(col_count1, col_count2);
    calc_posterior_flat3(
        msa1,
        msa2,
        &seq_indexes1,
        &seq_indexes2,
        &sparse_mxs,
        &mut post,
    );
    let mut dp_rows = alloc_dp_rows(col_count1, col_count2);
    let mut tb = alloc_tb(col_count1, col_count2);
    let (_score, aln_path) = calc_aln_flat(&post, col_count1, col_count2, &mut dp_rows, &mut tb);
    *path = aln_path;
    avg_ea
}
