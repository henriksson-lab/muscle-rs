// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Builds a posterior probability matrix to align a pair of alignments.
/// Variant of `CalcPostFlat` where sparse posterior matrices cover only pairs
/// with one sequence from MSA1 and the other from MSA2 (not the union).
#[track_caller]
pub fn calc_posterior_flat3(
    msa1: &MultiSequence,
    msa2: &MultiSequence,
    seq_indexes1: &[uint],
    seq_indexes2: &[uint],
    sparse_mxs: &[MySparseMx],
    flat: &mut [f32],
) {
    let _seq_count1 = msa1.seqs.len() as uint;
    let _seq_count2 = msa1.seqs.len() as uint;

    let col_count1 = multi_sequence_get_col_count(msa1);
    let col_count2 = multi_sequence_get_col_count(msa2);

    let flat_size = col_count1 * col_count2;
    assert!(flat.len() >= flat_size as usize);
    for x in flat.iter_mut().take(flat_size as usize) {
        *x = 0.0;
    }

    let pair_count = sparse_mxs.len() as uint;
    assert_eq!(seq_indexes1.len() as uint, pair_count);
    assert_eq!(seq_indexes2.len() as uint, pair_count);
    for pair_index in 0..pair_count {
        let seq_index1 = seq_indexes1[pair_index as usize];
        let seq_index2 = seq_indexes2[pair_index as usize];

        let seq1 = &msa1.seqs[seq_index1 as usize];
        let seq2 = &msa2.seqs[seq_index2 as usize];

        let col_count_seq1 = seq1.char_vec.len() as uint;
        let col_count_seq2 = seq2.char_vec.len() as uint;

        assert_eq!(col_count_seq1, col_count1);
        assert_eq!(col_count_seq2, col_count2);

        let post_mx12 = &sparse_mxs[pair_index as usize];
        let l1 = post_mx12.lx;
        let l2 = post_mx12.ly;

        let pos_to_col1 = sequence_get_pos_to_col(&sequence_get_seq_as_string(seq1));
        let pos_to_col2 = sequence_get_pos_to_col(&sequence_get_seq_as_string(seq2));

        assert_eq!(pos_to_col1.len() as uint, l1);
        assert_eq!(pos_to_col2.len() as uint, l2);

        for pos1 in 0..l1 {
            let offset = my_sparse_mx_get_offset(post_mx12, pos1);
            let row_size = my_sparse_mx_get_size(post_mx12, pos1);
            assert!((pos1 as usize) < pos_to_col1.len());
            let col1 = pos_to_col1[pos1 as usize];
            let flat_base = col1 * col_count2;

            for k in 0..row_size {
                let (prob, pos2) = post_mx12.value_vec[(offset + k) as usize];
                assert!((pos2 as usize) < pos_to_col2.len());
                let col2 = pos_to_col2[pos2 as usize];
                let flat_offset = flat_base + col2;
                assert!(flat_offset < flat_size);
                flat[flat_offset as usize] += prob;
            }
        }
    }
}
