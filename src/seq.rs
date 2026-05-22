// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Set the sequence name.
#[track_caller]
pub fn seq_set_name(seq: &mut Seq, name: &str) {
    seq.name = Some(name.to_string());
}

/// Write the sequence to `file` in FASTA format, wrapping at 60 characters per line.
#[track_caller]
pub fn seq_to_fasta_file(seq: &Seq, file: &mut TextFile) {
    text_file_put_format(file, &format!(">{}\n", seq.name.as_deref().unwrap_or("")));
    for n in 0..seq.chars.len() {
        if n > 0 && n % 60 == 0 {
            text_file_put_string(file, "\n");
        }
        text_file_put_char(file, seq.chars[n] as u8);
    }
    text_file_put_string(file, "\n");
}

/// Read the next FASTA record from `file` into `seq`. Returns true on end-of-file.
#[track_caller]
pub fn seq_from_fasta_file(seq: &mut Seq, file: &mut TextFile) -> bool {
    seq.chars.clear();
    seq.name = None;
    seq.id = uint::MAX;

    let Some(line) = text_file_get_line(file, 16000) else {
        return true;
    };
    if !line.starts_with('>') {
        panic!(
            "Expecting '>' in FASTA file {} line {}",
            file.name, file.line_nr
        );
    }
    if line.len() == 1 {
        panic!(
            "Missing annotation following '>' in FASTA file {} line {}",
            file.name, file.line_nr
        );
    }
    seq.name = Some(line[1..].to_string());

    let mut pos = text_file_get_pos_l258(file);
    loop {
        let line = match text_file_get_line(file, 16000) {
            Some(line) => line,
            None => {
                if seq.chars.is_empty() {
                    panic!(
                        "Empty sequence in FASTA file {} line {}",
                        file.name, file.line_nr
                    );
                }
                return false;
            }
        };
        if line.starts_with('>') {
            if seq.chars.is_empty() {
                panic!(
                    "Empty sequence in FASTA file {} line {}",
                    file.name, file.line_nr
                );
            }
            text_file_set_pos_l267(file, pos);
            return false;
        }
        for c in line.chars() {
            if c.is_ascii_whitespace() {
                continue;
            }
            if c == '-' || c == '.' {
                continue;
            }
            if !c.is_ascii_alphabetic() {
                continue;
            }
            seq.chars.push(c.to_ascii_uppercase());
        }
        pos = text_file_get_pos_l258(file);
    }
}

/// Build a single-row MSA from `seq` with all gap characters removed.
#[track_caller]
pub fn seq_extract_ungapped(seq: &Seq, msa: &mut MultiSequence) {
    msa_free(msa);
    msa_set_size(msa, 1, 1);
    let mut ungapped_pos = 0;
    for &c in &seq.chars {
        if c != '-' && c != '.' {
            msa_set_char(msa, 0, ungapped_pos, c);
            ungapped_pos += 1;
        }
    }
    msa_set_seq_name(msa, 0, seq.name.as_deref().unwrap_or(""));
}

/// Copy `rhs` into `seq` (chars, name, id).
#[track_caller]
pub fn seq_copy(seq: &mut Seq, rhs: &Seq) {
    seq.chars.clear();
    for c in &rhs.chars {
        seq.chars.push(*c);
    }
    seq.name = rhs.name.clone();
    seq.id = rhs.id;
}

/// Copy `rhs` into `seq` with the character order reversed.
#[track_caller]
pub fn seq_copy_reversed(seq: &mut Seq, rhs: &Seq) {
    seq.chars.clear();
    for c in rhs.chars.iter().rev() {
        seq.chars.push(*c);
    }
    seq.name = rhs.name.clone();
}

/// Remove all gap characters ('-' and '.') from `seq`.
#[track_caller]
pub fn seq_strip_gaps(seq: &mut Seq) {
    seq.chars.retain(|c| *c != '-' && *c != '.');
}

/// Remove gap characters and whitespace from `seq`.
#[track_caller]
pub fn seq_strip_gaps_and_whitespace(seq: &mut Seq) {
    seq.chars
        .retain(|c| !c.is_ascii_whitespace() && *c != '-' && *c != '.');
}

/// Convert all lowercase characters in `seq` to uppercase in place.
#[track_caller]
pub fn seq_to_upper(seq: &mut Seq) {
    for c in &mut seq.chars {
        if c.is_ascii_lowercase() {
            *c = c.to_ascii_uppercase();
        }
    }
}

/// Return the alphabet letter index for the character at position `index`.
#[track_caller]
pub fn seq_get_letter(seq: &Seq, index: uint) -> uint {
    assert!((index as usize) < seq.chars.len());
    let state = ALPHA_STATE.lock().unwrap();
    state.char_to_letter[seq.chars[index as usize] as usize]
}

/// Case-insensitive sequence equality, with gap characters treated as equivalent.
#[track_caller]
pub fn seq_eq_ignore_case(seq: &Seq, s: &Seq) -> bool {
    let n = seq.chars.len();
    if n != s.chars.len() {
        return false;
    }
    for i in 0..n {
        let c1 = seq.chars[i];
        let c2 = s.chars[i];
        if c1 == '-' || c1 == '.' {
            if c2 != '-' && c2 != '.' {
                return false;
            }
        } else if c1.to_ascii_uppercase() != c2.to_ascii_uppercase() {
            return false;
        }
    }
    true
}

/// Exact character-by-character sequence equality.
#[track_caller]
pub fn seq_eq(seq: &Seq, s: &Seq) -> bool {
    if seq.chars.len() != s.chars.len() {
        return false;
    }
    for i in 0..seq.chars.len() {
        if seq.chars[i] != s.chars[i] {
            return false;
        }
    }
    true
}

/// Case-insensitive equality after stripping gap characters from both sequences.
#[track_caller]
pub fn seq_eq_ignore_case_and_gaps(seq: &Seq, s: &Seq) -> bool {
    let this_length = seq.chars.len();
    let other_length = s.chars.len();
    let mut this_pos = 0usize;
    let mut other_pos = 0usize;

    loop {
        if this_pos == this_length && other_pos == other_length {
            break;
        }

        let c_this;
        loop {
            if this_pos == this_length {
                c_this = None;
                break;
            }
            let c = seq.chars[this_pos];
            this_pos += 1;
            if c != '-' && c != '.' {
                c_this = Some(c.to_ascii_uppercase());
                break;
            }
        }

        let c_other;
        loop {
            if other_pos == other_length {
                c_other = None;
                break;
            }
            let c = s.chars[other_pos];
            other_pos += 1;
            if c != '-' && c != '.' {
                c_other = Some(c.to_ascii_uppercase());
                break;
            }
        }

        if c_this != c_other {
            return false;
        }
    }
    true
}

/// Number of non-gap characters in `seq`.
pub fn seq_get_ungapped_length(seq: &Seq) -> uint {
    let mut ungapped_length = 0;
    for c in &seq.chars {
        if *c != '-' && *c != '.' {
            ungapped_length += 1;
        }
    }
    ungapped_length
}

/// Render `seq` as a FASTA-like text block for logging.
#[track_caller]
pub fn seq_log_me(seq: &Seq) -> String {
    format!(
        ">{}\n{}\n",
        seq.name.as_deref().unwrap_or(""),
        seq.chars.iter().collect::<String>()
    )
}

/// Populate `seq` from a plain text sequence and a name.
#[track_caller]
pub fn seq_from_string(seq: &mut Seq, seq_text: &str, name: &str) {
    seq.chars.clear();
    for c in seq_text.chars() {
        seq.chars.push(c);
    }
    seq.name = Some(name.to_string());
}

/// True if `seq` contains any gap character.
pub fn seq_has_gap(seq: &Seq) -> bool {
    for c in &seq.chars {
        if *c == '-' || *c == '.' {
            return true;
        }
    }
    false
}
