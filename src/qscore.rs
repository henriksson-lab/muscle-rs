// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// O(NL) computation of PREFAB Q score and Balibase TC score. Each reference
/// column is annotated with the test column where the same letter appears; a
/// pair of identical column indices in the same ref column means a correctly
/// aligned pair. Returns `(Q, TC)`.
#[track_caller]
pub fn cmd_qscore(test_file_name: &str, ref_file_name: &str, by_sequence: bool) -> (f64, f64) {
    let mut msa_test = MultiSequence::default();
    let mut msa_ref = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa_test, test_file_name, false);
    multi_sequence_load_mfa_l8(&mut msa_ref, ref_file_name, false);
    assert!(multi_sequence_is_aligned(&msa_test));
    assert!(multi_sequence_is_aligned(&msa_ref));

    let ref_seq_count = msa_ref.seqs.len() as uint;
    let test_seq_count = msa_test.seqs.len() as uint;
    let ref_col_count = multi_sequence_get_col_count(&msa_ref);
    let test_col_count = multi_sequence_get_col_count(&msa_test);

    let mut ref_seq_name_to_index = std::collections::BTreeMap::new();
    let mut ref_seq_to_index = std::collections::BTreeMap::new();
    let mut ref_to_test_seq_index = vec![uint::MAX; ref_seq_count as usize];
    for ref_seq_index in 0..ref_seq_count {
        let ref_seq = &msa_ref.seqs[ref_seq_index as usize];
        let seq_name = ref_seq.label.clone();
        let mut useq = String::new();
        for c in &ref_seq.char_vec {
            if *c != '-' && *c != '.' {
                useq.push(*c);
            }
        }
        ref_seq_name_to_index.insert(seq_name, ref_seq_index);
        ref_seq_to_index.insert(useq, ref_seq_index);
    }

    let mut found_count = 0;
    for test_seq_index in 0..test_seq_count {
        let test_seq = &msa_test.seqs[test_seq_index as usize];
        if by_sequence {
            let mut useq = String::new();
            for c in &test_seq.char_vec {
                if *c != '-' && *c != '.' {
                    useq.push(*c);
                }
            }
            if let Some(ref_seq_index) = ref_seq_to_index.get(&useq) {
                assert_ne!(*ref_seq_index, uint::MAX);
                ref_to_test_seq_index[*ref_seq_index as usize] = test_seq_index;
                found_count += 1;
            }
        } else if let Some(ref_seq_index) = ref_seq_name_to_index.get(&test_seq.label) {
            assert_ne!(*ref_seq_index, uint::MAX);
            ref_to_test_seq_index[*ref_seq_index as usize] = test_seq_index;
            found_count += 1;
        }
    }
    if found_count < 2 {
        if by_sequence {
            panic!("{found_count} ref seqs found in test MSA");
        } else {
            panic!("{found_count} ref labels found in test MSA");
        }
    }

    let mut test_col_index = vec![0; test_seq_count as usize];
    let mut test_col_index_count = vec![0; test_col_count as usize + 1];
    let mut test_col_indexes = Vec::new();

    let mut ref_aligned_col_count = 0;
    let mut correct_col_count = 0;
    let mut seq_diff_count = 0;
    let mut correct_pair_count: uint64 = 0;
    let mut ref_aligned_pair_count: uint64 = 0;

    for ref_col_index in 0..ref_col_count {
        test_col_indexes.clear();
        test_col_indexes.reserve(ref_seq_count as usize);

        let mut non_gapped_count = 0;
        let mut first_test_col_index = uint::MAX;
        let mut ref_col_is_aligned = false;
        let mut test_col_all_correct = true;
        let mut test_all_aligned = true;
        for ref_seq_index in 0..ref_seq_count {
            let test_seq_index = ref_to_test_seq_index[ref_seq_index as usize];
            if test_seq_index == uint::MAX {
                continue;
            }

            let c_ref = msa_ref.seqs[ref_seq_index as usize].char_vec[ref_col_index as usize];
            if c_ref != '-' && c_ref != '.' {
                let mut col = test_col_index[test_seq_index as usize];
                let mut c_test;
                loop {
                    c_test = msa_test.seqs[test_seq_index as usize].char_vec[col as usize];
                    col += 1;
                    if c_test != '-' && c_test != '.' {
                        break;
                    }
                }
                if !c_ref.eq_ignore_ascii_case(&c_test) {
                    seq_diff_count += 1;
                }
                if c_ref.is_ascii_alphabetic() && c_ref.is_ascii_uppercase() {
                    ref_col_is_aligned = true;
                    non_gapped_count += 1;
                    if c_test.is_ascii_uppercase() {
                        test_col_indexes.push(col);
                        test_col_index_count[col as usize] += 1;
                        if first_test_col_index == uint::MAX {
                            first_test_col_index = col;
                        } else if first_test_col_index != col {
                            test_col_all_correct = false;
                        }
                    } else {
                        test_all_aligned = false;
                    }
                } else if ref_col_is_aligned {
                    panic!("Ref col {ref_col_index} has both upper- and lower-case letters");
                }
                test_col_index[test_seq_index as usize] = col;
            }
        }

        if ref_col_is_aligned && non_gapped_count > 1 {
            ref_aligned_col_count += 1;
            if test_col_all_correct && test_all_aligned {
                correct_col_count += 1;
            }
        }

        let mut col_pair_count = 0;
        for col in &test_col_indexes {
            let count = test_col_index_count[*col as usize];
            if count > 0 {
                col_pair_count += count * (count - 1) / 2;
            }
            test_col_index_count[*col as usize] = 0;
        }

        correct_pair_count += col_pair_count as uint64;
        ref_aligned_pair_count += (non_gapped_count * (non_gapped_count - 1) / 2) as uint64;
    }

    let q = if ref_aligned_pair_count == 0 {
        0.0
    } else {
        correct_pair_count as f64 / ref_aligned_pair_count as f64
    };

    let tc = if ref_aligned_col_count == 0 {
        0.0
    } else {
        correct_col_count as f64 / ref_aligned_col_count as f64
    };
    let _ = seq_diff_count;
    (q, tc)
}
