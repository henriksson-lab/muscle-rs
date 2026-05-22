// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct TransAln {
    pub msa: Option<MultiSequence>,
    pub fresh_seqs: Option<MultiSequence>,
    pub fresh_index_to_msa_index: Vec<uint>,
    pub pw_paths: Vec<String>,
    pub msa_col_count: uint,
    pub extended_msa_col_count: uint,
    pub ungapped_msa_seqs: Vec<Sequence>,
    pub msa_paths: Vec<String>,
    pub t_paths1: Vec<String>,
    pub t_paths2: Vec<String>,
    pub msa_col_to_max_inserts: Vec<uint>,
    pub m_path: String,
    pub extended_msa: Option<MultiSequence>,
} // original: TransAln (muscle/src/transaln.h)

/// Dumps the full state of a `TransAln` (paths, inserts, alignments) for
/// diagnostics.
#[track_caller]
pub fn trans_aln_log_me(ta: &TransAln) -> String {
    let mut out = String::new();
    out.push('\n');
    out.push_str("Pair-wise alignments:\n");
    let fresh_count = trans_aln_get_fresh_count(ta);
    for fresh_index in 0..fresh_count {
        let fresh_seq = trans_aln_get_fresh_seq(ta, fresh_index);
        let msa_index = trans_aln_get_msa_index(ta, fresh_index);
        let ungapped_msa_seq = trans_aln_get_ungapped_msa_seq(ta, msa_index);
        let pw_path = trans_aln_get_pw_path(ta, fresh_index);
        let mut fresh_pos = 0usize;
        let mut msa_pos = 0usize;
        let mut fresh_row = String::new();
        let mut msa_row = String::new();
        for c in pw_path.bytes() {
            match c {
                b'B' => {
                    fresh_row.push(fresh_seq.char_vec[fresh_pos]);
                    msa_row.push(ungapped_msa_seq.char_vec[msa_pos]);
                    fresh_pos += 1;
                    msa_pos += 1;
                }
                b'X' => {
                    fresh_row.push(fresh_seq.char_vec[fresh_pos]);
                    msa_row.push('-');
                    fresh_pos += 1;
                }
                b'Y' => {
                    fresh_row.push('-');
                    msa_row.push(ungapped_msa_seq.char_vec[msa_pos]);
                    msa_pos += 1;
                }
                _ => panic!("Invalid PWPath char {}", c as char),
            }
        }
        out.push_str(&format!(
            "{pw_path}\n{}  >{}\n{}  >{}\n",
            fresh_row, fresh_seq.label, msa_row, ungapped_msa_seq.label
        ));
    }

    out.push('\n');
    out.push_str("MSAPaths:\n");
    let msa_count = trans_aln_get_msa_count(ta);
    for msa_index in 0..msa_count {
        let msa_path = trans_aln_get_msa_path(ta, msa_index);
        let msa_label = trans_aln_get_msa_label(ta, msa_index);
        out.push_str(msa_path);
        out.push_str(&format!("  >{msa_label}\n"));
    }

    out.push('\n');
    out.push_str("MaxInserts:\n");
    let mut m = 0;
    for col in 0..ta.msa_col_to_max_inserts.len() {
        let n = ta.msa_col_to_max_inserts[col];
        if n > 0 {
            out.push_str(&format!(" [{col}]={n}"));
            m += 1;
        }
    }
    out.push_str(&format!(" ({m})\n"));
    out.push_str(&format!(
        "ExtendedColCount = {}\n\n",
        ta.extended_msa_col_count
    ));

    out.push('\n');
    out.push_str("TPaths1:\n");
    for i in 0..ta.t_paths1.len() {
        let path1 = &ta.t_paths1[i];
        let fresh_label = trans_aln_get_fresh_label(ta, i as uint);
        out.push_str(path1);
        out.push_str(&format!("  >{fresh_label}\n"));
    }

    for i in 0..ta.t_paths1.len() {
        out.push_str(&trans_aln_log_t_path1_aln(ta, i as uint));
    }

    out.push('\n');
    out.push_str("TPaths2:\n");
    for i in 0..ta.t_paths2.len() {
        let path2 = &ta.t_paths2[i];
        let fresh_label = trans_aln_get_fresh_label(ta, i as uint);
        out.push_str(path2);
        out.push_str(&format!("  >{fresh_label}\n"));
    }
    out.push('\n');
    out.push_str("MPath\n");
    out.push_str(&format!("{}\n", ta.m_path));

    for i in 0..ta.t_paths2.len() {
        out.push_str(&trans_aln_log_t_path2_aln(ta, i as uint, true));
    }

    out.push('\n');
    for i in 0..msa_count {
        out.push_str(&trans_aln_log_m_path_aln(ta, i, false));
    }
    for i in 0..ta.t_paths2.len() {
        out.push_str(&trans_aln_log_t_path2_aln(ta, i as uint, false));
    }
    out
}

/// Returns the number of fresh (newly-aligned) sequences.
#[track_caller]
pub fn trans_aln_get_fresh_count(ta: &TransAln) -> uint {
    ta.fresh_seqs
        .as_ref()
        .expect("TransAln::GetFreshCount null fresh seqs")
        .seqs
        .len() as uint
}

/// Returns the number of sequences in the reference MSA.
#[track_caller]
pub fn trans_aln_get_msa_count(ta: &TransAln) -> uint {
    ta.msa
        .as_ref()
        .expect("TransAln::GetMSACount null MSA")
        .seqs
        .len() as uint
}

/// Returns the MSA seed-sequence index that the given fresh sequence was
/// aligned against.
#[track_caller]
pub fn trans_aln_get_msa_index(ta: &TransAln, fresh_index: uint) -> uint {
    assert!((fresh_index as usize) < ta.fresh_index_to_msa_index.len());
    ta.fresh_index_to_msa_index[fresh_index as usize]
}

/// Returns the pairwise alignment path for the given fresh sequence.
#[track_caller]
pub fn trans_aln_get_pw_path(ta: &TransAln, fresh_index: uint) -> &str {
    assert!((fresh_index as usize) < ta.pw_paths.len());
    &ta.pw_paths[fresh_index as usize]
}

/// Returns the MSA-gap path for the given MSA-sequence index.
#[track_caller]
pub fn trans_aln_get_msa_path(ta: &TransAln, fresh_index: uint) -> &str {
    assert!((fresh_index as usize) < ta.msa_paths.len());
    &ta.msa_paths[fresh_index as usize]
}

/// Returns the unextended T-path (PW joined with MSA gaps).
#[track_caller]
pub fn trans_aln_get_t_path1(ta: &TransAln, fresh_index: uint) -> &str {
    assert!((fresh_index as usize) < ta.t_paths1.len());
    &ta.t_paths1[fresh_index as usize]
}

/// Returns the extended T-path with max-inserts gaps applied.
#[track_caller]
pub fn trans_aln_get_t_path2(ta: &TransAln, fresh_index: uint) -> &str {
    assert!((fresh_index as usize) < ta.t_paths2.len());
    &ta.t_paths2[fresh_index as usize]
}

/// Returns the label of an MSA sequence.
#[track_caller]
pub fn trans_aln_get_msa_label(ta: &TransAln, msa_index: uint) -> &str {
    &trans_aln_get_msa_seq(ta, msa_index).label
}

/// Returns the label of a fresh sequence.
#[track_caller]
pub fn trans_aln_get_fresh_label(ta: &TransAln, fresh_index: uint) -> &str {
    &trans_aln_get_fresh_seq(ta, fresh_index).label
}

/// Returns the gapped MSA sequence by index.
#[track_caller]
pub fn trans_aln_get_msa_seq(ta: &TransAln, msa_index: uint) -> &Sequence {
    let msa = ta.msa.as_ref().expect("TransAln::GetMSASeq null MSA");
    assert!((msa_index as usize) < msa.seqs.len());
    &msa.seqs[msa_index as usize]
}

/// Returns the cached ungapped MSA sequence by index.
#[track_caller]
pub fn trans_aln_get_ungapped_msa_seq(ta: &TransAln, msa_index: uint) -> &Sequence {
    assert!((msa_index as usize) < ta.ungapped_msa_seqs.len());
    &ta.ungapped_msa_seqs[msa_index as usize]
}

/// Returns the fresh (unaligned input) sequence by index.
#[track_caller]
pub fn trans_aln_get_fresh_seq(ta: &TransAln, fresh_index: uint) -> &Sequence {
    let fresh_seqs = ta
        .fresh_seqs
        .as_ref()
        .expect("TransAln::GetFreshSeq null fresh seqs");
    assert!((fresh_index as usize) < fresh_seqs.seqs.len());
    &fresh_seqs.seqs[fresh_index as usize]
}

/// Returns the length of the ungapped MSA sequence at `msa_index`.
#[track_caller]
pub fn trans_aln_get_ungapped_msa_seq_length(ta: &TransAln, msa_index: uint) -> uint {
    trans_aln_get_ungapped_msa_seq(ta, msa_index).char_vec.len() as uint
}

/// Returns the length of a fresh sequence.
#[track_caller]
pub fn trans_aln_get_fresh_seq_length(ta: &TransAln, fresh_index: uint) -> uint {
    trans_aln_get_fresh_seq(ta, fresh_index).char_vec.len() as uint
}

/// Builds path1: the PW path projected onto MSA columns, inserting `g`
/// where the MSA has a gap.
#[track_caller]
pub fn trans_aln_make_t_path1(ta: &TransAln, fresh_index: uint) -> String {
    let msa_col_count = ta.msa_col_count;
    let msa_index = trans_aln_get_msa_index(ta, fresh_index);
    let pw_path = trans_aln_get_pw_path(ta, fresh_index).as_bytes();
    let msa_path = trans_aln_get_msa_path(ta, msa_index).as_bytes();
    let fresh_l = trans_aln_get_fresh_seq_length(ta, fresh_index);
    let ul = trans_aln_get_ungapped_msa_seq_length(ta, msa_index);

    let mut path1 = String::new();
    let mut nf = 0;
    let mut ng = 0;
    let mut ni = 0;
    let mut n_gap_msa = 0;
    let mut msa_col = 0usize;
    for c in pw_path {
        if *c == b'B' || *c == b'Y' {
            while msa_path[msa_col] == b'G' {
                msa_col += 1;
                path1.push('g');
                n_gap_msa += 1;
            }
        }

        match *c {
            b'B' => {
                path1.push('F');
                nf += 1;
                msa_col += 1;
            }
            b'X' => {
                path1.push('I');
                ni += 1;
            }
            b'Y' => {
                path1.push('G');
                ng += 1;
                msa_col += 1;
            }
            _ => panic!("invalid PW path char {}", *c as char),
        }
    }

    while msa_col < msa_col_count as usize {
        assert_eq!(msa_path[msa_col], b'G');
        msa_col += 1;
        path1.push('g');
        n_gap_msa += 1;
    }

    assert_eq!(nf + ng + n_gap_msa, msa_col_count);
    assert_eq!(nf + ni, fresh_l);
    assert_eq!(nf + ng, ul);
    path1
}

/// Builds the M/G path for an MSA sequence (M for residue, G for gap).
#[track_caller]
pub fn trans_aln_make_msa_path(ta: &TransAln, msa_index: uint) -> String {
    let msa_seq = trans_aln_get_msa_seq(ta, msa_index);
    assert_eq!(msa_seq.char_vec.len() as uint, ta.msa_col_count);
    let mut msa_path = String::new();
    for col in 0..ta.msa_col_count {
        let c = msa_seq.char_vec[col as usize];
        if c == '-' {
            msa_path.push('G');
        } else {
            msa_path.push('M');
        }
    }
    msa_path
}

/// Builds the master path describing extended MSA columns (M for
/// original, i for inserted).
#[track_caller]
pub fn trans_aln_make_m_path(ta: &TransAln) -> String {
    assert_eq!(
        ta.msa_col_to_max_inserts.len(),
        ta.msa_col_count as usize + 1
    );
    let mut m_path = String::new();
    for col in 0..=ta.msa_col_count {
        let ins = ta.msa_col_to_max_inserts[col as usize];
        for _ in 0..ins {
            m_path.push('i');
        }
        if col < ta.msa_col_count {
            m_path.push('M');
        }
    }
    m_path
}

/// Initialises the TransAln from an MSA, fresh sequences and pairwise
/// alignments; builds all derived path structures.
#[track_caller]
pub fn trans_aln_init(
    ta: &mut TransAln,
    msa: &MultiSequence,
    fresh_seqs: &MultiSequence,
    fresh_index_to_msa_index: &[uint],
    pw_paths: &[String],
) {
    ta.msa_paths.clear();
    ta.t_paths1.clear();
    ta.t_paths2.clear();
    ta.ungapped_msa_seqs.clear();

    ta.msa = Some(msa.clone());
    ta.fresh_seqs = Some(fresh_seqs.clone());
    ta.fresh_index_to_msa_index = fresh_index_to_msa_index.to_vec();
    ta.pw_paths = pw_paths.to_vec();
    ta.msa_col_count = multi_sequence_get_col_count(msa);

    let msa_seq_count = msa.seqs.len() as uint;
    for msa_index in 0..msa_seq_count {
        let msa_path = trans_aln_make_msa_path(ta, msa_index);
        ta.msa_paths.push(msa_path);

        let seq = &msa.seqs[msa_index as usize];
        let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
        let mut ungapped_seq = Sequence::default();
        sequence_from_string(&mut ungapped_seq, &seq.label, &ungapped);
        ta.ungapped_msa_seqs.push(ungapped_seq);
    }

    let fresh_seq_count = fresh_seqs.seqs.len() as uint;
    for fresh_index in 0..fresh_seq_count {
        let path1 = trans_aln_make_t_path1(ta, fresh_index);
        ta.t_paths1.push(path1);
    }

    trans_aln_set_max_inserts(ta);

    for fresh_index in 0..fresh_seq_count {
        let path2 = trans_aln_make_t_path2(ta, fresh_index);
        ta.t_paths2.push(path2);
    }
    ta.m_path = trans_aln_make_m_path(ta);
}

/// Computes the per-column maximum insert count and the resulting
/// extended-MSA column count.
#[track_caller]
pub fn trans_aln_set_max_inserts(ta: &mut TransAln) {
    let fresh_count = trans_aln_get_fresh_count(ta);
    assert_eq!(ta.t_paths1.len(), fresh_count as usize);
    ta.msa_col_to_max_inserts.clear();
    ta.msa_col_to_max_inserts
        .resize(ta.msa_col_count as usize + 1, 0);
    for fresh_index in 0..fresh_count {
        let msa_col_to_inserts = trans_aln_make_msa_col_to_inserts(ta, fresh_index);
        assert_eq!(msa_col_to_inserts.len(), ta.msa_col_count as usize + 1);
        for msa_col in 0..=ta.msa_col_count {
            let ins = msa_col_to_inserts[msa_col as usize];
            ta.msa_col_to_max_inserts[msa_col as usize] =
                std::cmp::max(ins, ta.msa_col_to_max_inserts[msa_col as usize]);
        }
    }

    ta.extended_msa_col_count = 0;
    for msa_col in 0..=ta.msa_col_count {
        let ins = ta.msa_col_to_max_inserts[msa_col as usize];
        ta.extended_msa_col_count += ins;
        if msa_col < ta.msa_col_count {
            ta.extended_msa_col_count += 1;
        }
    }
}

/// Counts how many `I` insertions occur before each MSA column for a
/// given fresh sequence.
#[track_caller]
pub fn trans_aln_make_msa_col_to_inserts(ta: &TransAln, fresh_index: uint) -> Vec<uint> {
    let t_path1 = trans_aln_get_t_path1(ta, fresh_index).as_bytes();
    let mut msa_col = 0usize;
    let mut msa_col_to_inserts = vec![0; ta.msa_col_count as usize + 1];

    for c in t_path1 {
        match *c {
            b'F' | b'G' | b'g' => {
                msa_col += 1;
            }
            b'I' => {
                assert!(msa_col <= ta.msa_col_count as usize);
                msa_col_to_inserts[msa_col] += 1;
            }
            _ => panic!("invalid TPath1 char {}", *c as char),
        }
    }

    assert_eq!(msa_col, ta.msa_col_count as usize);
    msa_col_to_inserts
}

/// Builds path2: path1 padded with `i` insert columns to match the
/// extended MSA layout.
#[track_caller]
pub fn trans_aln_make_t_path2(ta: &TransAln, fresh_index: uint) -> String {
    assert_eq!(
        ta.msa_col_to_max_inserts.len(),
        ta.msa_col_count as usize + 1
    );
    let t_path1 = trans_aln_get_t_path1(ta, fresh_index).as_bytes();
    let msa_col_to_inserts = trans_aln_make_msa_col_to_inserts(ta, fresh_index);

    let mut path2 = String::new();
    let mut msa_col = 0usize;
    for c in t_path1 {
        path2.push(*c as char);
        if *c != b'I' {
            assert!(msa_col < ta.msa_col_count as usize);
            let insert_count = msa_col_to_inserts[msa_col];
            let max_insert_count = ta.msa_col_to_max_inserts[msa_col];
            assert!(insert_count <= max_insert_count);
            for _ in insert_count..max_insert_count {
                path2.push('i');
            }
        }

        match *c {
            b'F' | b'G' | b'g' => {
                msa_col += 1;
            }
            b'I' => {}
            _ => panic!("invalid TPath1 char {}", *c as char),
        }
    }
    assert_eq!(msa_col, ta.msa_col_count as usize);
    let insert_count = msa_col_to_inserts[ta.msa_col_count as usize];
    let max_insert_count = ta.msa_col_to_max_inserts[ta.msa_col_count as usize];
    assert!(insert_count <= max_insert_count);
    for _ in insert_count..max_insert_count {
        path2.push('i');
    }

    if path2.len() != ta.extended_msa_col_count as usize {
        panic!(
            "|Path2|={}, m_ExtendedMSAColCount={}",
            path2.len(),
            ta.extended_msa_col_count
        );
    }
    path2
}

/// Renders a fresh-vs-MSA alignment row using path1 for inspection.
#[track_caller]
pub fn trans_aln_log_t_path1_aln(ta: &TransAln, fresh_index: uint) -> String {
    let t_path1 = trans_aln_get_t_path1(ta, fresh_index);
    let msa_index = trans_aln_get_msa_index(ta, fresh_index);
    let f = trans_aln_get_fresh_seq(ta, fresh_index);
    let u = trans_aln_get_ungapped_msa_seq(ta, msa_index);
    let fl = f.char_vec.len();
    let ul = u.char_vec.len();
    let f_label = trans_aln_get_fresh_label(ta, fresh_index);
    let u_label = trans_aln_get_msa_label(ta, msa_index);
    let mut f_pos = 0usize;
    let mut u_pos = 0usize;
    let mut f_row = String::new();
    let mut u_row = String::new();
    for c in t_path1.bytes() {
        match c {
            b'F' => {
                f_row.push(f.char_vec[f_pos]);
                u_row.push(u.char_vec[u_pos]);
                f_pos += 1;
                u_pos += 1;
            }
            b'G' => {
                f_row.push('-');
                u_row.push(u.char_vec[u_pos]);
                u_pos += 1;
            }
            b'I' => {
                f_row.push(f.char_vec[f_pos]);
                u_row.push('.');
                f_pos += 1;
            }
            b'g' => {
                f_row.push('.');
                u_row.push('.');
            }
            _ => {}
        }
    }
    assert_eq!(f_pos, fl);
    assert_eq!(u_pos, ul);
    format!("\n{t_path1}\n{f_row}  >{f_label}\n{u_row}  >{u_label}\n")
}

/// Renders a fresh-vs-MSA alignment row using path2 for inspection.
#[track_caller]
pub fn trans_aln_log_t_path2_aln(ta: &TransAln, fresh_index: uint, with_path: bool) -> String {
    let t_path2 = trans_aln_get_t_path2(ta, fresh_index);
    let msa_index = trans_aln_get_msa_index(ta, fresh_index);
    let f = trans_aln_get_fresh_seq(ta, fresh_index);
    let u = trans_aln_get_ungapped_msa_seq(ta, msa_index);
    let fl = f.char_vec.len();
    let ul = u.char_vec.len();
    let f_label = trans_aln_get_fresh_label(ta, fresh_index);
    let u_label = trans_aln_get_msa_label(ta, msa_index);
    let mut f_pos = 0usize;
    let mut u_pos = 0usize;
    let mut f_row = String::new();
    let mut u_row = String::new();
    for c in t_path2.bytes() {
        match c {
            b'F' => {
                f_row.push(f.char_vec[f_pos]);
                u_row.push(u.char_vec[u_pos]);
                f_pos += 1;
                u_pos += 1;
            }
            b'G' => {
                f_row.push('-');
                u_row.push(u.char_vec[u_pos]);
                u_pos += 1;
            }
            b'I' => {
                f_row.push(f.char_vec[f_pos]);
                u_row.push('.');
                f_pos += 1;
            }
            b'g' | b'i' => {
                f_row.push('.');
                u_row.push('.');
            }
            _ => {}
        }
    }
    assert_eq!(f_pos, fl);
    assert_eq!(u_pos, ul);
    let mut out = String::new();
    if with_path {
        out.push_str(&format!("\n{t_path2}\n"));
    }
    out.push_str(&format!("{f_row}  [F] >{f_label}\n"));
    out.push_str(&format!("{u_row}  [U] >{u_label}\n"));
    out
}

/// Renders an MSA sequence under the master path for inspection.
#[track_caller]
pub fn trans_aln_log_m_path_aln(ta: &TransAln, msa_index: uint, with_path: bool) -> String {
    let m = trans_aln_get_msa_seq(ta, msa_index);
    let m_label = trans_aln_get_msa_label(ta, msa_index);
    let mut msa_col = 0usize;
    let mut m_row = String::new();
    for c in ta.m_path.bytes() {
        match c {
            b'M' => {
                m_row.push(m.char_vec[msa_col]);
                msa_col += 1;
            }
            b'i' => {
                m_row.push('.');
            }
            _ => {}
        }
    }
    assert_eq!(msa_col, ta.msa_col_count as usize);
    let mut out = String::new();
    if with_path {
        out.push_str(&format!("\n{}\n", ta.m_path));
    }
    out.push_str(&format!("{m_row}  [M] >{m_label}\n"));
    out
}

/// Produces the gapped fresh sequence row aligned to the extended MSA.
#[track_caller]
pub fn trans_aln_extend_fresh_seq(ta: &TransAln, fresh_index: uint) -> Sequence {
    let f = trans_aln_get_fresh_seq(ta, fresh_index);
    let t_path2 = trans_aln_get_t_path2(ta, fresh_index);
    let col_count = ta.m_path.len();
    let mut fx = Sequence {
        label: f.label.clone(),
        char_vec: Vec::new(),
    };
    let mut f_pos = 0usize;
    for c in t_path2.bytes().take(col_count) {
        match c {
            b'F' | b'I' => {
                fx.char_vec.push(f.char_vec[f_pos]);
                f_pos += 1;
            }
            b'G' | b'g' | b'i' => {
                fx.char_vec.push('-');
            }
            _ => panic!("Invalid char '{}' in TPath2", c as char),
        }
    }
    assert_eq!(fx.char_vec.len(), ta.extended_msa_col_count as usize);
    fx
}

/// Produces the gapped MSA sequence row aligned to the extended MSA.
#[track_caller]
pub fn trans_aln_extend_msa_seq(ta: &TransAln, msa_index: uint) -> Sequence {
    let m = trans_aln_get_msa_seq(ta, msa_index);
    let mut mx = Sequence {
        label: m.label.clone(),
        char_vec: Vec::new(),
    };
    let mut msa_col = 0usize;
    for c in ta.m_path.bytes() {
        match c {
            b'M' => {
                mx.char_vec.push(m.char_vec[msa_col]);
                msa_col += 1;
            }
            b'i' => {
                mx.char_vec.push('-');
            }
            _ => panic!("Invalid char '{}' in MPath", c as char),
        }
    }
    assert_eq!(mx.char_vec.len(), ta.extended_msa_col_count as usize);
    mx
}

/// Assembles the extended MSA combining gapped reference and fresh
/// sequences.
#[track_caller]
pub fn trans_aln_make_extended_msa(ta: &mut TransAln) {
    let msa_count = trans_aln_get_msa_count(ta);
    let fresh_count = trans_aln_get_fresh_count(ta);
    let mut extended_msa = MultiSequence::default();

    for i in 0..msa_count {
        let s = trans_aln_extend_msa_seq(ta, i);
        extended_msa.seqs.push(s);
        extended_msa.owners.push(true);
    }

    for i in 0..fresh_count {
        let s = trans_aln_extend_fresh_seq(ta, i);
        extended_msa.seqs.push(s);
        extended_msa.owners.push(true);
    }
    ta.extended_msa = Some(extended_msa);
}

/// Entry point for the `transaln` command: builds pairwise alignments
/// between input and reference sequences and writes the extended MSA.
#[track_caller]
pub fn cmd_transaln<FAlignPairFlat>(
    input_file_name: &str,
    ref_file_name: &str,
    output_file_name: &str,
    mut align_pair_flat: FAlignPairFlat,
) -> (TransAln, MultiSequence)
where
    FAlignPairFlat: FnMut(&str, &str, &mut String),
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, false);
    let input_seq_count = input_seqs.seqs.len() as uint;

    let mut ref_msa = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut ref_msa, ref_file_name, false);
    let ref_seq_count = ref_msa.seqs.len() as uint;

    let mut global_input = MultiSequence::default();
    for seq in &input_seqs.seqs {
        let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
        let mut global_seq = Sequence::default();
        sequence_from_string(&mut global_seq, &seq.label, &ungapped);
        global_input.seqs.push(global_seq);
        global_input.owners.push(false);
    }
    for seq in &ref_msa.seqs {
        let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
        let mut global_seq = Sequence::default();
        sequence_from_string(&mut global_seq, &seq.label, &ungapped);
        global_input.seqs.push(global_seq);
        global_input.owners.push(false);
    }
    set_global_input_ms(&global_input);

    let mut pw_paths = Vec::new();
    let mut fresh_index_to_msa_index = Vec::new();
    for input_seq_index in 0..input_seq_count {
        let ref_seq_index = input_seq_index % ref_seq_count;
        fresh_index_to_msa_index.push(ref_seq_index);

        let input_label = input_seqs.seqs[input_seq_index as usize].label.clone();
        let ref_label = ref_msa.seqs[ref_seq_index as usize].label.clone();
        let mut pw_path = String::new();
        align_pair_flat(&input_label, &ref_label, &mut pw_path);
        pw_paths.push(pw_path);
    }

    let mut ta = TransAln::default();
    trans_aln_init(
        &mut ta,
        &ref_msa,
        &input_seqs,
        &fresh_index_to_msa_index,
        &pw_paths,
    );
    trans_aln_make_extended_msa(&mut ta);
    let extended_msa = ta
        .extended_msa
        .as_ref()
        .expect("TransAln extended MSA not built")
        .clone();
    msa_to_fasta_file_l103(&extended_msa, output_file_name);
    (ta, extended_msa)
}
