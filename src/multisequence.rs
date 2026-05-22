// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Load a multi-FASTA file into `ms`, optionally stripping gap characters.
#[track_caller]
pub fn multi_sequence_load_mfa_l8(ms: &mut MultiSequence, filename: &str, strip_gaps: bool) {
    let mut infile = text_file_text_file_l5(filename, false);
    multi_sequence_load_mfa_l99(ms, &mut infile, strip_gaps);
    text_file_destructor_text_file(&mut infile);
}

/// Assert the `owners` vector matches the sequence count (invariant check).
#[track_caller]
pub fn multi_sequence_assert_seq_ids(ms: &MultiSequence) {
    assert_eq!(ms.owners.len(), ms.seqs.len());
}

/// Clear all sequences and ownership flags from a `MultiSequence`.
pub fn multi_sequence_clear(ms: &mut MultiSequence) {
    assert_eq!(ms.owners.len(), ms.seqs.len());
    ms.seqs.clear();
    ms.owners.clear();
}

/// Deep-copy `rhs` into `ms`, replacing any existing contents.
pub fn multi_sequence_copy(ms: &mut MultiSequence, rhs: &MultiSequence) {
    multi_sequence_clear(ms);
    for seq in &rhs.seqs {
        ms.seqs.push(sequence_clone(seq));
        ms.owners.push(true);
    }
}

/// Return true if every sequence has a gap character at the given column.
pub fn multi_sequence_col_is_all_gaps(ms: &MultiSequence, col: uint) -> bool {
    for seq in &ms.seqs {
        let c = seq.char_vec[col as usize];
        if c != '-' && c != '.' {
            return false;
        }
    }
    true
}

/// Format a `MultiSequence` for debug logging.
#[track_caller]
pub fn multi_sequence_log_me(ms: &MultiSequence) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!(
        "MultiSequence::LogMe({:p}), {} seqs\n",
        ms,
        ms.seqs.len()
    ));
    for seq in &ms.seqs {
        out.push_str(&sequence_log_me(seq));
    }
    out
}

/// Return the (gapped) length of the sequence at the given index.
pub fn multi_sequence_get_seq_length(ms: &MultiSequence, seq_index: uint) -> uint {
    ms.seqs[seq_index as usize].char_vec.len() as uint
}

/// Return sequence indexes sorted by descending sequence length.
/// Uses the unstable quicksort from C++ `sort.h` so ties resolve identically
/// to the C++ MUSCLE binary (uclust/uclustpd output is sensitive to this).
pub fn multi_sequence_get_length_order(ms: &MultiSequence) -> Vec<uint> {
    quick_sort_order_desc_by(ms.seqs.len(), |a, b| {
        ms.seqs[a].char_vec.len().cmp(&ms.seqs[b].char_vec.len())
    })
}

/// Read all FASTA records from `infile` into `ms`, renaming duplicate labels when not allowed.
#[track_caller]
pub fn multi_sequence_load_mfa_l99(
    ms: &mut MultiSequence,
    infile: &mut TextFile,
    strip_gaps: bool,
) {
    let mut labels = std::collections::BTreeSet::<String>::new();
    let mut dupe_count = 0_u32;
    loop {
        let mut seq = Sequence::default();
        let ok = sequence_from_file_buffer(&mut seq, infile, strip_gaps);
        if !ok {
            break;
        }

        let mut label = seq.label.clone();
        let mut dupe = false;
        for i in 1..100 {
            if !labels.contains(&label) {
                break;
            }
            if !ms.dupe_labels_ok {
                dupe = true;
                label = format!("{} dupelabel{}", seq.label, i);
            }
        }
        if dupe {
            seq.label = label.clone();
            dupe_count += 1;
        }
        labels.insert(label);
        ms.seqs.push(seq);
        ms.owners.push(true);
    }
    let _ = dupe_count;
}

/// Return the column count of an aligned `MultiSequence`; panic if not aligned.
pub fn multi_sequence_get_col_count(ms: &MultiSequence) -> uint {
    assert!(multi_sequence_is_aligned(ms));
    ms.seqs[0].char_vec.len() as uint
}

/// Return true if all sequences have the same length and there is at least one sequence.
pub fn multi_sequence_is_aligned(ms: &MultiSequence) -> bool {
    if ms.seqs.is_empty() {
        return false;
    }
    let col_count0 = ms.seqs[0].char_vec.len();
    for i in 1..ms.seqs.len() {
        if ms.seqs[i].char_vec.len() != col_count0 {
            return false;
        }
    }
    true
}

/// Find a sequence by exact label; panic on missing if `fail_on_error`, else return `uint::MAX`.
pub fn multi_sequence_get_seq_index(ms: &MultiSequence, label: &str, fail_on_error: bool) -> uint {
    for (i, seq) in ms.seqs.iter().enumerate() {
        if seq.label == label {
            return i as uint;
        }
    }
    if fail_on_error {
        panic!("Label not found >{label}");
    }
    uint::MAX
}

/// Guess whether the sequences are nucleic acid by sampling 100 random characters.
pub fn multi_sequence_guess_is_nucleo(ms: &MultiSequence) -> bool {
    let seq_count = ms.seqs.len() as uint;
    if seq_count == 0 {
        // Match C++ behaviour: no input → fall back to amino. Avoids a
        // division-by-zero in the `randu32() % seq_count` sample step.
        return false;
    }
    let mut nucleo_count = 0;
    for _ in 0..100 {
        let seq_index = randu32() % seq_count;
        let seq = &ms.seqs[seq_index as usize];
        if seq.char_vec.is_empty() {
            continue;
        }
        let pos = randu32() % seq.char_vec.len() as uint;
        match seq.char_vec[pos as usize] {
            'A' | 'a' | 'C' | 'c' | 'G' | 'g' | 'T' | 't' | 'U' | 'u' => nucleo_count += 1,
            _ => {}
        }
    }
    nucleo_count > 75
}

/// Populate `ms` from parallel arrays of labels and sequence strings.
pub fn multi_sequence_from_strings(ms: &mut MultiSequence, labels: &[String], seqs: &[String]) {
    multi_sequence_clear(ms);
    assert_eq!(labels.len(), seqs.len());
    for i in 0..seqs.len() {
        let mut seq = Sequence::default();
        sequence_from_string(&mut seq, &labels[i], &seqs[i]);
        ms.seqs.push(seq);
        ms.owners.push(true);
    }
}

/// Convert an aligned `MultiSequence` into an `MSA`-style `MultiSequence` with explicit columns.
#[track_caller]
pub fn multi_sequence_to_msa(ms: &MultiSequence) -> MultiSequence {
    let seq_count = ms.seqs.len();
    let col_count = multi_sequence_get_col_count(ms);
    let mut msa = MultiSequence::default();
    msa_set_size(&mut msa, seq_count as uint, col_count);
    for seq_index in 0..seq_count {
        let seq = &ms.seqs[seq_index];
        assert_eq!(seq.char_vec.len(), col_count as usize);
        for col in 0..col_count as usize {
            msa_set_char(&mut msa, seq_index as uint, col as uint, seq.char_vec[col]);
        }
        msa_set_seq_name(&mut msa, seq_index as uint, &seq.label);
    }
    msa
}

/// Return the mean of the per-sequence (gapped) lengths.
pub fn multi_sequence_get_mean_seq_length(ms: &MultiSequence) -> f64 {
    let seq_count = ms.seqs.len();
    if seq_count == 0 {
        return 0.0;
    }
    let mut sum_seq_length = 0.0;
    for seq in &ms.seqs {
        sum_seq_length += seq.char_vec.len() as f64;
    }
    sum_seq_length / seq_count as f64
}

/// Return the maximum sequence length across all rows.
pub fn multi_sequence_get_max_seq_length(ms: &MultiSequence) -> uint {
    let mut max_seq_length = 0;
    for seq in &ms.seqs {
        max_seq_length = std::cmp::max(max_seq_length, seq.char_vec.len() as uint);
    }
    max_seq_length
}

/// Return the minimum sequence length across all rows (0 if empty).
pub fn multi_sequence_get_min_seq_length(ms: &MultiSequence) -> uint {
    if ms.seqs.is_empty() {
        return 0;
    }
    let mut min_seq_length = uint::MAX;
    for seq in &ms.seqs {
        min_seq_length = std::cmp::min(min_seq_length, seq.char_vec.len() as uint);
    }
    min_seq_length
}
