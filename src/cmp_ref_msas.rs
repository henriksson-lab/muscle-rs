// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct CmpMsaColorState {
    pub(crate) h: f64,
    pub(crate) colors: Vec<uint>,
    pub(crate) prev_random_color: uint,
}

pub(crate) static CMP_MSA_COLOR_STATE: std::sync::Mutex<CmpMsaColorState> =
    std::sync::Mutex::new(CmpMsaColorState {
        h: f64::MAX,
        colors: Vec::new(),
        prev_random_color: 0,
    });

/// Map a Q-score in [0,1] to a single display character: `|`, `:`, `.`, `@`, or `*`.
pub fn get_q_char(q: f64) -> char {
    assert!((0.0..=1.0).contains(&q));
    if q == 1.0 {
        '|'
    } else if q >= 0.9 {
        ':'
    } else if q >= 0.5 {
        '.'
    } else if q > 0.0 {
        '@'
    } else {
        '*'
    }
}

/// Compare a test MSA against a reference MSA: compute Q-scores, render a column-aligned report.
#[track_caller]
pub fn cmd_cmp_ref_msas(
    test_file_name: &str,
    ref_file_name: &str,
    max_gap_fract: f64,
) -> (QScorer, String) {
    let name = get_base_name(test_file_name);

    let test = msa_from_fasta_file_preserve_case(test_file_name);
    let ref_msa = msa_from_fasta_file_preserve_case(ref_file_name);

    let mut qs = QScorer {
        max_gap_fract,
        ..QScorer::default()
    };
    q_scorer_cmp_ref_ms_as(&mut qs, &name, &test, &ref_msa, false);

    let nc = qs.ref_msas_compared_col_count;
    let mut out = format!(
        "@CMP_REF_MSAs test={test_file_name} ref={ref_file_name} name={name} cols={nc} Q={:.4}\n",
        qs.ref_msas_q
    );
    if nc == 0 {
        return (qs, out);
    }
    let qs_col_qs = qs.ref_msas_col_qs.clone();
    let test_cols = qs.ref_msas_test_cols.clone();
    let ref_cols = qs.ref_msas_ref_cols.clone();
    assert_eq!(test_cols.len(), nc as usize);
    assert_eq!(ref_cols.len(), nc as usize);
    assert_eq!(qs_col_qs.len(), nc as usize);
    let test_seq_indexes = qs.test_seq_indexes.clone();
    let ref_seq_indexes = qs.ref_seq_indexes.clone();
    let ns = test_seq_indexes.len();
    assert_eq!(ref_seq_indexes.len(), ns);
    for i in 0..nc as usize {
        let test_col = test_cols[i];
        let ref_col = ref_cols[i];
        let mut test_col_str = String::new();
        let mut ref_col_str = String::new();
        for j in 0..ns {
            test_col_str.push(msa_get_char(&test, test_seq_indexes[j], test_col));
            ref_col_str.push(msa_get_char(&ref_msa, ref_seq_indexes[j], ref_col));
        }
        out.push_str(&format!(
            "{test_col_str}  {ref_col_str}  {:6.4}\n",
            qs_col_qs[i]
        ));
    }

    out.push('\n');
    let anc = test_cols.len();
    assert!(anc > 0);
    let mut fixed_test_cols = vec![test_cols[0]];
    let mut fixed_ref_cols = vec![ref_cols[0]];
    let mut fixed_qs = vec![qs_col_qs[0]];
    for i in 1..anc {
        let col_x = test_cols[i];
        let col_y = ref_cols[i];
        if col_x > *fixed_test_cols.last().unwrap() && col_y > *fixed_ref_cols.last().unwrap() {
            fixed_test_cols.push(col_x);
            fixed_ref_cols.push(col_y);
            fixed_qs.push(qs_col_qs[i]);
        }
    }
    let nf = fixed_test_cols.len();
    assert_eq!(fixed_ref_cols.len(), nf);
    assert!(nf > 0);

    let test_subset = msa_from_seq_subset(&test, &test_seq_indexes);
    let ref_subset = msa_from_seq_subset(&ref_msa, &ref_seq_indexes);
    let mut test2 = MultiSequence::default();
    let mut ref2 = MultiSequence::default();
    let mut path = String::new();
    let mut merge_map = Vec::new();
    align_ms_as_by_cols(
        &test_subset,
        &ref_subset,
        &fixed_test_cols,
        &fixed_ref_cols,
        &mut path,
        &mut merge_map,
        &mut test2,
        &mut ref2,
    );
    let col_count2 = path.len() as uint;
    assert_eq!(multi_sequence_get_col_count(&test2), col_count2);
    assert_eq!(multi_sequence_get_col_count(&ref2), col_count2);
    assert_eq!(merge_map.len(), nf);

    let mut all_gaps = Vec::new();
    for i in 0..col_count2 {
        all_gaps.push(msa_is_gap_column(&test2, i) && msa_is_gap_column(&ref2, i));
    }

    let mut first_m = uint::MAX;
    let mut last_m = uint::MAX;
    for (i, c) in path.bytes().enumerate() {
        if c == b'M' {
            if first_m == uint::MAX {
                first_m = i as uint;
            }
            last_m = i as uint;
        }
    }

    let mut annot = vec!['_'; col_count2 as usize];
    for i in 0..nf {
        let k = merge_map[i] as usize;
        let q = fixed_qs[i];
        assert!(k < annot.len());
        annot[k] = if q == 1.0 {
            '|'
        } else if q >= 0.9 {
            ':'
        } else if q >= 0.5 {
            '.'
        } else if q > 0.0 {
            '@'
        } else {
            '*'
        };
    }

    for i in 0..test2.seqs.len() as uint {
        let label = msa_get_seq_label(&test2, i);
        let ref_label = msa_get_seq_label(&ref2, i);
        let row = msa_get_row_str(&test2, i);
        let row_bytes = row.as_bytes();
        let mut row_m = String::new();
        for j in first_m..=last_m {
            if !all_gaps[j as usize] {
                row_m.push(row_bytes[j as usize] as char);
            }
        }
        out.push_str(&format!("{row_m}  >{label} (={ref_label})\n"));
    }
    out.push('\n');

    let mut annot_m = String::new();
    for j in first_m..=last_m {
        if !all_gaps[j as usize] {
            annot_m.push(annot[j as usize]);
        }
    }
    out.push_str(&format!("{annot_m}\n\n"));

    for i in 0..ref2.seqs.len() as uint {
        let label = msa_get_seq_label(&ref2, i);
        let row = msa_get_row_str(&ref2, i);
        let row_bytes = row.as_bytes();
        let mut row_m = String::new();
        for j in first_m..=last_m {
            if !all_gaps[j as usize] {
                row_m.push(row_bytes[j as usize] as char);
            }
        }
        out.push_str(&format!("{row_m}  >{label}\n"));
    }
    (qs, out)
}
