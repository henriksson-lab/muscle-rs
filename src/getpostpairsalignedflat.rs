// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Computes posterior pair probabilities and aligned-pair scores for each indexed
/// sequence pair across two MSAs; fills `sparse_posts` and returns the mean EA score.
#[track_caller]
pub fn p_prog_get_post_pairs_aligned_flat(
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

        let (sparse_post, ea) = p_prog_calc_post_pair_aligned_flat_result(
            msa1,
            msa2,
            seq_indexes1,
            seq_indexes2,
            seq_count1,
            seq_count2,
            pair_index,
        );
        sparse_posts[pair_index as usize] = sparse_post;
        sum_ea += ea;
    }
    sum_ea / pair_count as f32
}

#[track_caller]
fn p_prog_calc_post_pair_aligned_flat_result(
    msa1: &MultiSequence,
    msa2: &MultiSequence,
    seq_indexes1: &[uint],
    seq_indexes2: &[uint],
    seq_count1: uint,
    seq_count2: uint,
    pair_index: uint,
) -> (MySparseMx, f32) {
    let _min = seq_count1.min(seq_count2);
    let _max = seq_count1.max(seq_count2);

    let seq_index1 = seq_indexes1[pair_index as usize];
    let seq_index2 = seq_indexes2[pair_index as usize];
    assert!(seq_index1 < seq_count1);
    assert!(seq_index2 < seq_count2);
    let label1 = &msa1.seqs[seq_index1 as usize].label;
    let label2 = &msa2.seqs[seq_index2 as usize].label;
    let l1 = get_seq_length_by_global_label(label1);
    let l2 = get_seq_length_by_global_label(label2);

    let post = calc_post(label1, label2);
    let mut dp_rows = alloc_dp_rows(l1, l2);
    let mut tb = alloc_tb(l1, l2);

    let (score, _path) = calc_aln_flat(&post, l1, l2, &mut dp_rows, &mut tb);
    let mut sparse_post = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse_post, &post, l1, l2);

    let ea = score / l1.min(l2) as f32;
    (sparse_post, ea)
}
