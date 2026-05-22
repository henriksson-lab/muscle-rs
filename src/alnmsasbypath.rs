// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Combines two MSAs into `msa2` by inserting gaps according to an `M/D/I` edit path.
#[track_caller]
pub fn align_two_ms_as_given_path(
    msa_a: &MultiSequence,
    msa_b: &MultiSequence,
    path: &str,
    msa2: &mut MultiSequence,
) {
    multi_sequence_clear(msa2);
    let seq_count_a = msa_a.seqs.len();
    let seq_count_b = msa_b.seqs.len();
    for seq_index_a in 0..seq_count_a {
        let input_row = &msa_a.seqs[seq_index_a];
        let aligned_row = sequence_add_gaps_path(input_row, path, 'D');
        msa2.seqs.push(aligned_row);
        msa2.owners.push(true);
    }

    for seq_index_b in 0..seq_count_b {
        let input_row = &msa_b.seqs[seq_index_b];
        let aligned_row = sequence_add_gaps_path(input_row, path, 'I');
        msa2.seqs.push(aligned_row);
        msa2.owners.push(true);
    }
}
