// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct PathEdge; // original: PathEdge (muscle/src/msa.h)

#[derive(Clone, Debug, Default)]
pub struct TextFile {
    pub data: Vec<u8>,
    pub pos: usize,
    pub line_nr: uint,
    pub col_nr: uint,
    pub name: String,
    pub last_char_was_eol: bool,
    pub pushed_back: Option<u8>,
} // original: TextFile (muscle/src/msa.h)

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seq {
    pub chars: Vec<char>,
    pub name: Option<String>,
    pub id: uint,
} // original: Seq (muscle/src/msa.h)

#[derive(Clone, Debug, Default)]
pub struct ClusterNode; // original: ClusterNode (muscle/src/msa.h)

#[derive(Clone, Debug, Default)]
pub struct NodeCounts; // original: NodeCounts (muscle/src/msa.h)

#[derive(Clone, Debug, Default)]
pub struct DataBuffer; // original: DataBuffer (muscle/src/msa.h)

#[derive(Clone, Debug, Default)]
pub struct MSA; // original: MSA (muscle/src/msa.h)

/// Default constructor: return an empty MultiSequence (Rust port of `MSA::MSA`).
#[track_caller]
pub fn msa_msa() -> MultiSequence {
    MultiSequence::default()
}

/// Destructor: release all memory held by the MSA (Rust port of `MSA::~MSA`).
#[track_caller]
pub fn msa_destructor_msa(msa: &mut MultiSequence) {
    msa_free(msa);
}

/// Clear all sequences and id mappings, leaving an empty MSA.
#[track_caller]
pub fn msa_free(msa: &mut MultiSequence) {
    msa.seqs.clear();
    msa.owners.clear();
    msa.id_to_seq_index.clear();
    msa.seq_index_to_id.clear();
}

/// Resize the MSA to `seq_count` rows of `col_count` placeholder characters.
#[track_caller]
pub fn msa_set_size(msa: &mut MultiSequence, seq_count: uint, col_count: uint) {
    msa_free(msa);
    if seq_count == 0 && col_count == 0 {
        return;
    }
    for _ in 0..seq_count {
        msa.seqs.push(Sequence {
            label: String::new(),
            char_vec: vec!['?'; col_count as usize],
        });
        msa.owners.push(true);
    }
    if msa.id_count > 0 {
        msa.id_to_seq_index = vec![uint::MAX; msa.id_count as usize];
        msa.seq_index_to_id = vec![uint::MAX; seq_count as usize];
    }
}

/// Format the MSA into a multi-block textual representation for logging.
#[track_caller]
pub fn msa_log_me(msa: &MultiSequence) -> String {
    if multi_sequence_get_col_count(msa) == 0 {
        return "MSA empty\n".to_string();
    }

    let cols_per_line = 50;
    let col_count = multi_sequence_get_col_count(msa);
    let lines_per_seq = (col_count - 1) / cols_per_line + 1;
    let mut out = String::new();
    for n in 0..lines_per_seq {
        let start = n * cols_per_line;
        let mut end = col_count;
        if end - start + 1 > cols_per_line {
            end = start + cols_per_line;
        }
        out.push_str("                       ");
        for i in start..end {
            out.push_str(&(i % 10).to_string());
        }
        out.push('\n');
        out.push_str("                       ");
        let mut i = start;
        while i + 9 < end {
            out.push_str(&format!("{:<10}", i));
            i += 10;
        }
        if n == lines_per_seq - 1 {
            out.push_str(&format!(" {:<10}", col_count));
        }
        out.push('\n');
        for seq_index in 0..msa.seqs.len() {
            let name = msa_get_seq_name(msa, seq_index as uint);
            let display_name = if name.len() > 12 { &name[..12] } else { &name };
            out.push_str(&format!("{display_name:>12}"));
            out.push_str("           ");
            for i in start..end {
                out.push(msa_get_char(msa, seq_index as uint, i));
            }
            if !msa.seq_index_to_id.is_empty() {
                out.push_str(&format!(" [{:5}]", msa.seq_index_to_id[seq_index]));
            }
            out.push('\n');
        }
        out.push_str("\n\n");
    }
    out
}

/// Return the character at `(seq_index, index)`, panicking if out of range.
#[track_caller]
pub fn msa_get_char(msa: &MultiSequence, seq_index: uint, index: uint) -> char {
    if (seq_index as usize) >= msa.seqs.len()
        || (index as usize) >= multi_sequence_get_col_count(msa) as usize
    {
        panic!(
            "MSA::GetChar({}/{},{}/{})",
            seq_index,
            msa.seqs.len(),
            index,
            multi_sequence_get_col_count(msa)
        );
    }
    msa.seqs[seq_index as usize].char_vec[index as usize]
}

/// Return the amino-acid letter index at `(seq_index, index)`; panic if not a legal letter.
#[track_caller]
pub fn msa_get_letter(msa: &MultiSequence, seq_index: uint, index: uint) -> uint {
    let c = msa_get_char(msa, seq_index, index);
    let state = ALPHA_STATE.lock().unwrap();
    let letter = state.char_to_letter[c as usize];
    if letter >= 20 {
        panic!(
            "MSA::GetLetter({}/{}, {}/{})='{c}'/{letter}",
            seq_index,
            msa.seqs.len(),
            index,
            multi_sequence_get_col_count(msa)
        );
    }
    letter
}

/// Set the label of one sequence in the MSA.
#[track_caller]
pub fn msa_set_seq_name(msa: &mut MultiSequence, seq_index: uint, name: &str) {
    if (seq_index as usize) >= msa.seqs.len() {
        panic!(
            "MSA::SetSeqName({seq_index}, {name}), count={}",
            msa.seqs.len()
        );
    }
    msa.seqs[seq_index as usize].label = name.to_string();
}

/// Return the label of one sequence (alias for `msa_get_seq_name`).
#[track_caller]
pub fn msa_get_seq_label(msa: &MultiSequence, seq_index: uint) -> String {
    msa_get_seq_name(msa, seq_index)
}

/// Return the label of one sequence, panicking on out-of-range index.
#[track_caller]
pub fn msa_get_seq_name(msa: &MultiSequence, seq_index: uint) -> String {
    if (seq_index as usize) >= msa.seqs.len() {
        panic!("MSA::GetSeqName({seq_index}), count={}", msa.seqs.len());
    }
    msa.seqs[seq_index as usize].label.clone()
}

/// Return true if the character at `(seq_index, index)` is a gap (`-` or `.`).
#[track_caller]
pub fn msa_is_gap(msa: &MultiSequence, seq_index: uint, index: uint) -> bool {
    let c = msa_get_char(msa, seq_index, index);
    c == '-' || c == '.'
}

/// Set one MSA character, growing every row's column buffer with `?` if needed.
#[track_caller]
pub fn msa_set_char(msa: &mut MultiSequence, seq_index: uint, index: uint, c: char) {
    if (seq_index as usize) >= msa.seqs.len() {
        panic!("MSA::SetChar({seq_index},{index})");
    }
    let target_len = index as usize + 1;
    for seq in &mut msa.seqs {
        if seq.char_vec.len() < target_len {
            seq.char_vec.resize(target_len, '?');
        }
    }
    msa.seqs[seq_index as usize].char_vec[index as usize] = c;
}

/// Return a slice of the underlying character buffer for one sequence.
#[track_caller]
pub fn msa_get_seq_char_ptr(msa: &MultiSequence, seq_index: uint) -> &[char] {
    assert!((seq_index as usize) < msa.seqs.len());
    &msa.seqs[seq_index as usize].char_vec
}

/// Return one MSA row as a `String` of length `col_count`.
#[track_caller]
pub fn msa_get_row_str(msa: &MultiSequence, seq_index: uint) -> String {
    let col_count = multi_sequence_get_col_count(msa);
    let seq_char_ptr = msa_get_seq_char_ptr(msa, seq_index);
    let mut row_str = String::new();
    for i in 0..col_count {
        row_str.push(seq_char_ptr[i as usize]);
    }
    row_str
}

/// Extract one MSA row as an ungapped uppercase `Seq` with its name.
#[track_caller]
pub fn msa_get_seq(msa: &MultiSequence, seq_index: uint) -> Seq {
    assert!((seq_index as usize) < msa.seqs.len());
    let mut seq = Seq::default();
    for n in 0..multi_sequence_get_col_count(msa) {
        if !msa_is_gap(msa, seq_index, n) {
            let mut c = msa_get_char(msa, seq_index, n);
            if !c.is_ascii_alphabetic() {
                panic!("Invalid character '{c}' in sequence");
            }
            c = c.to_ascii_uppercase();
            seq.chars.push(c);
        }
    }
    seq.name = Some(msa_get_seq_name(msa, seq_index));
    seq
}

/// Return true if any cell of the MSA is a gap.
#[track_caller]
pub fn msa_has_gap(msa: &MultiSequence) -> bool {
    for seq_index in 0..msa.seqs.len() {
        for n in 0..multi_sequence_get_col_count(msa) {
            if msa_is_gap(msa, seq_index as uint, n) {
                return true;
            }
        }
    }
    false
}

/// Return true if `letter` is a legal amino-acid index (i.e. less than 20).
#[track_caller]
pub fn msa_is_legal_letter(letter: uint) -> bool {
    letter < 20
}

/// Reset the MSA to `seq_count` rows of 500 placeholder columns.
#[track_caller]
pub fn msa_set_seq_count(msa: &mut MultiSequence, seq_count: uint) {
    msa_free(msa);
    msa_set_size(msa, seq_count, 500);
}

/// Copy all characters from column `from_col` into column `to_col` (no-op if equal).
#[track_caller]
pub fn msa_copy_col(msa: &mut MultiSequence, from_col: uint, to_col: uint) {
    assert!(from_col < multi_sequence_get_col_count(msa));
    assert!(to_col < multi_sequence_get_col_count(msa));
    if from_col == to_col {
        return;
    }
    for seq_index in 0..msa.seqs.len() {
        let c = msa_get_char(msa, seq_index as uint, from_col);
        msa_set_char(msa, seq_index as uint, to_col, c);
    }
}

/// Return a deep copy of an MSA including labels and id mappings.
#[track_caller]
pub fn msa_copy(msa: &MultiSequence) -> MultiSequence {
    let seq_count = msa.seqs.len();
    let col_count = multi_sequence_get_col_count(msa);
    let mut copy = MultiSequence::default();
    copy.id_count = msa.id_count;
    msa_set_size(&mut copy, seq_count as uint, col_count);
    for seq_index in 0..seq_count {
        copy.seqs[seq_index].label = msa.seqs[seq_index].label.clone();
        if seq_index < msa.seq_index_to_id.len() {
            let id = msa.seq_index_to_id[seq_index];
            if id != uint::MAX {
                msa_set_seq_id(&mut copy, seq_index as uint, id);
            }
        }
        for col_index in 0..col_count {
            let c = msa_get_char(msa, seq_index as uint, col_index);
            msa_set_char(&mut copy, seq_index as uint, col_index, c);
        }
    }
    copy.dupe_labels_ok = msa.dupe_labels_ok;
    copy
}

/// Count uppercase, lowercase, gap, '.' and '-' characters at one column.
#[track_caller]
pub fn msa_get_upper_lower_gap_count(
    msa: &MultiSequence,
    col_index: uint,
) -> (uint, uint, uint, uint, uint) {
    let mut nu = 0;
    let mut nl = 0;
    let mut ng = 0;
    let mut n_dots = 0;
    let mut n_dashes = 0;
    for seq_index in 0..msa.seqs.len() {
        let c = msa_get_char(msa, seq_index as uint, col_index);
        if c == '.' {
            n_dots += 1;
            ng += 1;
        } else if c == '-' {
            n_dashes += 1;
            ng += 1;
        } else if c.is_ascii_uppercase() {
            nu += 1;
        } else if c.is_ascii_lowercase() {
            nl += 1;
        }
    }
    (nu, nl, ng, n_dots, n_dashes)
}

/// Return the number of gap characters in the given column.
#[track_caller]
pub fn msa_get_gap_count(msa: &MultiSequence, col_index: uint) -> uint {
    let mut n = 0;
    for seq_index in 0..msa.seqs.len() {
        if msa_is_gap(msa, seq_index as uint, col_index) {
            n += 1;
        }
    }
    n
}

/// Return true if every row has a gap at `col_index`.
#[track_caller]
pub fn msa_is_gap_column(msa: &MultiSequence, col_index: uint) -> bool {
    assert!(!msa.seqs.is_empty());
    for seq_index in 0..msa.seqs.len() {
        if !msa_is_gap(msa, seq_index as uint, col_index) {
            return false;
        }
    }
    true
}

/// Find a sequence by label (case-insensitive); panic if missing when `fail_on_error`.
#[track_caller]
pub fn msa_get_seq_index_l367(msa: &MultiSequence, label: &str, fail_on_error: bool) -> uint {
    for seq_index in 0..msa.seqs.len() {
        if msa_get_seq_name(msa, seq_index as uint).eq_ignore_ascii_case(label) {
            return seq_index as uint;
        }
    }
    if fail_on_error {
        panic!("Not found >{label}");
    }
    uint::MAX
}

/// Find a sequence by name (case-insensitive), returning `None` if not present.
#[track_caller]
pub fn msa_get_seq_index_l377(msa: &MultiSequence, seq_name: &str) -> Option<uint> {
    for seq_index in 0..msa.seqs.len() {
        if msa_get_seq_name(msa, seq_index as uint).eq_ignore_ascii_case(seq_name) {
            return Some(seq_index as uint);
        }
    }
    None
}

/// Delete a single column from every sequence.
#[track_caller]
pub fn msa_delete_col(msa: &mut MultiSequence, col_index: uint) {
    assert!(col_index < multi_sequence_get_col_count(msa));
    for seq in &mut msa.seqs {
        seq.char_vec.remove(col_index as usize);
    }
}

/// Delete `col_count` consecutive columns starting at `col_index`.
#[track_caller]
pub fn msa_delete_columns(msa: &mut MultiSequence, col_index: uint, col_count: uint) {
    for _ in 0..col_count {
        msa_delete_col(msa, col_index);
    }
}

/// Load an MSA from an open FASTA-formatted text file.
#[track_caller]
pub fn msa_from_file(file: &mut TextFile) -> MultiSequence {
    msa_from_fasta_file_l5(file)
}

/// Format a single byte as a left-justified string padded to `u_width`.
pub fn fmt_char(c: byte, u_width: uint) -> String {
    let mut s = String::new();
    s.push(char::from(c));
    for _ in 0..u_width.saturating_sub(1) {
        s.push(' ');
    }
    s
}

/// Format a positive integer (or `.` if zero) right-padded to `u_width`.
pub fn fmt_int(u: uint, u_width: uint) -> String {
    let mut s = if u > 0 {
        u.to_string()
    } else {
        ".".to_string()
    };
    let n = s.len() as uint;
    if n < u_width {
        for _ in 0..u_width - n {
            s.push(' ');
        }
    }
    s
}

/// Format an integer right-padded to `u_width`, including zero as `0`.
pub fn fmt_int0(u: uint, u_width: uint) -> String {
    let mut s = u.to_string();
    let n = s.len() as uint;
    if n < u_width {
        for _ in 0..u_width - n {
            s.push(' ');
        }
    }
    s
}

/// Return a string of `n` space characters.
pub fn fmt_pad(n: uint) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push(' ');
    }
    s
}

/// Build a label list and label-to-index map; panic on duplicate labels.
#[track_caller]
pub fn msa_get_label_to_seq_index(
    msa: &MultiSequence,
) -> (Vec<String>, std::collections::BTreeMap<String, uint>) {
    let mut labels = Vec::new();
    let mut label_to_seq_index = std::collections::BTreeMap::new();
    for seq_index in 0..msa.seqs.len() {
        let label = msa_get_seq_name(msa, seq_index as uint);
        if label_to_seq_index.contains_key(&label) {
            panic!("Dupe  label >{label}");
        }
        labels.push(label.clone());
        label_to_seq_index.insert(label, seq_index as uint);
    }
    (labels, label_to_seq_index)
}

/// Build a single-row MSA from one `Sequence`.
#[track_caller]
pub fn msa_from_sequence(seq: &Sequence) -> MultiSequence {
    let mut msa = MultiSequence::default();
    msa.seqs.push(sequence_clone(seq));
    msa.owners.push(true);
    msa
}

/// Return a clone of the given `MultiSequence` (identity adapter).
#[track_caller]
pub fn msa_from_multi_sequence(ms: &MultiSequence) -> MultiSequence {
    ms.clone()
}

/// Build a single-row MSA from a `Seq` value.
#[track_caller]
pub fn msa_from_seq(seq: &Seq) -> MultiSequence {
    let mut msa = MultiSequence::default();
    let label = seq.name.clone().unwrap_or_default();
    msa.seqs.push(Sequence {
        label,
        char_vec: seq.chars.clone(),
    });
    msa.owners.push(true);
    msa
}

/// Return the number of non-gap characters in row `seq_index` up to and including `col_index`.
#[track_caller]
pub fn msa_get_char_count(msa: &MultiSequence, seq_index: uint, col_index: uint) -> uint {
    assert!((seq_index as usize) < msa.seqs.len());
    assert!(col_index < multi_sequence_get_col_count(msa));

    let mut col = 0;
    for n in 0..=col_index {
        if !msa_is_gap(msa, seq_index, n) {
            col += 1;
        }
    }
    col
}

/// Replace row `to_seq_index` in `msa` with a clone of row `from_seq_index` from `msa_from`.
#[track_caller]
pub fn msa_copy_seq(
    msa: &mut MultiSequence,
    to_seq_index: uint,
    msa_from: &MultiSequence,
    from_seq_index: uint,
) {
    assert!((to_seq_index as usize) < msa.seqs.len());
    let col_count = multi_sequence_get_col_count(msa_from);
    assert!(
        multi_sequence_get_col_count(msa) == col_count || multi_sequence_get_col_count(msa) == 0
    );
    let from_seq = &msa_from.seqs[from_seq_index as usize];
    msa.seqs[to_seq_index as usize] = sequence_clone(from_seq);
}

/// Return a borrowed slice of one sequence's character buffer.
#[track_caller]
pub fn msa_get_seq_buffer(msa: &MultiSequence, seq_index: uint) -> &[char] {
    assert!((seq_index as usize) < msa.seqs.len());
    &msa.seqs[seq_index as usize].char_vec
}

/// Remove one sequence from the MSA.
#[track_caller]
pub fn msa_delete_seq(msa: &mut MultiSequence, seq_index: uint) {
    assert!((seq_index as usize) < msa.seqs.len());
    msa.seqs.remove(seq_index as usize);
    if (seq_index as usize) < msa.owners.len() {
        msa.owners.remove(seq_index as usize);
    }
}

/// Return true if every row has a gap at `col_index` (alias of `msa_is_gap_column`).
#[track_caller]
pub fn msa_is_empty_col(msa: &MultiSequence, col_index: uint) -> bool {
    let seq_count = msa.seqs.len();
    for seq_index in 0..seq_count {
        if !msa_is_gap(msa, seq_index as uint, col_index) {
            return false;
        }
    }
    true
}

/// Placeholder: aligned-col to original-col mapping (not yet implemented in this port).
#[track_caller]
pub fn msa_aligned_col_index_to_col_index() {
    panic!("MSA::AlignedColIndexToColIndex not implemented");
}

/// Compare two sequences across alignments after stripping gaps and ignoring case.
#[track_caller]
pub fn msa_seqs_eq(
    a1: &MultiSequence,
    seq_index1: uint,
    a2: &MultiSequence,
    seq_index2: uint,
) -> bool {
    let mut s1 = msa_get_seq(a1, seq_index1);
    let mut s2 = msa_get_seq(a2, seq_index2);
    seq_strip_gaps(&mut s1);
    seq_strip_gaps(&mut s2);
    seq_eq_ignore_case(&s1, &s2)
}

/// Return the number of non-gap characters in one row.
#[track_caller]
pub fn msa_get_ungapped_seq_length(msa: &MultiSequence, seq_index: uint) -> uint {
    assert!((seq_index as usize) < msa.seqs.len());
    let mut length = 0;
    for col_index in 0..multi_sequence_get_col_count(msa) {
        if !msa_is_gap(msa, seq_index, col_index) {
            length += 1;
        }
    }
    length
}

/// Compute pairwise percent identity over aligned non-gap positions, returning (pct, positions).
#[track_caller]
pub fn msa_get_pwid(msa: &MultiSequence, seq_index1: uint, seq_index2: uint) -> (f64, uint) {
    assert!((seq_index1 as usize) < msa.seqs.len());
    assert!((seq_index2 as usize) < msa.seqs.len());

    let mut same_count = 0;
    let mut pos_count = 0;
    for col_index in 0..multi_sequence_get_col_count(msa) {
        let c1 = msa_get_char(msa, seq_index1, col_index);
        if c1 == '-' || c1 == '.' {
            continue;
        }
        let c2 = msa_get_char(msa, seq_index2, col_index);
        if c2 == '-' || c2 == '.' {
            continue;
        }
        pos_count += 1;
        if c1 == c2 {
            same_count += 1;
        }
    }
    if pos_count > 0 {
        (100.0 * (same_count as f64) / (pos_count as f64), pos_count)
    } else {
        (0.0, pos_count)
    }
}

/// Return the fraction of non-gap characters in the given column.
#[track_caller]
pub fn msa_get_occ(msa: &MultiSequence, col_index: uint) -> f64 {
    let mut gap_count = 0;
    for seq_index in 0..msa.seqs.len() {
        if msa_is_gap(msa, seq_index as uint, col_index) {
            gap_count += 1;
        }
    }
    let seq_count = msa.seqs.len() as uint;
    ((seq_count - gap_count) as f64) / (seq_count as f64)
}

/// Serialize the MSA to FASTA text.
#[track_caller]
pub fn msa_to_file(msa: &MultiSequence) -> String {
    msa_to_fasta_file_l112(msa)
}

/// Return one row as an ungapped uppercase string.
#[track_caller]
pub fn msa_get_ungapped_seq_str(msa: &MultiSequence, seq_index: uint) -> String {
    let mut seq_str = String::new();
    assert!((seq_index as usize) < msa.seqs.len());
    for i in 0..multi_sequence_get_col_count(msa) {
        let c = msa.seqs[seq_index as usize].char_vec[i as usize];
        if c != '-' && c != '.' {
            seq_str.push(c.to_ascii_uppercase());
        }
    }
    seq_str
}

/// Return true if any row has a gap at the given column.
#[track_caller]
pub fn msa_column_has_gap(msa: &MultiSequence, col_index: uint) -> bool {
    let seq_count = msa.seqs.len();
    for seq_index in 0..seq_count {
        if msa_is_gap(msa, seq_index as uint, col_index) {
            return true;
        }
    }
    false
}

/// Set the id-count once; later calls may only shrink, never grow.
#[track_caller]
pub fn msa_set_id_count(msa: &mut MultiSequence, id_count: uint) {
    if msa.id_count > 0 {
        if id_count > msa.id_count {
            panic!("MSA::SetIdCount: cannot increase count");
        }
        return;
    }
    msa.id_count = id_count;
}

/// Assign an external id to a sequence and maintain both id↔index lookup tables.
#[track_caller]
pub fn msa_set_seq_id(msa: &mut MultiSequence, seq_index: uint, id: uint) {
    assert!((seq_index as usize) < msa.seqs.len());
    assert!(id == uint::MAX || id < msa.id_count);
    if msa.seq_index_to_id.is_empty() {
        if msa.id_count == 0 {
            panic!("MSA::SetSeqId, SetIdCount has not been called");
        }
        msa.id_to_seq_index = vec![uint::MAX; msa.id_count as usize];
        msa.seq_index_to_id = vec![uint::MAX; msa.seqs.len()];
    }
    msa.seq_index_to_id[seq_index as usize] = id;
    if id != uint::MAX {
        msa.id_to_seq_index[id as usize] = seq_index;
    }
}

/// Return the row index for a given external id; panic if unknown.
#[track_caller]
pub fn msa_get_seq_index_l733(msa: &MultiSequence, id: uint) -> uint {
    assert!(id < msa.id_count);
    assert!(!msa.id_to_seq_index.is_empty());
    let seq_index = msa.id_to_seq_index[id as usize];
    assert!((seq_index as usize) < msa.seqs.len());
    seq_index
}

/// Linear scan to find the row index for an id, returning `None` if absent.
#[track_caller]
pub fn msa_get_seq_index_l742(msa: &MultiSequence, id: uint) -> Option<uint> {
    for seq_index in 0..msa.seqs.len() {
        if !msa.seq_index_to_id.is_empty() && id == msa.seq_index_to_id[seq_index] {
            return Some(seq_index as uint);
        }
    }
    None
}

/// Return the external id for one row, or `uint::MAX` if ids aren't set.
#[track_caller]
pub fn msa_get_seq_id(msa: &MultiSequence, seq_index: uint) -> uint {
    if msa.seq_index_to_id.is_empty() {
        return uint::MAX;
    }
    assert!((seq_index as usize) < msa.seqs.len());
    let id = msa.seq_index_to_id[seq_index as usize];
    assert!(id == uint::MAX || id < msa.id_count);
    id
}

/// Build a new MSA containing only the rows with the given external ids.
#[track_caller]
pub fn msa_subset_by_ids(msa_in: &MultiSequence, ids: &[uint]) -> MultiSequence {
    let col_count = multi_sequence_get_col_count(msa_in);
    let mut msa_out = MultiSequence::default();
    msa_set_id_count(&mut msa_out, msa_in.id_count);
    msa_set_size(&mut msa_out, ids.len() as uint, col_count);
    for id in ids {
        let seq_index_out = msa_out
            .seqs
            .iter()
            .position(|seq| seq.label.is_empty())
            .unwrap();
        let seq_index_in = msa_get_seq_index_l733(msa_in, *id);
        let mut seq = sequence_clone(&msa_in.seqs[seq_index_in as usize]);
        assert_eq!(seq.char_vec.len(), col_count as usize);
        seq.label = msa_get_seq_name(msa_in, seq_index_in);
        msa_out.seqs[seq_index_out] = seq;
        msa_set_seq_id(&mut msa_out, seq_index_out as uint, *id);
    }
    msa_out
}

/// Append a sequence row; panic if its length doesn't match existing columns.
#[track_caller]
pub fn msa_append_seq(msa: &mut MultiSequence, seq_chars: &[char], label: &str) {
    if !msa.seqs.is_empty() && seq_chars.len() != multi_sequence_get_col_count(msa) as usize {
        panic!("Internal error MSA::AppendSeq");
    }
    msa.seqs.push(Sequence {
        label: label.to_string(),
        char_vec: seq_chars.to_vec(),
    });
    msa.owners.push(true);
}

/// Grow row count up to `seq_count` with placeholder rows; column count must match.
#[track_caller]
pub fn msa_expand_cache(msa: &mut MultiSequence, seq_count: uint, col_count: uint) {
    if !msa.id_to_seq_index.is_empty()
        || !msa.seq_index_to_id.is_empty()
        || (seq_count as usize) < msa.seqs.len()
    {
        panic!("Internal error MSA::ExpandCache");
    }
    if !msa.seqs.is_empty() && col_count != multi_sequence_get_col_count(msa) {
        panic!("Internal error MSA::ExpandCache, ColCount changed");
    }
    while (msa.seqs.len() as uint) < seq_count {
        msa.seqs.push(Sequence {
            label: String::new(),
            char_vec: vec!['?'; col_count as usize],
        });
        msa.owners.push(true);
    }
}

/// Build the ungapped position → column index map for one row.
#[track_caller]
pub fn msa_get_pos_to_col(msa: &MultiSequence, seq_index: uint) -> Vec<uint> {
    let mut pos_to_col = Vec::new();
    let col_count = multi_sequence_get_col_count(msa);
    pos_to_col.reserve(col_count as usize);
    let seq = &msa.seqs[seq_index as usize];
    for col in 0..col_count {
        let c = seq.char_vec[col as usize];
        if c != '-' && c != '.' {
            pos_to_col.push(col);
        }
    }
    pos_to_col
}

/// Build the column → 1-based ungapped position map; gaps yield the negated previous position.
#[track_caller]
pub fn msa_get_col_to_pos1(msa: &MultiSequence, seq_index: uint) -> Vec<i32> {
    let mut col_to_pos = Vec::new();
    let col_count = multi_sequence_get_col_count(msa);
    col_to_pos.reserve(col_count as usize);
    let seq = &msa.seqs[seq_index as usize];
    let mut pos = 0_i32;
    for col in 0..col_count {
        let c = seq.char_vec[col as usize];
        if c == '-' || c == '.' {
            col_to_pos.push(if pos == 0 { 0 } else { -pos });
        } else {
            pos += 1;
            col_to_pos.push(pos);
        }
    }
    col_to_pos
}

/// Build the column → 0-based ungapped position map; gaps yield `uint::MAX`.
#[track_caller]
pub fn msa_get_col_to_pos(msa: &MultiSequence, seq_index: uint) -> Vec<uint> {
    let mut col_to_pos = Vec::new();
    let col_count = multi_sequence_get_col_count(msa);
    col_to_pos.reserve(col_count as usize);
    let seq = &msa.seqs[seq_index as usize];
    let mut pos = 0;
    for col in 0..col_count {
        let c = seq.char_vec[col as usize];
        if c == '-' || c == '.' {
            col_to_pos.push(uint::MAX);
        } else {
            col_to_pos.push(pos);
            pos += 1;
        }
    }
    col_to_pos
}

/// Return true if a column qualifies as an upper-case (match) column under a gap-fraction cap.
#[track_caller]
pub fn msa_col_is_upper(msa: &MultiSequence, col_index: uint, max_gap_fract: f64) -> bool {
    let seq_count = msa.seqs.len();
    let mut upper_count = 0;
    let mut lower_count = 0;
    let mut gap_count = 0;
    for seq_index in 0..seq_count {
        let c = msa.seqs[seq_index].char_vec[col_index as usize];
        if c == '-' || c == '.' {
            gap_count += 1;
            continue;
        }
        if !c.is_ascii_alphabetic() {
            continue;
        }
        if c.is_ascii_uppercase() {
            upper_count += 1;
        } else {
            lower_count += 1;
        }
    }

    if upper_count == 0 && lower_count == 0 {
        return false;
    }
    if upper_count > 0 && lower_count > 0 {
        panic!("Column {col_index} has mixed case letters");
    }
    if seq_count > 0 && (gap_count as f64) / (seq_count as f64) > max_gap_fract {
        return false;
    }
    if upper_count == 0 {
        return false;
    }
    true
}

/// Return true if a column has at least two upper-case letters and no mixed case.
#[track_caller]
pub fn msa_col_is_aligned(msa: &MultiSequence, col_index: uint) -> bool {
    let seq_count = msa.seqs.len();
    let mut upper_count = 0;
    let mut lower_count = 0;
    for seq_index in 0..seq_count {
        let c = msa.seqs[seq_index].char_vec[col_index as usize];
        if c == '-' || c == '.' {
            continue;
        }
        if !c.is_ascii_alphabetic() {
            continue;
        }
        if c.is_ascii_uppercase() {
            upper_count += 1;
        } else {
            lower_count += 1;
        }
    }

    if upper_count == 0 && lower_count == 0 {
        return false;
    }
    if upper_count > 0 && lower_count > 0 {
        panic!("Column {col_index} has mixed case letters");
    }
    if upper_count < 2 {
        return false;
    }
    true
}

/// Return a copy of the MSA with every all-gap column removed.
#[track_caller]
pub fn msa_delete_all_gap_cols(msa_in: &MultiSequence) -> MultiSequence {
    let col_count = multi_sequence_get_col_count(msa_in);
    let seq_count = msa_in.seqs.len();
    let mut keep_cols = Vec::new();
    for col in 0..col_count {
        let mut gap_count = 0usize;
        for seq_index in 0..seq_count {
            let c = msa_in.seqs[seq_index].char_vec[col as usize];
            if c == '-' || c == '.' {
                gap_count += 1;
            }
        }
        if gap_count != seq_count {
            keep_cols.push(col);
        }
    }

    let mut msa_out = MultiSequence::default();
    for seq in &msa_in.seqs {
        let mut out_seq = Sequence {
            label: seq.label.clone(),
            char_vec: Vec::with_capacity(keep_cols.len()),
        };
        for col in &keep_cols {
            out_seq.char_vec.push(seq.char_vec[*col as usize]);
        }
        msa_out.seqs.push(out_seq);
        msa_out.owners.push(true);
    }
    msa_out
}
