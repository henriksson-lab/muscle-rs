// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub(crate) static SEQUENCE_NEW_COUNT: std::sync::Mutex<uint> = std::sync::Mutex::new(0);

pub(crate) static SEQUENCE_DELETE_COUNT: std::sync::Mutex<uint> = std::sync::Mutex::new(0);

/// Return a one-line summary of how many Sequence objects have been created and destroyed.
#[track_caller]
pub fn sequence_log_new_delete_counts() -> String {
    format!(
        "Sequence::LogNewDeleteCounts new={}, delete={}\n",
        *SEQUENCE_NEW_COUNT.lock().unwrap(),
        *SEQUENCE_DELETE_COUNT.lock().unwrap()
    )
}

/// Allocate a new Sequence and bump the global new-count.
#[track_caller]
pub fn sequence_new_sequence() -> Sequence {
    *SEQUENCE_NEW_COUNT.lock().unwrap() += 1;
    Sequence::default()
}

/// Drop a Sequence and bump the global delete-count.
#[track_caller]
pub fn sequence_delete_sequence(_seq: Sequence) {
    *SEQUENCE_DELETE_COUNT.lock().unwrap() += 1;
}

/// Read the next FASTA record from `infile` into `seq`, optionally stripping gaps.
#[track_caller]
pub fn sequence_from_file_buffer(
    seq: &mut Sequence,
    infile: &mut TextFile,
    strip_gaps: bool,
) -> bool {
    if infile.pos >= infile.data.len() && infile.pushed_back.is_none() {
        return false;
    }

    seq.label.clear();

    loop {
        if infile.pos >= infile.data.len() && infile.pushed_back.is_none() {
            if seq.label.is_empty() {
                return false;
            }
            panic!("Unexpected EOF while reading FASTA label");
        }
        let Some(line) = text_file_get_line(infile, 4096) else {
            if seq.label.is_empty() {
                return false;
            }
            panic!("Unexpected EOF while reading FASTA label");
        };
        seq.label = line;
        if !seq.label.is_empty() {
            break;
        }
    }

    if !seq.label.starts_with('>') {
        panic!("Expected '>' in FASTA, got '{}'", seq.label);
    }
    seq.label = seq.label[1..].to_string();
    seq.char_vec.clear();

    while let Some(mut ch) = text_file_get_char(infile) {
        if ch == b'>' {
            infile.pushed_back = Some(ch);
            break;
        }
        if (ch as char).is_ascii_whitespace() {
            continue;
        }
        if strip_gaps && (ch == b'-' || ch == b'.') {
            continue;
        }
        if strip_gaps {
            ch = ch.to_ascii_uppercase();
        }
        seq.char_vec.push(ch as char);
    }
    true
}

/// Write `seq` to `file` as a single FASTA record.
#[track_caller]
pub fn sequence_write_mfa(seq: &Sequence, file: &mut TextFile) {
    let s = sequence_get_seq_as_string(seq);
    text_file_put_string(file, &seq_to_fasta_l2561(&s, &seq.label));
}

/// Return a deep clone of `seq`.
pub fn sequence_clone(seq: &Sequence) -> Sequence {
    Sequence {
        label: seq.label.clone(),
        char_vec: seq.char_vec.clone(),
    }
}

/// Apply an alignment path string to `seq`, inserting '-' for delete states matching `id`.
pub fn sequence_add_gaps_path(seq: &Sequence, path: &str, id: char) -> Sequence {
    let mut ret = Sequence {
        label: seq.label.clone(),
        char_vec: Vec::new(),
    };
    let mut data_iter = seq.char_vec.iter();
    for c in path.chars() {
        if c == 'M' || c == 'B' || c == id {
            ret.char_vec.push(*data_iter.next().unwrap());
        } else {
            ret.char_vec.push('-');
        }
    }
    ret
}

/// Map each ungapped position to its column in `seq` (e.g. "ATGCC---GT" -> [0,1,2,3,4,8,9]).
pub fn sequence_get_pos_to_col(seq: &str) -> Vec<uint> {
    let mut pos_to_col = Vec::new();
    for (col, c) in seq.bytes().enumerate() {
        if c != b'-' {
            pos_to_col.push(col as uint);
        }
    }
    pos_to_col
}

/// Return the sequence as a plain string.
pub fn sequence_get_seq_as_string(seq: &Sequence) -> String {
    let mut out = String::new();
    for c in &seq.char_vec {
        out.push(*c);
    }
    out
}

/// Map each column to its ungapped position, or uint::MAX for gap columns.
pub fn sequence_get_col_to_pos(seq: &str) -> Vec<uint> {
    let mut col_to_pos = Vec::new();
    let mut pos = 0;
    for c in seq.bytes() {
        if c == b'-' {
            col_to_pos.push(uint::MAX);
        } else {
            col_to_pos.push(pos);
            pos += 1;
        }
    }
    col_to_pos
}

/// Return a copy of `seq` with all '-' gap characters removed.
pub fn sequence_copy_delete_gaps(seq: &str) -> String {
    let mut out = String::new();
    for c in seq.bytes() {
        if c != b'-' {
            out.push(char::from(c));
        }
    }
    out
}

/// Populate `seq` from a label and a sequence string.
pub fn sequence_from_string(seq: &mut Sequence, label: &str, s: &str) {
    seq.label = label.to_string();
    seq.char_vec.clear();
    for c in s.chars() {
        seq.char_vec.push(c);
    }
}

/// Render `seq` as a short log line: characters, label and length.
#[track_caller]
pub fn sequence_log_me(seq: &Sequence) -> String {
    let s = sequence_get_seq_as_string(seq);
    format!("{}  >{} ({})\n", s, seq.label, seq.char_vec.len())
}
