// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Reads all sequences from an open FASTA `TextFile` into a `MultiSequence`.
#[track_caller]
pub fn msa_from_fasta_file_l5(file: &mut TextFile) -> MultiSequence {
    let mut labels = Vec::new();
    let mut seqs = Vec::new();
    while let Some((label, seq)) = get_fasta_seq(file, false) {
        labels.push(label);
        seqs.push(seq);
    }
    msa_from_strings2(&labels, &seqs)
}

/// Reads an MSA from a FASTA file without forcing letters to uppercase.
#[track_caller]
pub fn msa_from_fasta_file_preserve_case(file_name: &str) -> MultiSequence {
    let _save_upper = *FASTA_UPPER.lock().unwrap();
    *FASTA_UPPER.lock().unwrap() = false;
    let msa = msa_from_fasta_file_l95(file_name);
    *FASTA_UPPER.lock().unwrap() = true;
    msa
}

/// Builds an MSA from raw FASTA lines (`>label` headers followed by sequence data).
#[track_caller]
pub fn msa_from_strings(strings: &[String]) -> MultiSequence {
    if strings.is_empty() {
        panic!("MSA::FromStrings, no data");
    }
    let mut labels = Vec::new();
    let mut seqs = Vec::new();
    let mut curr_seq = String::new();
    for s in strings {
        let s0 = s.as_bytes().first().copied().unwrap_or(0) as char;
        if s0 == '>' {
            if !labels.is_empty() {
                seqs.push(curr_seq.clone());
            }
            labels.push(s[1..].to_string());
            curr_seq.clear();
        } else {
            for c in s.chars() {
                if !c.is_whitespace() {
                    curr_seq.push(c);
                }
            }
        }
    }
    seqs.push(curr_seq);
    msa_from_strings2(&labels, &seqs)
}

/// Builds an MSA from parallel `labels` and `seqs` vectors; checks aligned lengths.
#[track_caller]
pub fn msa_from_strings2(labels: &[String], seqs: &[String]) -> MultiSequence {
    let seq_count = labels.len();
    if seqs.len() != seq_count {
        panic!("Invalid FASTA, {} labels {} seqs", labels.len(), seqs.len());
    }
    if seq_count == 0 {
        panic!("Empty FASTA");
    }
    let col_count = seqs[0].len();
    let mut msa = MultiSequence::default();
    for seq_index in 0..seq_count {
        let label = &labels[seq_index];
        let seq = &seqs[seq_index];
        let n = seq.len();
        if n != col_count {
            panic!("MSA not aligned, seq lengths {col_count}, {n}");
        }
        msa.seqs.push(Sequence {
            label: label.clone(),
            char_vec: seq.chars().collect(),
        });
        msa.owners.push(true);
    }
    msa
}

/// Reads an MSA from the FASTA file at `file_name`.
#[track_caller]
pub fn msa_from_fasta_file_l95(file_name: &str) -> MultiSequence {
    let mut tf = text_file_text_file_l5(file_name, false);
    let msa = msa_from_fasta_file_l5(&mut tf);
    text_file_destructor_text_file(&mut tf);
    msa
}

/// Writes `msa` to `file_name` in FASTA format; a no-op if `file_name` is empty.
/// Uses the legacy `MSA::ToFASTAFile` 60-char block layout.
#[track_caller]
pub fn msa_to_fasta_file_l103(msa: &MultiSequence, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    std::fs::write(file_name, msa_to_fasta_file_l124(msa)).expect("failed to write MSA FASTA");
}

/// Writes `msa` to `file_name` using the C++ `MultiSequence::WriteMFA` layout
/// (80-character rows via `SeqToFasta`). No-op if `file_name` is empty.
#[track_caller]
pub fn multi_sequence_write_mfa(msa: &MultiSequence, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    // Match C++ `MultiSequence::WriteMFA`: an empty MSA produces an empty
    // file rather than panicking in `multi_sequence_is_aligned`. Hit by
    // `cmd_super4 -perm all`, whose per-perm outputs land in
    // `final_msa_{none,abc,acb,bca}` and leaves the unused `final_msa` slot
    // empty.
    let body = if msa.seqs.is_empty() {
        String::new()
    } else {
        msa_to_fasta_file_l112(msa)
    };
    std::fs::write(file_name, body).expect("failed to write MultiSequence FASTA");
}

/// Formats `msa` as a FASTA string using 80-character lines (variant of `ToFASTAFile`).
#[track_caller]
pub fn msa_to_fasta_file_l112(msa: &MultiSequence) -> String {
    const ROW_LEN: usize = 80;
    let col_count = multi_sequence_get_col_count(msa) as usize;
    if col_count == 0 {
        return String::new();
    }
    let block_count = col_count.div_ceil(ROW_LEN);
    let mut out = String::new();
    for seq in &msa.seqs {
        out.push('>');
        out.push_str(&seq.label);
        out.push('\n');
        for block_index in 0..block_count {
            let from = block_index * ROW_LEN;
            let to = std::cmp::min(from + ROW_LEN, col_count);
            for pos in from..to {
                out.push(seq.char_vec[pos]);
            }
            out.push('\n');
        }
    }
    out
}

/// Formats `msa` as a FASTA string using 60-letter blocks per line.
#[track_caller]
pub fn msa_to_fasta_file_l124(msa: &MultiSequence) -> String {
    const FASTA_BLOCK: usize = 60;
    let col_count = multi_sequence_get_col_count(msa) as usize;
    assert!(col_count > 0);
    let lines_per_seq = (col_count - 1) / FASTA_BLOCK + 1;
    let mut out = String::new();
    for seq in &msa.seqs {
        out.push('>');
        out.push_str(&seq.label);
        out.push('\n');
        let mut n = 0usize;
        for line in 0..lines_per_seq {
            let mut letters = col_count - line * FASTA_BLOCK;
            if letters > FASTA_BLOCK {
                letters = FASTA_BLOCK;
            }
            for _ in 0..letters {
                out.push(seq.char_vec[n]);
                n += 1;
            }
            out.push('\n');
        }
    }
    out
}
