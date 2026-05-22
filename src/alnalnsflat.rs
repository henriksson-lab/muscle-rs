// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Aligns two MSAs using the MPCFlat posterior and Viterbi traceback; returns the merged MSA,
/// score and edit path.
#[track_caller]
pub fn mpc_flat_align_alns(
    mpc: &mut MPCFlat,
    msa1: &MultiSequence,
    msa2: &MultiSequence,
) -> (MultiSequence, f32, String) {
    let seq_count1 = msa1.seqs.len() as uint;
    let seq_count2 = msa2.seqs.len() as uint;
    let col_count1 = multi_sequence_get_col_count(msa1);
    let col_count2 = multi_sequence_get_col_count(msa2);

    let seq_count = mpc_flat_get_seq_count(mpc);
    if mpc.weights.len() != seq_count as usize {
        mpc.weights = vec![1.0; seq_count as usize];
    }

    let post = mpc_flat_build_post(mpc, msa1, msa2);
    let mut dp_rows = alloc_dp_rows(col_count1, col_count2);
    let mut tb = alloc_tb(col_count1, col_count2);
    let (score, path) = calc_aln_flat(&post, col_count1, col_count2, &mut dp_rows, &mut tb);

    let mut result = MultiSequence::default();
    for seq_index1 in 0..seq_count1 {
        let input_row = &msa1.seqs[seq_index1 as usize];
        let aligned_row = sequence_add_gaps_path(input_row, &path, 'X');
        result.seqs.push(aligned_row);
        result.owners.push(true);
    }
    for seq_index2 in 0..seq_count2 {
        let input_row = &msa2.seqs[seq_index2 as usize];
        let aligned_row = sequence_add_gaps_path(input_row, &path, 'Y');
        result.seqs.push(aligned_row);
        result.owners.push(true);
    }
    (result, score, path)
}
