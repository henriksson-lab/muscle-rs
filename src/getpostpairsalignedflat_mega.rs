// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Mega-profile variant: runs fwd/bwd, builds posteriors, and aligns each indexed pair;
/// returns the mean expected-accuracy score over all pairs.
#[track_caller]
pub fn get_post_pairs_aligned_flat_mega(
    progress_str: &str,
    msa1: &MultiSequence,
    msa2: &MultiSequence,
    seq_indexes1: &[uint],
    seq_indexes2: &[uint],
    sparse_posts: &mut Vec<MySparseMx>,
) -> f32 {
    let progress_str = if progress_str.len() > 20 {
        &progress_str[..20]
    } else {
        progress_str
    };

    let seq_count1 = msa1.seqs.len() as uint;
    let seq_count2 = msa2.seqs.len() as uint;
    let pair_count = seq_indexes1.len() as uint;
    assert_eq!(seq_indexes2.len() as uint, pair_count);
    assert!(sparse_posts.is_empty());

    sparse_posts.resize(pair_count as usize, MySparseMx::default());

    let mut sum_ea = 0.0_f32;
    for pair_index in 0..pair_count {
        let min = seq_count1.min(seq_count2);
        let max = seq_count1.max(seq_count2);
        let progress_msg = format!("{progress_str} [{min} x {max}, {pair_count} pairs]");
        let _ = progress_step(pair_index, pair_count, &progress_msg);

        let seq_index1 = seq_indexes1[pair_index as usize];
        let seq_index2 = seq_indexes2[pair_index as usize];
        assert!(seq_index1 < seq_count1);
        assert!(seq_index2 < seq_count2);

        let gapped_seq1 = &msa1.seqs[seq_index1 as usize];
        let gapped_seq2 = &msa2.seqs[seq_index2 as usize];
        let seq1 = sequence_copy_delete_gaps(&sequence_get_seq_as_string(gapped_seq1));
        let seq2 = sequence_copy_delete_gaps(&sequence_get_seq_as_string(gapped_seq2));
        let byte_seq1 = seq1.into_bytes();
        let byte_seq2 = seq2.into_bytes();
        let l1 = byte_seq1.len() as uint;
        let l2 = byte_seq2.len() as uint;

        let mut fwd = alloc_fb(l1, l2);
        let mut bwd = alloc_fb(l1, l2);
        let mut post = alloc_post(l1, l2);

        calc_fwd_flat_l12(&byte_seq1, l1, &byte_seq2, l2, &mut fwd);
        calc_bwd_flat_l10(&byte_seq1, l1, &byte_seq2, l2, &mut bwd);
        calc_post_flat(&fwd, &bwd, l1, l2, &mut post);

        let mut dp_rows = alloc_dp_rows(l1, l2);
        let mut tb = alloc_tb(l1, l2);
        let (score, _path) = calc_aln_flat(&post, l1, l2, &mut dp_rows, &mut tb);

        my_sparse_mx_from_post(&mut sparse_posts[pair_index as usize], &post, l1, l2);

        let ea = score / l1.min(l2) as f32;
        sum_ea += ea;
    }
    sum_ea / pair_count as f32
}
