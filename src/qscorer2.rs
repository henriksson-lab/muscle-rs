// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct QScorer2; // original: QScorer2 (muscle/src/qscorer2.h)

/// Return a copy of `row` with all gap characters removed.
pub fn q_scorer2_strip_gaps(row: &str) -> String {
    let mut seq = String::new();
    for c in row.bytes() {
        if c != b'-' && c != b'.' {
            seq.push(char::from(c));
        }
    }
    seq
}

/// Build a vector that maps each ungapped position to its column index.
#[track_caller]
pub fn q_scorer2_get_pos_to_col(row: &str) -> Vec<uint> {
    let mut pos_to_col = Vec::new();
    for (col, c) in row.bytes().enumerate() {
        if c != b'-' && c != b'.' {
            pos_to_col.push(col as uint);
        }
    }
    pos_to_col
}

/// Pairwise Q score: fraction of reference-aligned residue pairs whose two test
/// columns coincide.
#[track_caller]
pub fn q_scorer2_get_q(t1: &str, t2: &str, r1: &str, r2: &str) -> f64 {
    let nt = t1.len();
    assert_eq!(t2.len(), nt);
    let nr = r1.len();
    assert_eq!(r2.len(), nr);

    assert_eq!(q_scorer2_strip_gaps(t1), q_scorer2_strip_gaps(r1));
    assert_eq!(q_scorer2_strip_gaps(t2), q_scorer2_strip_gaps(r2));

    let pos_to_col_t1 = q_scorer2_get_pos_to_col(t1);
    let pos_to_col_t2 = q_scorer2_get_pos_to_col(t2);

    let t1_bytes = t1.as_bytes();
    let t2_bytes = t2.as_bytes();
    let r1_bytes = r1.as_bytes();
    let r2_bytes = r2.as_bytes();
    let mut ref_pos1 = 0usize;
    let mut ref_pos2 = 0usize;
    let mut ref_aligned_count = 0_u32;
    let mut correct_count = 0_u32;
    for ref_col in 0..nr {
        let rc1 = r1_bytes[ref_col];
        let rc2 = r2_bytes[ref_col];
        let gap1 = rc1 == b'-' || rc1 == b'.';
        let gap2 = rc2 == b'-' || rc2 == b'.';
        if !gap1 && !gap2 {
            assert!(ref_pos1 < pos_to_col_t1.len());
            assert!(ref_pos2 < pos_to_col_t2.len());
            let test_col1 = pos_to_col_t1[ref_pos1] as usize;
            let test_col2 = pos_to_col_t2[ref_pos2] as usize;
            let tc1 = t1_bytes[test_col1];
            let tc2 = t2_bytes[test_col2];
            assert_eq!(rc1.to_ascii_uppercase(), tc1.to_ascii_uppercase());
            assert_eq!(rc2.to_ascii_uppercase(), tc2.to_ascii_uppercase());
            ref_aligned_count += 1;
            if test_col1 == test_col2 {
                correct_count += 1;
            }
        }
        if !gap1 {
            ref_pos1 += 1;
        }
        if !gap2 {
            ref_pos2 += 1;
        }
    }
    correct_count as f64 / ref_aligned_count as f64
}

/// Wrapper forwarding to `q_scorer2_run_l100`.
#[track_caller]
pub fn q_scorer2_run_l91(test: &MultiSequence, ref_ms: &MultiSequence) -> f64 {
    q_scorer2_run_l100(test, ref_ms)
}

/// Average Q score over every consecutive reference pair (sequences are paired
/// 0+1, 2+3, ...) matched into the test MSA by label.
#[track_caller]
pub fn q_scorer2_run_l100(test: &MultiSequence, ref_ms: &MultiSequence) -> f64 {
    let mut test_label_to_seq_index = std::collections::BTreeMap::new();
    for (seq_index, seq) in test.seqs.iter().enumerate() {
        test_label_to_seq_index.insert(seq.label.clone(), seq_index as uint);
    }

    let ref_seq_count = ref_ms.seqs.len() as uint;
    assert_eq!(ref_seq_count % 2, 0);
    let ref_pair_count = ref_seq_count / 2;
    let mut sum_q = 0.0_f64;
    for ref_pair_index in 0..ref_pair_count {
        let ref_seq_index1 = ref_pair_index * 2;
        let ref_seq_index2 = ref_seq_index1 + 1;
        let ref_seq1 = &ref_ms.seqs[ref_seq_index1 as usize];
        let ref_seq2 = &ref_ms.seqs[ref_seq_index2 as usize];
        let label1 = &ref_seq1.label;
        let label2 = &ref_seq2.label;
        let col_count = ref_seq1.char_vec.len();
        assert_eq!(ref_seq2.char_vec.len(), col_count);

        let test_seq_index1 = *test_label_to_seq_index
            .get(label1)
            .unwrap_or_else(|| panic!("test label not found: {label1}"));
        let test_seq_index2 = *test_label_to_seq_index
            .get(label2)
            .unwrap_or_else(|| panic!("test label not found: {label2}"));

        let test_row1 = sequence_get_seq_as_string(&test.seqs[test_seq_index1 as usize]);
        let test_row2 = sequence_get_seq_as_string(&test.seqs[test_seq_index2 as usize]);
        let ref_row1 = sequence_get_seq_as_string(ref_seq1);
        let ref_row2 = sequence_get_seq_as_string(ref_seq2);
        let q = q_scorer2_get_q(&test_row1, &test_row2, &ref_row1, &ref_row2);
        sum_q += q;
    }
    sum_q / ref_pair_count as f64
}
