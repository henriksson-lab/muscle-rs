// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Drop every column of `msa` whose characters are all gaps.
#[track_caller]
pub fn delete_all_gap_columns(msa: &mut MultiSequence) {
    let seq_count = msa.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(msa);

    let mut keeps = Vec::new();
    let mut keep_count = 0;
    for col_index in 0..col_count {
        let keep = msa_get_gap_count(msa, col_index) < seq_count;
        keeps.push(keep);
        if keep {
            keep_count += 1;
        }
    }
    if keep_count == 0 {
        panic!("MSA is all gaps");
    }

    for seq_index in 0..seq_count {
        let mut new_row = Vec::with_capacity(keep_count as usize);
        for col_index in 0..col_count {
            let c = msa.seqs[seq_index as usize].char_vec[col_index as usize];
            if !keeps[col_index as usize] {
                assert!(c == '-' || c == '.');
                continue;
            }
            new_row.push(c);
        }
        assert_eq!(new_row.len(), keep_count as usize);
        msa.seqs[seq_index as usize].char_vec = new_row;
    }
}

/// Restrict `test` to columns aligned to uppercase positions in matching `ref_msa` sequences.
#[track_caller]
pub fn trim_to_ref(test: &MultiSequence, ref_msa: &MultiSequence) -> MultiSequence {
    let test_seq_count = test.seqs.len() as uint;
    let test_col_count = multi_sequence_get_col_count(test);

    let ref_seq_count = test.seqs.len() as uint;
    let ref_col_count = multi_sequence_get_col_count(ref_msa);

    let (ref_labels, ref_label_to_seq_index) = msa_get_label_to_seq_index(ref_msa);
    let (test_labels, test_label_to_seq_index) = msa_get_label_to_seq_index(test);
    assert_eq!(ref_labels.len(), ref_seq_count as usize);
    assert_eq!(test_labels.len(), test_seq_count as usize);

    let mut labels = Vec::new();
    let mut ref_seq_indexes = Vec::new();
    for test_seq_index in 0..test_seq_count {
        let label = &test_labels[test_seq_index as usize];
        if let Some(ref_seq_index) = ref_label_to_seq_index.get(label).copied() {
            labels.push(label.clone());
            ref_seq_indexes.push(ref_seq_index);
        }
    }

    let trimmed_seq_count = ref_seq_indexes.len() as uint;
    let mut trimmed = MultiSequence::default();
    msa_set_size(&mut trimmed, trimmed_seq_count, test_col_count);
    for trimmed_seq_index in 0..trimmed_seq_count {
        let ref_seq_index = ref_seq_indexes[trimmed_seq_index as usize];
        let label = &labels[trimmed_seq_index as usize];
        trimmed.seqs[trimmed_seq_index as usize].label = label.clone();

        let mut pos_to_upper = Vec::new();
        for ref_col_index in 0..ref_col_count {
            let c = ref_msa.seqs[ref_seq_index as usize].char_vec[ref_col_index as usize];
            if c != '-' && c != '.' {
                pos_to_upper.push(c.is_ascii_uppercase());
            }
        }

        let test_seq_index = test_label_to_seq_index
            .get(label)
            .copied()
            .expect("trim_to_ref test label missing");
        let mut pos = 0usize;
        for test_col_index in 0..test_col_count {
            let mut c = test.seqs[test_seq_index as usize].char_vec[test_col_index as usize];
            if c != '-' && c != '.' {
                let upper = pos_to_upper[pos];
                pos += 1;
                if !upper {
                    c = '-';
                }
            }
            trimmed.seqs[trimmed_seq_index as usize].char_vec[test_col_index as usize] = c;
        }
        assert_eq!(pos, pos_to_upper.len());
    }
    delete_all_gap_columns(&mut trimmed);
    trimmed
}

/// CLI entry: trim `test_file_name` to columns of `ref_file_name` and optionally save to `output_file_name`.
#[track_caller]
pub fn cmd_trimtoref(
    test_file_name: &str,
    ref_file_name: &str,
    output_file_name: &str,
) -> MultiSequence {
    let test = msa_from_fasta_file_l95(test_file_name);
    let ref_msa = msa_from_fasta_file_preserve_case(ref_file_name);
    let trimmed = trim_to_ref(&test, &ref_msa);
    if !output_file_name.is_empty() {
        msa_to_fasta_file_l103(&trimmed, output_file_name);
    }
    trimmed
}
