// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub struct QScorer {
    pub name: String,
    pub test: Option<MultiSequence>,
    pub ref_msa: Option<MultiSequence>,
    pub ref_aligned_col_count: uint,
    pub labels: Vec<String>,
    pub ref_seq_indexes: Vec<uint>,
    pub test_seq_indexes: Vec<uint>,
    pub ref_seq_index_to_test_seq_index: Vec<uint>,
    pub ref_cols: Vec<uint>,
    pub ref_ungapped_counts: Vec<uint>,
    pub pos_to_test_col_vec: Vec<Vec<uint>>,
    pub pos_to_ref_col_vec: Vec<Vec<uint>>,
    pub test_col_to_pos_vec: Vec<Vec<uint>>,
    pub ref_col_to_pos_vec: Vec<Vec<uint>>,
    pub ref_col_to_test_col_vec: Vec<Vec<uint>>,
    pub test_col_to_best_ref_col: Vec<uint>,
    pub max_fracts: Vec<f32>,
    pub best_test_cols: Vec<uint>,
    pub total_pairs: uint64,
    pub total_cols: uint64,
    pub correct_pairs: uint64,
    pub correct_cols: uint64,
    pub q: f32,
    pub tc: f32,
    pub ref_labels: Vec<String>,
    pub ref_label_to_seq_index: std::collections::BTreeMap<String, uint>,
    pub ref_seq_to_seq_index: std::collections::BTreeMap<String, uint>,
    pub test_col_to_count: Vec<uint>,
    pub test_col_is_aligned: Vec<bool>,
    pub ref_col_is_aligned: Vec<bool>,
    pub max_gap_fract: f64,
    pub ref_msas_compared_col_count: uint,
    pub ref_msas_q: f64,
    pub ref_msas_test_cols: Vec<uint>,
    pub ref_msas_ref_cols: Vec<uint>,
    pub ref_msas_col_qs: Vec<f64>,
} // original: QScorer (muscle/src/qscorer.h)

impl Default for QScorer {
    /// Default-initialise a QScorer with empty buffers and `max_gap_fract = 1.0`.
    fn default() -> Self {
        Self {
            name: String::new(),
            test: None,
            ref_msa: None,
            ref_aligned_col_count: 0,
            labels: Vec::new(),
            ref_seq_indexes: Vec::new(),
            test_seq_indexes: Vec::new(),
            ref_seq_index_to_test_seq_index: Vec::new(),
            ref_cols: Vec::new(),
            ref_ungapped_counts: Vec::new(),
            pos_to_test_col_vec: Vec::new(),
            pos_to_ref_col_vec: Vec::new(),
            test_col_to_pos_vec: Vec::new(),
            ref_col_to_pos_vec: Vec::new(),
            ref_col_to_test_col_vec: Vec::new(),
            test_col_to_best_ref_col: Vec::new(),
            max_fracts: Vec::new(),
            best_test_cols: Vec::new(),
            total_pairs: 0,
            total_cols: 0,
            correct_pairs: 0,
            correct_cols: 0,
            q: 0.0,
            tc: 0.0,
            ref_labels: Vec::new(),
            ref_label_to_seq_index: std::collections::BTreeMap::new(),
            ref_seq_to_seq_index: std::collections::BTreeMap::new(),
            test_col_to_count: Vec::new(),
            test_col_is_aligned: Vec::new(),
            ref_col_is_aligned: Vec::new(),
            max_gap_fract: 1.0,
            ref_msas_compared_col_count: 0,
            ref_msas_q: 0.0,
            ref_msas_test_cols: Vec::new(),
            ref_msas_ref_cols: Vec::new(),
            ref_msas_col_qs: Vec::new(),
        }
    }
}

/// Reset all QScorer buffers and counters to a fresh state.
#[track_caller]
pub fn q_scorer_clear(qs: &mut QScorer) {
    qs.test = None;
    qs.ref_msa = None;
    qs.ref_aligned_col_count = 0;
    qs.labels.clear();
    qs.ref_seq_indexes.clear();
    qs.test_seq_indexes.clear();
    qs.ref_seq_index_to_test_seq_index.clear();
    qs.ref_cols.clear();
    qs.ref_ungapped_counts.clear();
    qs.pos_to_test_col_vec.clear();
    qs.pos_to_ref_col_vec.clear();
    qs.test_col_to_pos_vec.clear();
    qs.ref_col_to_pos_vec.clear();
    qs.ref_col_to_test_col_vec.clear();
    qs.test_col_to_best_ref_col.clear();
    qs.max_fracts.clear();
    qs.best_test_cols.clear();
    qs.total_pairs = 0;
    qs.total_cols = 0;
    qs.correct_pairs = 0;
    qs.correct_cols = 0;
    qs.q = 0.0;
    qs.tc = 0.0;
    qs.ref_labels.clear();
    qs.ref_label_to_seq_index.clear();
    qs.ref_seq_to_seq_index.clear();
    qs.test_col_to_count.clear();
    qs.test_col_is_aligned.clear();
    qs.ref_col_is_aligned.clear();
    qs.ref_msas_compared_col_count = 0;
    qs.ref_msas_q = 0.0;
    qs.ref_msas_test_cols.clear();
    qs.ref_msas_ref_cols.clear();
    qs.ref_msas_col_qs.clear();
}

/// Index reference sequences by their ungapped sequence (used when matching by
/// sequence content rather than label).
#[track_caller]
pub fn q_scorer_init_ref_labels_bysequence(qs: &mut QScorer) {
    qs.ref_seq_to_seq_index.clear();
    qs.ref_label_to_seq_index.clear();
    qs.ref_labels.clear();
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    for (ref_seq_index, seq) in ref_msa.seqs.iter().enumerate() {
        let label = seq.label.clone();
        qs.ref_labels.push(label);
        let unseq = q_scorer2_strip_gaps(&sequence_get_seq_as_string(seq));
        if qs.ref_seq_to_seq_index.contains_key(&unseq) {
            continue;
        }
        qs.ref_seq_to_seq_index.insert(unseq, ref_seq_index as uint);
    }
}

/// Match test sequences to reference sequences by ungapped sequence content.
#[track_caller]
pub fn q_scorer_init_ref_to_test_bysequence(qs: &mut QScorer) {
    let ref_seq_count = qs.ref_msa.as_ref().expect("QScorer ref not set").seqs.len();
    let test_msa = qs.test.as_ref().expect("QScorer test not set");
    qs.ref_seq_index_to_test_seq_index = vec![uint::MAX; ref_seq_count];
    for (test_seq_index, seq) in test_msa.seqs.iter().enumerate() {
        let label = seq.label.clone();
        let unseq = q_scorer2_strip_gaps(&sequence_get_seq_as_string(seq));
        let Some(ref_seq_index) = qs.ref_seq_to_seq_index.get(&unseq).copied() else {
            continue;
        };
        assert!((ref_seq_index as usize) < ref_seq_count);
        qs.ref_seq_index_to_test_seq_index[ref_seq_index as usize] = test_seq_index as uint;
        qs.labels.push(label);
        qs.ref_seq_indexes.push(ref_seq_index);
        qs.test_seq_indexes.push(test_seq_index as uint);
    }
}

/// Index reference sequences by their label (the default matching mode).
#[track_caller]
pub fn q_scorer_init_ref_labels(qs: &mut QScorer) {
    qs.ref_labels.clear();
    qs.ref_label_to_seq_index.clear();
    qs.ref_seq_to_seq_index.clear();
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    for (ref_seq_index, seq) in ref_msa.seqs.iter().enumerate() {
        let label = seq.label.clone();
        if qs.ref_label_to_seq_index.contains_key(&label) {
            panic!("Dupe ref label >{label}");
        }
        qs.ref_labels.push(label.clone());
        qs.ref_label_to_seq_index
            .insert(label, ref_seq_index as uint);
    }
}

/// Match test sequences to reference sequences by label.
#[track_caller]
pub fn q_scorer_init_ref_to_test(qs: &mut QScorer) {
    let ref_seq_count = qs.ref_msa.as_ref().expect("QScorer ref not set").seqs.len();
    let test_msa = qs.test.as_ref().expect("QScorer test not set");
    qs.ref_seq_index_to_test_seq_index = vec![uint::MAX; ref_seq_count];
    for (test_seq_index, seq) in test_msa.seqs.iter().enumerate() {
        let label = seq.label.clone();
        let Some(ref_seq_index) = qs.ref_label_to_seq_index.get(&label).copied() else {
            continue;
        };
        assert!((ref_seq_index as usize) < ref_seq_count);
        qs.ref_seq_index_to_test_seq_index[ref_seq_index as usize] = test_seq_index as uint;
        qs.labels.push(label);
        qs.ref_seq_indexes.push(ref_seq_index);
        qs.test_seq_indexes.push(test_seq_index as uint);
    }
}

/// Build per-sequence pos/column lookup tables for the `i`-th matched pair and
/// verify that the test/ref ungapped sequences agree.
#[track_caller]
pub fn q_scorer_init_col_pos_vecs1(qs: &mut QScorer, i: uint) {
    let test_msa = qs.test.as_ref().expect("QScorer test not set");
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    let test_col_count = multi_sequence_get_col_count(test_msa);
    let ref_col_count = multi_sequence_get_col_count(ref_msa);

    let ref_seq_index = qs.ref_seq_indexes[i as usize];
    let test_seq_index = qs.test_seq_indexes[i as usize];
    let label = &qs.labels[i as usize];
    let test_label = &test_msa.seqs[test_seq_index as usize].label;
    let ref_label = &ref_msa.seqs[ref_seq_index as usize].label;
    if test_label != ref_label && qs.ref_seq_to_seq_index.is_empty() {
        panic!("QScorer label mismatch >{label}");
    }

    let ref_row = sequence_get_seq_as_string(&ref_msa.seqs[ref_seq_index as usize]);
    let test_row = sequence_get_seq_as_string(&test_msa.seqs[test_seq_index as usize]);
    qs.pos_to_ref_col_vec[i as usize] = msa_get_pos_to_col(ref_msa, ref_seq_index);
    qs.pos_to_test_col_vec[i as usize] = msa_get_pos_to_col(test_msa, test_seq_index);
    let ref_ungapped_length = qs.pos_to_ref_col_vec[i as usize].len();
    let test_ungapped_length = qs.pos_to_test_col_vec[i as usize].len();
    assert_eq!(ref_ungapped_length, test_ungapped_length);

    qs.ref_col_to_pos_vec[i as usize] = msa_get_col_to_pos(ref_msa, ref_seq_index);
    qs.test_col_to_pos_vec[i as usize] = msa_get_col_to_pos(test_msa, test_seq_index);
    assert_eq!(
        qs.ref_col_to_pos_vec[i as usize].len(),
        ref_col_count as usize
    );
    assert_eq!(
        qs.test_col_to_pos_vec[i as usize].len(),
        test_col_count as usize
    );

    let l = qs.pos_to_ref_col_vec[i as usize].len();
    let lt = qs.pos_to_test_col_vec[i as usize].len();
    if l != lt {
        panic!("Seq lengths differ ref={l}, test={lt} >{label}");
    }

    qs.ref_col_to_test_col_vec[i as usize] = vec![uint::MAX; ref_col_count as usize];
    for ref_col in 0..ref_col_count {
        let pos = qs.ref_col_to_pos_vec[i as usize][ref_col as usize];
        if pos == uint::MAX {
            qs.ref_col_to_test_col_vec[i as usize][ref_col as usize] = uint::MAX;
        } else {
            assert!((pos as usize) < qs.pos_to_test_col_vec[i as usize].len());
            let test_col = qs.pos_to_test_col_vec[i as usize][pos as usize];
            assert!(test_col < test_col_count);
            let test_char = test_msa.seqs[test_seq_index as usize].char_vec[test_col as usize];
            let ref_char = ref_msa.seqs[ref_seq_index as usize].char_vec[ref_col as usize];
            assert!(!matches!(test_char, '-' | '.') && !matches!(ref_char, '-' | '.'));
            if test_char.to_ascii_uppercase() != ref_char.to_ascii_uppercase() {
                panic!("Sequences differ pos {pos} test {test_char} ref {ref_char} >{label}");
            }
            qs.ref_col_to_test_col_vec[i as usize][ref_col as usize] = test_col;
        }
    }
}

/// Build pos/column lookup tables for every matched sequence pair.
#[track_caller]
pub fn q_scorer_init_col_pos_vecs(qs: &mut QScorer) {
    let n = qs.ref_seq_indexes.len();
    if n == 0 {
        panic!("No matches to ref found in test MSA {}", qs.name);
    }
    qs.pos_to_test_col_vec.clear();
    qs.pos_to_ref_col_vec.clear();
    qs.test_col_to_pos_vec.clear();
    qs.ref_col_to_pos_vec.clear();
    qs.ref_col_to_test_col_vec.clear();

    qs.pos_to_test_col_vec.resize(n, Vec::new());
    qs.pos_to_ref_col_vec.resize(n, Vec::new());
    qs.test_col_to_pos_vec.resize(n, Vec::new());
    qs.ref_col_to_pos_vec.resize(n, Vec::new());
    qs.ref_col_to_test_col_vec.resize(n, Vec::new());

    for i in 0..n {
        q_scorer_init_col_pos_vecs1(qs, i as uint);
    }
}

/// Collect indexes of upper-case reference columns (subject to `max_gap_fract`).
#[track_caller]
pub fn q_scorer_init_ref_cols(qs: &mut QScorer) {
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    let ref_col_count = multi_sequence_get_col_count(ref_msa);

    qs.ref_cols.clear();
    for ref_col in 0..ref_col_count {
        if msa_col_is_upper(ref_msa, ref_col, qs.max_gap_fract) {
            qs.ref_cols.push(ref_col);
        }
    }
}

/// Count non-gap letters in each aligned reference column.
#[track_caller]
pub fn q_scorer_init_ref_ungapped_counts(qs: &mut QScorer) {
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    qs.ref_aligned_col_count = qs.ref_cols.len() as uint;
    if qs.ref_aligned_col_count == 0 {
        panic!("Qscorer: No upper case columns in ref");
    }

    qs.ref_ungapped_counts.clear();
    let n = qs.ref_seq_indexes.len();
    for k in 0..qs.ref_aligned_col_count {
        let ref_col = qs.ref_cols[k as usize];
        let mut ungapped_count = 0;
        for i in 0..n {
            let ref_seq_index = qs.ref_seq_indexes[i];
            let c = ref_msa.seqs[ref_seq_index as usize].char_vec[ref_col as usize];
            if c != '-' && c != '.' {
                ungapped_count += 1;
            }
        }
        qs.ref_ungapped_counts.push(ungapped_count);
    }
}

/// Score one reference column: tally correct pairs, pick the best matching
/// test column and update running totals.
#[track_caller]
pub fn q_scorer_do_ref_col(qs: &mut QScorer, k: uint) {
    let ref_col = qs.ref_cols[k as usize];
    let ref_seq_count = qs.ref_msa.as_ref().expect("QScorer ref not set").seqs.len() as uint;

    let mut correct_pairs_col = 0_u64;
    let mut test_col_indexes_found = Vec::new();
    let n = qs.ref_seq_indexes.len();
    let mut test_letter_count = 0_u64;
    for i in 0..n {
        let test_col = qs.ref_col_to_test_col_vec[i][ref_col as usize];
        if test_col != uint::MAX {
            test_letter_count += 1;
            assert!((test_col as usize) < qs.test_col_to_count.len());
            if qs.test_col_to_count[test_col as usize] == 0 {
                test_col_indexes_found.push(test_col);
            }
            qs.test_col_to_count[test_col as usize] += 1;
        }
    }

    let mut max_count = 0_u64;
    let mut best_test_col = uint::MAX;
    for test_col in test_col_indexes_found {
        let count = qs.test_col_to_count[test_col as usize];
        assert!(count > 0);
        let count64 = count as u64;
        if count64 > max_count {
            max_count = count64;
            best_test_col = test_col;
        }
        correct_pairs_col += (count64 * (count64 - 1)) / 2;
        qs.test_col_to_count[test_col as usize] = 0;
    }
    qs.correct_pairs += correct_pairs_col;
    if max_count <= test_letter_count / 2 {
        best_test_col = uint::MAX;
    }
    qs.best_test_cols.push(best_test_col);
    let max_fract = (max_count as f32) / (ref_seq_count as f32);
    qs.max_fracts.push(max_fract);

    assert!((k as usize) < qs.ref_ungapped_counts.len());
    let ungapped_count = qs.ref_ungapped_counts[k as usize] as u64;
    let ungapped_pair_count = (ungapped_count * (ungapped_count - 1)) / 2;
    qs.total_pairs += ungapped_pair_count;

    assert!(ungapped_pair_count >= correct_pairs_col);
    if ungapped_pair_count == correct_pairs_col {
        qs.correct_cols += 1;
    }
}

/// Score every aligned reference column (accumulates Q / TC numerators).
#[track_caller]
pub fn q_scorer_do_ref_cols(qs: &mut QScorer) {
    qs.best_test_cols.clear();
    qs.max_fracts.clear();
    qs.test_col_to_count.clear();

    let test_col_count =
        multi_sequence_get_col_count(qs.test.as_ref().expect("QScorer test not set"));
    qs.test_col_to_count.resize(test_col_count as usize, 0);

    qs.correct_pairs = 0;
    qs.correct_cols = 0;
    for k in 0..qs.ref_aligned_col_count {
        q_scorer_do_ref_col(qs, k);
    }
}

/// For each test column, record which reference column it best matches.
#[track_caller]
pub fn q_scorer_set_test_col_to_best_ref_col(qs: &mut QScorer) {
    let test_col_count =
        multi_sequence_get_col_count(qs.test.as_ref().expect("QScorer test not set"));
    qs.test_col_to_best_ref_col = vec![uint::MAX; test_col_count as usize];
    for k in 0..qs.ref_aligned_col_count {
        let ref_col = qs.ref_cols[k as usize];
        let best_test_col = qs.best_test_cols[k as usize];
        if ref_col == uint::MAX || best_test_col == uint::MAX {
            continue;
        }
        assert!((best_test_col as usize) < qs.test_col_to_best_ref_col.len());
        qs.test_col_to_best_ref_col[best_test_col as usize] = ref_col;
    }
}

/// Convenience wrapper around `q_scorer_run_l346` with `by_sequence = false`.
#[track_caller]
pub fn q_scorer_run_l337(
    qs: &mut QScorer,
    name: &str,
    test: &MultiSequence,
    ref_msa: &MultiSequence,
) {
    let _ = q_scorer_run_l346(qs, name, test, ref_msa, false);
}

/// Full QScorer pipeline: initialise lookups, score columns and populate the
/// `q` / `tc` fields. Returns false if no test sequences matched the reference.
#[track_caller]
pub fn q_scorer_run_l346(
    qs: &mut QScorer,
    name: &str,
    test: &MultiSequence,
    ref_msa: &MultiSequence,
    by_sequence: bool,
) -> bool {
    q_scorer_clear(qs);

    qs.name = name.to_string();
    qs.test = Some(test.clone());
    qs.ref_msa = Some(ref_msa.clone());

    if by_sequence {
        q_scorer_init_ref_labels_bysequence(qs);
        q_scorer_init_ref_to_test_bysequence(qs);
    } else {
        q_scorer_init_ref_labels(qs);
        q_scorer_init_ref_to_test(qs);
    }
    if qs.ref_seq_indexes.is_empty() {
        return false;
    }
    q_scorer_init_col_pos_vecs(qs);
    q_scorer_init_ref_cols(qs);
    q_scorer_init_ref_ungapped_counts(qs);
    q_scorer_do_ref_cols(qs);
    q_scorer_set_test_col_to_best_ref_col(qs);

    qs.q = (qs.correct_pairs as f32) / (qs.total_pairs as f32);
    qs.tc = (qs.correct_cols as f32) / (qs.ref_aligned_col_count as f32);
    true
}

/// Increment per-letter counts for the best test match of one ref column.
#[track_caller]
pub fn q_scorer_update_ref_letter_counts_col(
    qs: &QScorer,
    k: uint,
    letter_counts_vec: &mut Vec<Vec<uint>>,
) {
    assert!((k as usize) < qs.ref_cols.len());
    assert!((k as usize) < qs.best_test_cols.len());

    let ref_col = qs.ref_cols[k as usize];
    let best_test_col = qs.best_test_cols[k as usize];

    let n = qs.ref_seq_indexes.len();
    assert_eq!(qs.test_seq_indexes.len(), n);
    for i in 0..n {
        let pos = qs.ref_col_to_pos_vec[i][ref_col as usize];
        if pos == uint::MAX {
            continue;
        }
        let test_col = qs.pos_to_test_col_vec[i][pos as usize];
        if test_col == best_test_col {
            let ref_seq_index = qs.ref_seq_indexes[i];
            letter_counts_vec[ref_seq_index as usize][ref_col as usize] += 1;
        }
    }
}

/// `letter_counts_vec[ref_seq][ref_col]` is the number of times this position
/// appears in the best-match test column; can be incremented across multiple
/// test MSAs that share the same reference MSA.
#[track_caller]
pub fn q_scorer_update_ref_letter_counts(qs: &QScorer, letter_counts_vec: &mut Vec<Vec<uint>>) {
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    let ref_seq_count = ref_msa.seqs.len();
    let ref_col_count = multi_sequence_get_col_count(ref_msa) as usize;

    if letter_counts_vec.is_empty() {
        letter_counts_vec.clear();
        letter_counts_vec.resize(ref_seq_count, Vec::new());
        for counts in letter_counts_vec.iter_mut().take(ref_seq_count) {
            counts.resize(ref_col_count, 0);
        }
    } else {
        assert!(ref_seq_count > 0);
        assert_eq!(letter_counts_vec.len(), ref_seq_count);
        assert_eq!(letter_counts_vec[0].len(), ref_col_count);
    }

    let k_count = qs.ref_cols.len();
    for k in 0..k_count {
        q_scorer_update_ref_letter_counts_col(qs, k as uint, letter_counts_vec);
    }
}

/// Record whether each test column is considered aligned.
#[track_caller]
pub fn q_scorer_set_test_col_is_aligned(qs: &mut QScorer) {
    qs.test_col_is_aligned.clear();
    let test_msa = qs.test.as_ref().expect("QScorer test not set");
    let test_col_count = multi_sequence_get_col_count(test_msa);
    for test_col in 0..test_col_count {
        qs.test_col_is_aligned
            .push(msa_col_is_aligned(test_msa, test_col));
    }
}

/// Record whether each reference column is considered aligned.
#[track_caller]
pub fn q_scorer_set_ref_col_is_aligned(qs: &mut QScorer) {
    qs.ref_col_is_aligned.clear();
    let ref_msa = qs.ref_msa.as_ref().expect("QScorer ref not set");
    let ref_col_count = multi_sequence_get_col_count(ref_msa);
    for ref_col in 0..ref_col_count {
        qs.ref_col_is_aligned
            .push(msa_col_is_aligned(ref_msa, ref_col));
    }
}

/// Reference-vs-reference comparison mode: for each aligned test column find
/// the best-matching reference column and accumulate per-column Q values.
#[track_caller]
pub fn q_scorer_cmp_ref_ms_as(
    qs: &mut QScorer,
    name: &str,
    test: &MultiSequence,
    ref_msa: &MultiSequence,
    by_sequence: bool,
) {
    q_scorer_clear(qs);

    qs.name = name.to_string();
    qs.test = Some(test.clone());
    qs.ref_msa = Some(ref_msa.clone());

    let test_col_count = multi_sequence_get_col_count(test);

    if by_sequence {
        q_scorer_init_ref_labels_bysequence(qs);
        q_scorer_init_ref_to_test_bysequence(qs);
    } else {
        q_scorer_init_ref_labels(qs);
        q_scorer_init_ref_to_test(qs);
    }
    q_scorer_init_col_pos_vecs(qs);
    q_scorer_set_test_col_is_aligned(qs);
    q_scorer_set_ref_col_is_aligned(qs);

    let n = qs.ref_seq_indexes.len();
    assert_eq!(qs.test_seq_indexes.len(), n);
    if n == 0 {
        panic!("No matched sequences/labels {name}");
    }

    let mut ref_col_to_count = std::collections::BTreeMap::<uint, uint>::new();
    qs.ref_msas_compared_col_count = 0;
    let mut sum_col_q = 0.0;
    for test_col in 0..test_col_count {
        if !qs.test_col_is_aligned[test_col as usize] {
            continue;
        }
        let mut m = 0;
        let mut best_n = 0;
        let mut best_ref_col = uint::MAX;
        for i in 0..n {
            let test_pos = qs.test_col_to_pos_vec[i][test_col as usize];
            if test_pos == uint::MAX {
                continue;
            }
            m += 1;
            let ref_col = qs.pos_to_ref_col_vec[i][test_pos as usize];
            if !qs.ref_col_is_aligned[ref_col as usize] {
                continue;
            }
            let n_col = if let Some(old_n) = ref_col_to_count.get(&ref_col).copied() {
                let new_n = old_n + 1;
                ref_col_to_count.insert(ref_col, new_n);
                new_n
            } else {
                ref_col_to_count.insert(ref_col, 1);
                1
            };
            if n_col > best_n {
                best_ref_col = ref_col;
                best_n = n_col;
            }
        }
        if best_ref_col == uint::MAX || m < 2 {
            continue;
        }
        assert!(m > 0);
        qs.ref_msas_compared_col_count += 1;
        let col_q = (best_n as f64) / (m as f64);
        qs.ref_msas_col_qs.push(col_q);
        qs.ref_msas_test_cols.push(test_col);
        qs.ref_msas_ref_cols.push(best_ref_col);
        sum_col_q += col_q;
    }
    qs.ref_msas_q = -1.0;
    if qs.ref_msas_compared_col_count > 0 {
        qs.ref_msas_q = sum_col_q / (qs.ref_msas_compared_col_count as f64);
    }
}
