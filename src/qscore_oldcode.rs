// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Original O(NL) PREFAB Q / Balibase TC scoring (annotates each ref column
/// with the matching test column; equal indices mean a correct pair). Returns
/// `(Q, TC, log)`.
#[track_caller]
pub fn cmd_qscore_oldcode(test_file_name: &str, ref_file_name: &str) -> (f64, f64, String) {
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
    let mut ref_to_test_seq_index = vec![uint::MAX; ref_seq_count as usize];
    for ref_seq_index in 0..ref_seq_count {
        let seq_name = msa_ref.seqs[ref_seq_index as usize].label.clone();
        ref_to_test_seq_index[ref_seq_index as usize] = uint::MAX;
        ref_seq_name_to_index.insert(seq_name, ref_seq_index);
    }

    let mut found_count = 0;
    for test_seq_index in 0..test_seq_count {
        let seq_name = &msa_test.seqs[test_seq_index as usize].label;
        if let Some(ref_seq_index) = ref_seq_name_to_index.get(seq_name).copied() {
            assert_ne!(ref_seq_index, uint::MAX);
            ref_to_test_seq_index[ref_seq_index as usize] = test_seq_index;
            found_count += 1;
        }
    }
    if found_count == 0 {
        die("No reference labels found in test MSA");
    }

    let mut test_col_index = vec![0; test_seq_count as usize];
    let mut test_col_index_count = vec![0; test_col_count as usize + 1];
    let mut test_col_indexes = Vec::<uint>::new();
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
            if c_ref == '-' || c_ref == '.' {
                continue;
            }

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
            if c_ref.is_ascii_alphabetic() && (c_ref.is_ascii_uppercase() || c_ref == 'x') {
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
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let log = format!(
        "{test_file_name} Q={}, TC={}\n",
        format_g3(q),
        format_g3(tc)
    );
    (q, tc, log)
}
