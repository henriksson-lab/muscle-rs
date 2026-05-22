// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Build a new MSA containing a contiguous range of sequences from the input.
#[track_caller]
pub fn msa_from_seq_range(
    msa_in: &MultiSequence,
    from_seq_index: uint,
    seq_count: uint,
) -> MultiSequence {
    assert!((from_seq_index + seq_count) as usize <= msa_in.seqs.len());
    let col_count = multi_sequence_get_col_count(msa_in);
    let mut msa_out = MultiSequence::default();
    for seq_index in from_seq_index..from_seq_index + seq_count {
        let seq = &msa_in.seqs[seq_index as usize];
        assert_eq!(seq.char_vec.len(), col_count as usize);
        msa_out.seqs.push(sequence_clone(seq));
        msa_out.owners.push(true);
    }
    msa_out
}

/// Build a new MSA containing a contiguous range of columns from the input.
#[track_caller]
pub fn msa_from_col_range(
    msa_in: &MultiSequence,
    from_col_index: uint,
    col_count: uint,
) -> MultiSequence {
    let in_col_count = multi_sequence_get_col_count(msa_in);
    if from_col_index + col_count > in_col_count {
        panic!("MSAFromColRange, out of bounds");
    }
    let mut msa_out = MultiSequence::default();
    for seq in &msa_in.seqs {
        let mut out_seq = Sequence {
            label: seq.label.clone(),
            char_vec: Vec::with_capacity(col_count as usize),
        };
        for col_index in from_col_index..from_col_index + col_count {
            out_seq.char_vec.push(seq.char_vec[col_index as usize]);
        }
        msa_out.seqs.push(out_seq);
        msa_out.owners.push(true);
    }
    msa_out
}

/// Remove every column from `msa` that contains only gap characters.
#[track_caller]
pub fn delete_gapped_cols(msa: &mut MultiSequence) {
    let mut col_index = 0usize;
    while !msa.seqs.is_empty() && col_index < msa.seqs[0].char_vec.len() {
        let is_gap_column = msa
            .seqs
            .iter()
            .all(|seq| matches!(seq.char_vec[col_index], '-' | '.'));
        if is_gap_column {
            for seq in &mut msa.seqs {
                seq.char_vec.remove(col_index);
            }
        } else {
            col_index += 1;
        }
    }
}

/// Build a new MSA containing the rows listed in `seq_indexes`, in that order.
#[track_caller]
pub fn msa_from_seq_subset(msa_in: &MultiSequence, seq_indexes: &[uint]) -> MultiSequence {
    let col_count = multi_sequence_get_col_count(msa_in);
    let mut msa_out = MultiSequence::default();
    for seq_index in seq_indexes {
        let seq = &msa_in.seqs[*seq_index as usize];
        assert_eq!(seq.char_vec.len(), col_count as usize);
        msa_out.seqs.push(sequence_clone(seq));
        msa_out.owners.push(true);
    }
    msa_out
}

/// Assert that two MSAs contain the same labelled sequences modulo case and gaps.
#[track_caller]
pub fn assert_msa_eq_ignore_case_and_gaps(msa1: &MultiSequence, msa2: &MultiSequence) {
    assert_eq!(msa1.seqs.len(), msa2.seqs.len(), "Seq count differs");
    for seq1 in &msa1.seqs {
        let seq2 = msa2
            .seqs
            .iter()
            .find(|seq| seq.label == seq1.label)
            .unwrap_or_else(|| panic!("Seq {} not found", seq1.label));
        let s1 = sequence_get_seq_as_string(seq1)
            .chars()
            .filter(|c| *c != '-' && *c != '.')
            .collect::<String>()
            .to_ascii_uppercase();
        let s2 = sequence_get_seq_as_string(seq2)
            .chars()
            .filter(|c| *c != '-' && *c != '.')
            .collect::<String>()
            .to_ascii_uppercase();
        assert_eq!(s1, s2, "Seq {} differ", seq1.label);
    }
}

/// Assert that two MSAs contain exactly the same labelled aligned sequences.
#[track_caller]
pub fn assert_msa_eq(msa1: &MultiSequence, msa2: &MultiSequence) {
    assert_eq!(msa1.seqs.len(), msa2.seqs.len(), "Seq count differs");
    for seq1 in &msa1.seqs {
        let seq2 = msa2
            .seqs
            .iter()
            .find(|seq| seq.label == seq1.label)
            .unwrap_or_else(|| panic!("Seq {} not found", seq1.label));
        assert_eq!(
            sequence_get_seq_as_string(seq1),
            sequence_get_seq_as_string(seq2),
            "Seq {} differ",
            seq1.label
        );
    }
}

/// Append columns of `msa2` to `msa1` in place, matching sequences by label.
#[track_caller]
pub fn msa_append(msa1: &mut MultiSequence, msa2: &MultiSequence) {
    assert!(multi_sequence_is_aligned(msa1));
    assert!(multi_sequence_is_aligned(msa2));
    let col_count2 = multi_sequence_get_col_count(msa2);
    for seq1 in &mut msa1.seqs {
        let seq2 = msa2
            .seqs
            .iter()
            .find(|seq| seq.label == seq1.label)
            .unwrap_or_else(|| panic!("Seq {} not found", seq1.label));
        assert_eq!(seq2.char_vec.len(), col_count2 as usize);
        seq1.char_vec.extend_from_slice(&seq2.char_vec);
    }
}

/// Concatenate two aligned MSAs column-wise; rows of `msa1` missing in `msa2` get gaps.
#[track_caller]
pub fn msa_cat(msa1: &MultiSequence, msa2: &MultiSequence) -> MultiSequence {
    assert!(multi_sequence_is_aligned(msa1));
    assert!(multi_sequence_is_aligned(msa2));
    let col_count2 = multi_sequence_get_col_count(msa2);
    let mut msa_cat = MultiSequence::default();
    for seq1 in &msa1.seqs {
        let mut seq = sequence_clone(seq1);
        if let Some(seq2) = msa2.seqs.iter().find(|seq2| seq2.label == seq1.label) {
            seq.char_vec.extend_from_slice(&seq2.char_vec);
        } else {
            seq.char_vec
                .extend(std::iter::repeat('-').take(col_count2 as usize));
        }
        msa_cat.seqs.push(seq);
        msa_cat.owners.push(true);
    }
    msa_cat
}
