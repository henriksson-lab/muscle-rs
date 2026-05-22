// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub(crate) static ASSERT_SAME_SEQS_OK_COUNT: std::sync::Mutex<uint> = std::sync::Mutex::new(0);

/// Panics if any sequence label in `msa1` has a different ungapped sequence in `msa2`.
#[track_caller]
pub fn assert_seqs_eq(file_name: &str, line_nr: uint, msa1: &MultiSequence, msa2: &MultiSequence) {
    let seq_count1 = msa1.seqs.len();
    for seq_index1 in 0..seq_count1 {
        let seq1 = &msa1.seqs[seq_index1];
        let label = &seq1.label;
        let seq_index2 = multi_sequence_get_seq_index(msa2, label, true);
        let seq2 = &msa2.seqs[seq_index2 as usize];

        let mut v1 = Vec::new();
        for &c in &seq1.char_vec {
            if c != '-' && c != '.' {
                v1.push(c);
            }
        }
        let mut v2 = Vec::new();
        for &c in &seq2.char_vec {
            if c != '-' && c != '.' {
                v2.push(c);
            }
        }
        if v1 != v2 {
            panic!("AssertSeqsEq {file_name}:{line_nr}");
        }
    }
}

/// Panics if `ms` contains any sequence whose ungapped letters differ from the original input MSA.
#[track_caller]
pub fn assert_seqs_eq_input(file: &str, line: uint, ms: &MultiSequence) {
    let gn = get_global_ms_seq_count();

    let seq_count = ms.seqs.len();

    let mut gsis = std::collections::BTreeSet::<uint>::new();
    for i in 0..seq_count {
        let seq = &ms.seqs[i];
        let input_seq = get_global_input_seq_by_label(&seq.label);
        let label = seq.label.clone();
        let global_label = input_seq.label.clone();
        let gsi = get_gsi_by_label(&label);
        if global_label != label {
            panic!(
                "{file}:{line} AssertSeqsEqInput Seq({i}) GSI {gsi} label '{label}' != '{global_label}'"
            );
        }

        if gsi >= gn {
            panic!("{file}:{line} AssertSeqsEqInput GSI={gsi} > GN={gn}");
        }
        gsis.insert(gsi);

        let ungapped_input_seq: Vec<char> = input_seq
            .char_vec
            .iter()
            .copied()
            .filter(|&c| c != '-' && c != '.')
            .collect();
        let l = ungapped_input_seq.len() as uint;
        let ungapped_seq: Vec<char> = seq
            .char_vec
            .iter()
            .copied()
            .filter(|&c| c != '-' && c != '.')
            .collect();
        let msl = ungapped_seq.len() as uint;
        if l != msl {
            panic!(
                "{file}:{line} AssertSeqsEqInput Seq({i}) GSI={gsi} L={l}, MSL={msl}, label={label}"
            );
        }

        for pos in 0..l as usize {
            let input_char = ungapped_input_seq[pos];
            let char_ = ungapped_seq[pos];
            if input_char.to_ascii_uppercase() != char_.to_ascii_uppercase() {
                panic!(
                    "{file}:{line} AssertSeqsEqInput Seq({i}) GSI={gsi} Pos[{pos}]={char_},{input_char} label={label}"
                );
            }
        }
    }
}

/// Asserts that `ms` contains the same sequences as the union of MSAs in `v` (by reference slice).
#[track_caller]
pub fn assert_same_seqs_vec_l91(file: &str, line: uint, ms: &MultiSequence, v: &[&MultiSequence]) {
    let mut combined_ms = MultiSequence::default();
    for ms1 in v {
        let n = ms1.seqs.len();
        for j in 0..n {
            let seq = &ms1.seqs[j];
            combined_ms.seqs.push(seq.clone());
            combined_ms.owners.push(false);
        }
    }
    assert_same_seqs(file, line, ms, &combined_ms);
    *ASSERT_SAME_SEQS_OK_COUNT.lock().unwrap() += 1;
}

/// Asserts that `ms` contains the same sequences as the union of MSAs in `v` (owned slice variant).
#[track_caller]
pub fn assert_same_seqs_vec_l111(file: &str, line: uint, ms: &MultiSequence, v: &[MultiSequence]) {
    let mut combined_ms = MultiSequence::default();
    for ms1 in v {
        let n = ms1.seqs.len();
        for j in 0..n {
            let seq = &ms1.seqs[j];
            combined_ms.seqs.push(seq.clone());
            combined_ms.owners.push(false);
        }
    }
    assert_same_seqs(file, line, ms, &combined_ms);
    *ASSERT_SAME_SEQS_OK_COUNT.lock().unwrap() += 1;
}

/// Asserts that the join of `ms1` and `ms2` is identical (label-wise) to `ms12`.
#[track_caller]
pub fn assert_same_seqs_join(
    file: &str,
    line: uint,
    ms1: &MultiSequence,
    ms2: &MultiSequence,
    ms12: &MultiSequence,
) {
    let v = [ms1, ms2];
    assert_same_seqs_vec_l91(file, line, ms12, &v);
}

/// Returns the count of successful `assert_same_seqs` invocations so far.
#[track_caller]
pub fn get_assert_same_seqs_ok_count() -> uint {
    *ASSERT_SAME_SEQS_OK_COUNT.lock().unwrap()
}

/// Panics if any label in `ms` is unknown, duplicated, or mismatches the global input MSA.
#[track_caller]
pub fn assert_same_labels(file: &str, line: uint, ms: &MultiSequence) {
    let global_ms = get_global_input_ms();
    let gn = get_global_ms_seq_count();
    let seq_count = ms.seqs.len();

    let mut gsis = std::collections::BTreeSet::<uint>::new();
    for i in 0..seq_count {
        let seq = &ms.seqs[i];
        let gsi = get_gsi_by_label(&seq.label);
        if gsi >= gn {
            panic!("{file}:{line} AssertSameLabels GSI1={gsi} > GN={gn}");
        }
        if gsis.contains(&gsi) {
            panic!("{file}:{line} AssertSameLabels dupe GSI={gsi}");
        }

        let label = ms.seqs[i].label.clone();
        let global_label = global_ms.seqs[gsi as usize].label.clone();
        if global_label != label {
            panic!(
                "{file}:{line} AssertSameLabels Seq({i}) GSI {gsi} label '{label}' != '{global_label}'"
            );
        }

        gsis.insert(gsi);
    }
}

/// Asserts that two MSAs cover the same set of input sequences by global label (GSI).
#[track_caller]
pub fn assert_same_seqs(file: &str, line: uint, ms1: &MultiSequence, ms2: &MultiSequence) {
    let global_ms = get_global_input_ms();
    let gn = get_global_ms_seq_count();

    let seq_count = ms1.seqs.len();
    let seq_count2 = ms2.seqs.len();
    if seq_count2 != seq_count {
        panic!("{file}:{line} AssertSameSeqs N1={seq_count}, N22={seq_count2}");
    }

    let mut gsis1 = std::collections::BTreeSet::<uint>::new();
    let mut gsis2 = std::collections::BTreeSet::<uint>::new();
    for i in 0..seq_count {
        let seq1 = &ms1.seqs[i];
        let seq2 = &ms2.seqs[i];
        let gsi1 = get_gsi_by_label(&seq1.label);
        let gsi2 = get_gsi_by_label(&seq2.label);
        if gsi1 >= gn {
            panic!("{file}:{line} AssertSameSeqs GSI1={gsi1} > GN={gn}");
        }
        if gsi2 >= gn {
            panic!("{file}:{line} AssertSameSeqs GSI2={gsi2} > GN={gn}");
        }
        if gsis1.contains(&gsi1) {
            panic!("{file}:{line} AssertSameSeqs dupe GSI1={gsi1}");
        }
        if gsis2.contains(&gsi2) {
            panic!("{file}:{line} AssertSameSeqs dupe GSI2={gsi2}");
        }

        let label1 = ms1.seqs[i].label.clone();
        let label2 = ms2.seqs[i].label.clone();

        let global_label1 = global_ms.seqs[gsi1 as usize].label.clone();
        let global_label2 = global_ms.seqs[gsi2 as usize].label.clone();

        if global_label1 != label1 {
            panic!(
                "{file}:{line} AssertSameSeqs Seq1({i}) GI {gsi1} label '{label1}' != '{global_label1}'"
            );
        }

        if global_label2 != label2 {
            panic!(
                "{file}:{line} AssertSameSeqs Seq2({i}) GI {gsi2} label '{label2}' != '{global_label2}'"
            );
        }

        gsis1.insert(gsi1);
        gsis2.insert(gsi2);
    }

    for gsi1 in &gsis1 {
        if !gsis2.contains(gsi1) {
            panic!("{file}:{line} AssertSameSeqs GSI1={gsi1} missing in MS2");
        }
    }

    for gsi2 in &gsis2 {
        if !gsis1.contains(gsi2) {
            panic!("{file}:{line} AssertSameSeqs GSI2={gsi2} missing in MS1");
        }
    }

    *ASSERT_SAME_SEQS_OK_COUNT.lock().unwrap() += 1;
}
