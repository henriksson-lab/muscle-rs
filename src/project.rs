// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Project an MSA onto a list of sequence indexes (wrapper that forwards to the
/// set-based variant).
#[track_caller]
pub fn multi_sequence_project_l3(ms: &MultiSequence, seq_indexes: &[uint]) -> MultiSequence {
    let mut index_set = std::collections::BTreeSet::new();
    for i in 0..seq_indexes.len() {
        let index = seq_indexes[i] as i32;
        index_set.insert(index);
    }
    multi_sequence_project_l16(ms, &index_set)
}

/// Create a new MSA from a subset of sequences, removing all-gap columns.
#[track_caller]
pub fn multi_sequence_project_l16(
    ms: &MultiSequence,
    seq_indexes: &std::collections::BTreeSet<i32>,
) -> MultiSequence {
    let mut subset_msa = MultiSequence::default();
    let mut old_col_count = uint::MAX;
    for old_seq_index in seq_indexes {
        let old_seq = &ms.seqs[*old_seq_index as usize];
        let label = old_seq.label.clone();
        let l = old_seq.char_vec.len() as uint;
        if old_col_count == uint::MAX {
            old_col_count = l;
        } else {
            assert_eq!(l, old_col_count);
        }

        let new_seq = Sequence {
            label,
            char_vec: Vec::with_capacity(l as usize),
        };
        subset_msa.seqs.push(new_seq);
        subset_msa.owners.push(true);
    }

    let new_seq_count = seq_indexes.len() as uint;
    let mut new_col = vec!['\0'; new_seq_count as usize];
    for old_col_index in 0..old_col_count {
        let mut all_gaps = true;
        let mut new_seq_index = 0;
        for old_seq_index in seq_indexes {
            let old_seq = &ms.seqs[*old_seq_index as usize];
            let c = old_seq.char_vec[old_col_index as usize];
            new_col[new_seq_index as usize] = c;
            new_seq_index += 1;
            if c != '-' {
                all_gaps = false;
            }
        }
        if all_gaps {
            continue;
        }

        for new_seq_index in 0..new_seq_count {
            let c = new_col[new_seq_index as usize];
            subset_msa.seqs[new_seq_index as usize].char_vec.push(c);
        }
    }

    subset_msa
}
