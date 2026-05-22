// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Ensemble {
    pub msas: Vec<MultiSequence>,
    pub msa_names: Vec<String>,
    pub labels0: Vec<String>,
    pub label_to_seq_index0: std::collections::BTreeMap<String, uint>,
    pub ungapped_seqs: Vec<String>,
    pub column_strings: Vec<String>,
    pub column_positions: Vec<Vec<i32>>,
    pub col_to_pos_vec: Vec<Vec<Vec<i32>>>,
    pub ix_to_msa_index: Vec<uint>,
    pub ix_to_col_index: Vec<uint>,
    pub msa_col_to_ix: Vec<Vec<uint>>,
    pub unique_ixs: Vec<uint>,
    pub unique_ix_to_ixs: Vec<Vec<uint>>,
    pub ix_to_unique_ix: Vec<uint>,
    pub unique_col_map: std::collections::BTreeMap<Vec<i32>, uint>,
} // original: Ensemble (muscle/src/ensemble.h)

/// Reads and returns the very first character of `file_name`, panicking on empty/error.
#[track_caller]
pub fn read_first_char(file_name: &str) -> char {
    let bytes =
        std::fs::read(file_name).unwrap_or_else(|e| panic!("OpenStdioFile({file_name}): {e}"));
    assert!(
        !bytes.is_empty(),
        "ReadStdioFile({file_name}) attempted 1 got 0"
    );
    bytes[0] as char
}

/// Recomputes all derived ensemble state (uppercase, labels, sorted MSAs, columns, ...).
#[track_caller]
pub fn ensemble_set_derived(e: &mut Ensemble) {
    ensemble_to_upper(e);
    ensemble_map_labels(e);
    ensemble_sort_ms_as(e);
    ensemble_set_ungapped_seqs(e);
    ensemble_set_col_to_pos_vec(e);
    ensemble_set_columns(e);
}

/// Captures the canonical (first MSA) labels and builds the label->index map.
#[track_caller]
pub fn ensemble_map_labels(e: &mut Ensemble) {
    assert!(!e.msas.is_empty());
    let m0 = &e.msas[0];
    let seq_count = m0.seqs.len();
    let (labels, label_to_seq_index) = msa_get_label_to_seq_index(m0);
    assert_eq!(labels.len(), seq_count);
    e.labels0 = labels;
    e.label_to_seq_index0 = label_to_seq_index;
}

/// Reorders the sequences of `m` so they match the canonical label order in `e`.
#[track_caller]
pub fn ensemble_sort_msa(e: &Ensemble, m: &mut MultiSequence) {
    assert!(!e.msas.is_empty());
    let seq_count = ensemble_get_seq_count(e);
    let (labels2, _label_to_seq_index2) = msa_get_label_to_seq_index(m);

    let mut seqs_sorted: Vec<Option<Sequence>> = vec![None; seq_count as usize];
    for seq_index in 0..seq_count {
        let label = &labels2[seq_index as usize];
        let seq_index0 = *e
            .label_to_seq_index0
            .get(label)
            .unwrap_or_else(|| panic!("SortMSA, different labels ({label})"));
        assert!(seqs_sorted[seq_index0 as usize].is_none());
        let mut seq = m.seqs[seq_index as usize].clone();
        seq.label = e.labels0[seq_index0 as usize].clone();
        seqs_sorted[seq_index0 as usize] = Some(seq);
    }
    m.seqs = seqs_sorted.into_iter().map(Option::unwrap).collect();
    m.owners = vec![true; seq_count as usize];

    let (labels_check, label_to_seq_index_check) = msa_get_label_to_seq_index(m);
    assert_eq!(labels_check, e.labels0);
    assert_eq!(label_to_seq_index_check, e.label_to_seq_index0);
}

/// Sorts every MSA in the ensemble to share the canonical sequence ordering.
#[track_caller]
pub fn ensemble_sort_ms_as(e: &mut Ensemble) {
    let msa_count = e.msas.len() as uint;
    let seq_count = ensemble_get_seq_count(e);

    for msa_index in 1..msa_count {
        let seq_count2 = e.msas[msa_index as usize].seqs.len() as uint;
        if seq_count2 != seq_count {
            panic!("Bad ensemble, different nr seqs");
        }

        let view = Ensemble {
            msas: vec![e.msas[0].clone()],
            msa_names: Vec::new(),
            labels0: e.labels0.clone(),
            label_to_seq_index0: e.label_to_seq_index0.clone(),
            ..Ensemble::default()
        };
        ensemble_sort_msa(&view, &mut e.msas[msa_index as usize]);
    }
}

/// Loads an Ensemble from an EFA-format file (one or more MSAs separated by `<name`).
#[track_caller]
pub fn ensemble_from_efa(e: &mut Ensemble, file_name: &str) {
    *e = Ensemble::default();

    let strings = read_strings_from_file(file_name);
    if strings.is_empty() {
        panic!("Empty EFA ({file_name})");
    }
    if !strings[0].starts_with('<') {
        panic!("Invalid EFA, must start with '<' ({file_name})");
    }

    let mut msa_strings = Vec::<String>::new();
    for s in strings {
        if s.starts_with('<') {
            if !msa_strings.is_empty() {
                let m = msa_from_strings(&msa_strings);
                e.msas.push(m);
                msa_strings.clear();
            }
            e.msa_names.push(s[1..].to_string());
        } else {
            msa_strings.push(s);
        }
    }

    let m = msa_from_strings(&msa_strings);
    e.msas.push(m);
    if e.msas.len() != e.msa_names.len() {
        panic!(
            "Invalid EFA, {} MSAs {} names ({file_name})",
            e.msas.len(),
            e.msa_names.len()
        );
    }
    ensemble_set_derived(e);
}

/// Writes the ensemble as an EFA-format file (a no-op when `file_name` is empty).
#[track_caller]
pub fn ensemble_to_efa(e: &Ensemble, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    let msa_count = e.msas.len();
    assert_eq!(e.msa_names.len(), msa_count);
    let mut out = String::new();
    for msa_index in 0..msa_count {
        out.push('<');
        out.push_str(&e.msa_names[msa_index]);
        out.push('\n');
        out.push_str(&msa_to_fasta_file_l112(&e.msas[msa_index]));
    }
    std::fs::write(file_name, out).expect("failed to write EFA");
}

/// Loads an Ensemble, auto-detecting EFA (`<`-prefixed) vs. a list of MSA file paths.
#[track_caller]
pub fn ensemble_from_file(e: &mut Ensemble, file_name: &str) {
    let c = read_first_char(file_name);
    if c == '<' {
        ensemble_from_efa(e, file_name);
    } else {
        ensemble_from_msa_paths(e, file_name, false, false);
    }
}

/// Loads each MSA listed in `file_name`, optionally renaming entries by basename / suffix.
#[track_caller]
pub fn ensemble_from_msa_paths(e: &mut Ensemble, file_name: &str, basename: bool, intsuffix: bool) {
    *e = Ensemble::default();
    e.msa_names = read_strings_from_file(file_name);
    let msa_count = e.msa_names.len() as uint;
    if msa_count == 0 {
        return;
    }

    for msa_index in 0..msa_count {
        let msa_file_name = &e.msa_names[msa_index as usize];
        let m = msa_from_fasta_file_l95(msa_file_name);
        e.msas.push(m);
    }

    if basename {
        for msa_index in 0..msa_count {
            let msa_file_name = &e.msa_names[msa_index as usize];
            e.msa_names[msa_index as usize] = base_name(msa_file_name).to_string();
        }
    }

    if intsuffix {
        for msa_index in 0..msa_count {
            e.msa_names[msa_index as usize].push_str(&format!(".{msa_index}"));
        }
    }

    ensemble_set_derived(e);
}

/// Returns the number of sequences shared by every MSA in the ensemble.
#[track_caller]
pub fn ensemble_get_seq_count(e: &Ensemble) -> uint {
    if e.msas.is_empty() {
        return 0;
    }
    e.msas[0].seqs.len() as uint
}

/// Upper-cases every character of every sequence across all MSAs.
#[track_caller]
pub fn ensemble_to_upper(e: &mut Ensemble) {
    let msa_count = e.msas.len();
    let seq_count = ensemble_get_seq_count(e) as usize;
    for msa_index in 0..msa_count {
        let col_count = multi_sequence_get_col_count(&e.msas[msa_index]) as usize;
        for seq_index in 0..seq_count {
            for i in 0..col_count {
                let c = e.msas[msa_index].seqs[seq_index].char_vec[i];
                e.msas[msa_index].seqs[seq_index].char_vec[i] = c.to_ascii_uppercase();
            }
        }
    }
}

/// Builds an MSA from a list of unique column indexes drawn from the ensemble.
#[track_caller]
pub fn ensemble_make_resampled_msa(e: &Ensemble, unique_ixs: &[uint]) -> MultiSequence {
    let col_count = unique_ixs.len() as uint;
    let seq_count = ensemble_get_seq_count(e);
    let mut m = MultiSequence::default();
    msa_set_size(&mut m, seq_count, col_count);

    assert_eq!(e.labels0.len(), seq_count as usize);
    for seq_index in 0..seq_count {
        let label = &e.labels0[seq_index as usize];
        m.seqs[seq_index as usize].label = label.clone();
    }

    for col_index in 0..col_count {
        let unique_ix = unique_ixs[col_index as usize];
        assert!((unique_ix as usize) < e.unique_ixs.len());
        let ix = e.unique_ixs[unique_ix as usize];
        assert!((ix as usize) < e.column_strings.len());
        let column_string = &e.column_strings[ix as usize];
        assert_eq!(column_string.len(), seq_count as usize);
        for seq_index in 0..seq_count {
            let c = column_string.as_bytes()[seq_index as usize] as char;
            m.seqs[seq_index as usize].char_vec[col_index as usize] = c;
        }
    }
    m
}

/// Returns the unique-column indexes whose confidence is high enough and gap fraction low enough.
#[track_caller]
pub fn ensemble_get_hi_qual_unique_ixs(
    e: &Ensemble,
    max_gap_fract: f64,
    min_conf: f64,
) -> Vec<uint> {
    let mut unique_ixs = Vec::<uint>::new();
    let n = e.unique_ixs.len() as uint;
    for unique_ix in 0..n {
        let ix = e.unique_ixs[unique_ix as usize];
        let conf = ensemble_get_conf(e, unique_ix);
        if conf < min_conf {
            continue;
        }
        let gap_fract = ensemble_get_gap_fract(e, ix);
        if gap_fract <= max_gap_fract {
            unique_ixs.push(unique_ix);
        }
    }
    unique_ixs
}

/// Returns the median per-MSA count of columns meeting both `max_gap_fract` and `min_conf`.
#[track_caller]
pub fn ensemble_get_median_hi_qual_col_count(
    e: &Ensemble,
    max_gap_fract: f64,
    min_conf: f64,
) -> uint {
    let mut col_counts = Vec::<uint>::new();
    let msa_count = e.msas.len() as uint;
    if msa_count == 0 {
        return 0;
    }

    for msa_index in 0..msa_count {
        let m = &e.msas[msa_index as usize];
        let seq_count = m.seqs.len() as uint;
        let col_count = multi_sequence_get_col_count(m);
        let mut non_gappy_col_count = 0_u32;
        for col_index in 0..col_count {
            let conf = ensemble_get_conf_msa_col(e, msa_index, col_index);
            if conf < min_conf {
                continue;
            }
            let gap_count = msa_get_gap_count(m, col_index);
            let gap_fract = f64::from(gap_count) / f64::from(seq_count);
            if gap_fract <= max_gap_fract {
                non_gappy_col_count += 1;
            }
        }
        col_counts.push(non_gappy_col_count);
    }
    col_counts.sort_unstable();
    col_counts[msa_count as usize / 2]
}

/// Caches the ungapped sequence strings and verifies every MSA shares them.
#[track_caller]
pub fn ensemble_set_ungapped_seqs(e: &mut Ensemble) {
    e.ungapped_seqs.clear();
    let seq_count = ensemble_get_seq_count(e);
    for seq_index in 0..seq_count {
        let ungapped_seq = msa_get_ungapped_seq_str(&e.msas[0], seq_index);
        e.ungapped_seqs.push(ungapped_seq);
    }

    let msa_count = e.msas.len() as uint;
    for msa_index in 0..msa_count {
        let m = &e.msas[msa_index as usize];
        assert_eq!(m.seqs.len() as uint, seq_count);
        for seq_index in 0..seq_count {
            assert_eq!(
                e.msas[0].seqs[seq_index as usize].label,
                m.seqs[seq_index as usize].label
            );
            let ungapped_seq = msa_get_ungapped_seq_str(m, seq_index);
            if ungapped_seq != e.ungapped_seqs[seq_index as usize] {
                panic!("MSA {msa_index} UngappedSeq != m_UngappedSeqs[{seq_index}]");
            }
        }
    }
}

/// Builds the per-MSA, per-sequence column-to-ungapped-position lookup table.
#[track_caller]
pub fn ensemble_set_col_to_pos_vec(e: &mut Ensemble) {
    let msa_count = e.msas.len() as uint;
    let seq_count = ensemble_get_seq_count(e);
    e.col_to_pos_vec.clear();
    e.col_to_pos_vec.resize(msa_count as usize, Vec::new());
    for msa_index in 0..msa_count {
        e.col_to_pos_vec[msa_index as usize].resize(seq_count as usize, Vec::new());
        for seq_index in 0..seq_count {
            e.col_to_pos_vec[msa_index as usize][seq_index as usize] =
                msa_get_col_to_pos1(&e.msas[msa_index as usize], seq_index);
        }
    }
}

/// Returns the column string and ungapped-position vector for `msa_index, col_index`.
#[track_caller]
pub fn ensemble_get_column(e: &Ensemble, msa_index: uint, col_index: uint) -> (String, Vec<i32>) {
    let m = &e.msas[msa_index as usize];
    let seq_count = ensemble_get_seq_count(e);
    let mut col_str = String::new();
    let mut pos_vec = Vec::new();
    for seq_index in 0..seq_count {
        let c = msa_get_char(m, seq_index, col_index);
        col_str.push(c);

        let pos = e.col_to_pos_vec[msa_index as usize][seq_index as usize][col_index as usize];
        pos_vec.push(pos);
        if pos > 0 {
            let ungapped_seq = &e.ungapped_seqs[seq_index as usize];
            assert!((pos as usize) <= ungapped_seq.len());
            let c2 = ungapped_seq.as_bytes()[pos as usize - 1] as char;
            assert_eq!(c2, c);
        }
    }
    (col_str, pos_vec)
}

/// Collects every column across all MSAs into flat arrays and builds the unique-column map.
#[track_caller]
pub fn ensemble_set_columns(e: &mut Ensemble) {
    e.column_strings.clear();
    e.column_positions.clear();
    e.ix_to_msa_index.clear();
    e.ix_to_col_index.clear();

    let msa_count = e.msas.len() as uint;
    let seq_count = ensemble_get_seq_count(e);
    if msa_count == 0 {
        return;
    }

    for msa_index in 0..msa_count {
        let m = &e.msas[msa_index as usize];
        let seq_count2 = m.seqs.len() as uint;
        assert_eq!(seq_count2, seq_count);

        let col_count = multi_sequence_get_col_count(m);
        for col_index in 0..col_count {
            let (col_str, pos_vec) = ensemble_get_column(e, msa_index, col_index);
            e.column_strings.push(col_str);
            e.column_positions.push(pos_vec);
            e.ix_to_msa_index.push(msa_index);
            e.ix_to_col_index.push(col_index);
        }
    }
    ensemble_set_unique_col_map(e);
}

/// Groups identical columns (by position vector) into unique IDs and fills cross-index maps.
#[track_caller]
pub fn ensemble_set_unique_col_map(e: &mut Ensemble) {
    e.unique_ixs.clear();
    e.unique_ix_to_ixs.clear();
    e.ix_to_unique_ix.clear();
    e.unique_col_map.clear();
    e.msa_col_to_ix.clear();

    let msa_count = e.msas.len() as uint;
    e.msa_col_to_ix.resize(msa_count as usize, Vec::new());
    for msa_index in 0..msa_count {
        let col_count = multi_sequence_get_col_count(&e.msas[msa_index as usize]);
        e.msa_col_to_ix[msa_index as usize].resize(col_count as usize, uint::MAX);
    }

    let n = e.column_positions.len() as uint;
    for ix in 0..n {
        let msa_index = e.ix_to_msa_index[ix as usize];
        let col_index = e.ix_to_col_index[ix as usize];
        assert!(msa_index < msa_count);
        assert!((col_index as usize) < e.msa_col_to_ix[msa_index as usize].len());
        e.msa_col_to_ix[msa_index as usize][col_index as usize] = ix;

        let pos_vec = e.column_positions[ix as usize].clone();
        if let Some(unique_ix) = e.unique_col_map.get(&pos_vec).copied() {
            e.unique_ix_to_ixs[unique_ix as usize].push(ix);
            e.ix_to_unique_ix.push(unique_ix);
        } else {
            let unique_ix = e.unique_ixs.len() as uint;
            e.unique_col_map.insert(pos_vec, unique_ix);
            e.unique_ixs.push(ix);
            assert_eq!(e.unique_ix_to_ixs.len(), unique_ix as usize);
            e.unique_ix_to_ixs.push(vec![ix]);
            e.ix_to_unique_ix.push(unique_ix);
        }
    }
    ensemble_validate_unique_col_map(e);
}

/// Sanity-checks the unique-column data for a single `(msa_index, col_index)` cell.
#[track_caller]
pub fn ensemble_validate_unique_col_map1(e: &Ensemble, msa_index: uint, col_index: uint) {
    assert!((col_index as usize) < e.msa_col_to_ix[msa_index as usize].len());
    let ix = e.msa_col_to_ix[msa_index as usize][col_index as usize];
    assert!((ix as usize) < e.column_positions.len());
    let pos_vec = &e.column_positions[ix as usize];

    assert!((ix as usize) < e.ix_to_unique_ix.len());
    let unique_ix = e.ix_to_unique_ix[ix as usize];

    assert!((unique_ix as usize) < e.unique_ix_to_ixs.len());
    let ixs = &e.unique_ix_to_ixs[unique_ix as usize];
    let mut found = false;
    for &ix2 in ixs {
        if ix2 == ix {
            found = true;
            break;
        }
    }
    assert!(found);

    let unique_ix2 = *e
        .unique_col_map
        .get(pos_vec)
        .expect("UniqueColMap missing position vector");
    assert_eq!(unique_ix, unique_ix2);
}

/// Sanity-checks the consistency of unique-column data for a single `unique_ix`.
#[track_caller]
pub fn ensemble_validate_unique_ix(e: &Ensemble, unique_ix: uint) {
    assert!((unique_ix as usize) < e.unique_ixs.len());
    assert!((unique_ix as usize) < e.unique_ix_to_ixs.len());

    let ix = e.unique_ixs[unique_ix as usize];
    assert!((ix as usize) < e.column_positions.len());

    let pos_vec = &e.column_positions[ix as usize];
    let unique_ix_from_map = *e
        .unique_col_map
        .get(pos_vec)
        .expect("UniqueColMap missing position vector");
    assert_eq!(unique_ix_from_map, unique_ix);

    let ixs = &e.unique_ix_to_ixs[unique_ix as usize];
    for &ix2 in ixs {
        assert!((ix2 as usize) < e.ix_to_unique_ix.len());
        let unique_ix2 = e.ix_to_unique_ix[ix2 as usize];
        assert_eq!(unique_ix2, unique_ix);

        let pos_vec2 = &e.column_positions[ix2 as usize];
        assert_eq!(pos_vec2, pos_vec);
    }
}

/// Validates every entry of the unique-column map across all MSAs and unique columns.
#[track_caller]
pub fn ensemble_validate_unique_col_map(e: &Ensemble) {
    let msa_count = e.msas.len() as uint;
    for msa_index in 0..msa_count {
        let col_count = multi_sequence_get_col_count(&e.msas[msa_index as usize]);
        assert!((msa_index as usize) < e.msa_col_to_ix.len());
        for col_index in 0..col_count {
            ensemble_validate_unique_col_map1(e, msa_index, col_index);
        }
    }

    let unique_ix_count = e.unique_ixs.len() as uint;
    for unique_ix in 0..unique_ix_count {
        ensemble_validate_unique_ix(e, unique_ix);
    }
}

/// Returns the fraction of gap characters in the column at flat index `ix`.
#[track_caller]
pub fn ensemble_get_gap_fract(e: &Ensemble, ix: uint) -> f64 {
    assert!((ix as usize) < e.column_strings.len());
    let col_str = &e.column_strings[ix as usize];
    let seq_count = ensemble_get_seq_count(e);
    assert_eq!(col_str.len(), seq_count as usize);
    let mut gap_count = 0_u32;
    for c in col_str.chars() {
        if c == '-' || c == '.' {
            gap_count += 1;
        }
    }
    f64::from(gap_count) / f64::from(seq_count)
}

/// Bootstrap-resamples `col_count` ensemble columns whose gap fraction is at most `max_gap_fract`.
#[track_caller]
pub fn ensemble_subsample_with_replacement_l569(
    e: &Ensemble,
    max_gap_fract: f64,
    col_count: uint,
) -> MultiSequence {
    let ixs = ensemble_get_ix_subset(e, max_gap_fract);
    ensemble_subsample_with_replacement_l577(e, &ixs, col_count)
}

/// Builds an MSA of `col_count` columns sampled with replacement from `ixs`.
#[track_caller]
pub fn ensemble_subsample_with_replacement_l577(
    e: &Ensemble,
    ixs: &[uint],
    col_count: uint,
) -> MultiSequence {
    assert!(col_count > 0);
    let seq_count = ensemble_get_seq_count(e);
    let mut m = MultiSequence::default();
    msa_set_size(&mut m, seq_count, col_count);

    assert_eq!(e.labels0.len(), seq_count as usize);
    for seq_index in 0..seq_count {
        let label = &e.labels0[seq_index as usize];
        m.seqs[seq_index as usize].label = label.clone();
    }

    let n = ixs.len() as uint;
    for i in 0..n {
        let r = randu32() % n;
        let ix = ixs[r as usize];
        assert!((ix as usize) < e.column_strings.len());
        let col_str = &e.column_strings[ix as usize];
        assert_eq!(col_str.len(), seq_count as usize);
        for seq_index in 0..seq_count {
            m.seqs[seq_index as usize].char_vec[i as usize] =
                col_str.as_bytes()[seq_index as usize] as char;
        }
    }
    m
}

/// Returns the indexes of ensemble columns whose gap fraction is at most `max_gap_fract`.
#[track_caller]
pub fn ensemble_get_ix_subset(e: &Ensemble, max_gap_fract: f64) -> Vec<uint> {
    let mut ixs = Vec::new();
    let ix_count = e.column_strings.len() as uint;
    for ix in 0..ix_count {
        let gap_fract = ensemble_get_gap_fract(e, ix);
        if gap_fract <= max_gap_fract {
            ixs.push(ix);
        }
    }
    ixs
}

/// Sums per-MSA column-agreement counts (`ab -> count`) over all MSAs in the ensemble.
#[track_caller]
pub fn ensemble_get_ab_to_count_all(e: &Ensemble) -> Vec<uint> {
    let msa_count = e.msas.len() as uint;
    let mut ab_to_count_all = vec![0_u32; msa_count as usize + 1];
    for msa_index in 0..msa_count {
        let ab_to_count = ensemble_get_ab_to_count(e, msa_index);
        assert_eq!(ab_to_count.len(), msa_count as usize + 1);
        for i in 0..msa_count {
            ab_to_count_all[i as usize] += ab_to_count[i as usize];
        }
    }
    ab_to_count_all
}

/// Returns a histogram of column agreement counts (`ab` -> count) for the given MSA.
#[track_caller]
pub fn ensemble_get_ab_to_count(e: &Ensemble, msa_index: uint) -> Vec<uint> {
    let msa_count = e.msas.len() as uint;
    assert!(msa_index < msa_count);
    let mut ab_to_count = vec![0_u32; msa_count as usize + 1];
    let m = &e.msas[msa_index as usize];
    let col_count = multi_sequence_get_col_count(m);
    for col in 0..col_count {
        let ab = ensemble_get_ab(e, msa_index, col);
        assert!(ab > 0);
        assert!(ab <= msa_count);
        ab_to_count[ab as usize] += 1;
    }
    ab_to_count
}

/// Returns the flat column index for `(msa_index, col_index)`.
#[track_caller]
pub fn ensemble_get_ix(e: &Ensemble, msa_index: uint, col_index: uint) -> uint {
    assert!((msa_index as usize) < e.msa_col_to_ix.len());
    assert!((col_index as usize) < e.msa_col_to_ix[msa_index as usize].len());
    e.msa_col_to_ix[msa_index as usize][col_index as usize]
}

/// Returns the unique-column ID for the column at `(msa_index, col_index)`.
#[track_caller]
pub fn ensemble_get_unique_ix(e: &Ensemble, msa_index: uint, col_index: uint) -> uint {
    let ix = ensemble_get_ix(e, msa_index, col_index);
    assert!((ix as usize) < e.ix_to_unique_ix.len());
    e.ix_to_unique_ix[ix as usize]
}

/// Returns the median per-column confidence of MSA `msa_index`.
#[track_caller]
pub fn ensemble_get_median_conf(e: &Ensemble, msa_index: uint) -> f64 {
    assert!((msa_index as usize) < e.msas.len());
    let m = &e.msas[msa_index as usize];
    let col_count = multi_sequence_get_col_count(m);
    let mut confs = Vec::<f64>::new();
    for col_index in 0..col_count {
        let ix = e.msa_col_to_ix[msa_index as usize][col_index as usize];
        assert!((ix as usize) < e.ix_to_unique_ix.len());
        let unique_ix = e.ix_to_unique_ix[ix as usize];
        let conf = ensemble_get_conf(e, unique_ix);
        confs.push(conf);
    }
    confs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    confs[col_count as usize / 2]
}

/// Returns the sum of column confidences for MSA `msa_index`.
#[track_caller]
pub fn ensemble_get_total_conf(e: &Ensemble, msa_index: uint) -> f64 {
    assert!((msa_index as usize) < e.msas.len());
    let m = &e.msas[msa_index as usize];
    let col_count = multi_sequence_get_col_count(m);
    assert!((msa_index as usize) < e.msa_col_to_ix.len());
    assert_eq!(
        e.msa_col_to_ix[msa_index as usize].len(),
        col_count as usize
    );
    let mut sum_conf = 0.0;
    for col_index in 0..col_count {
        let ix = e.msa_col_to_ix[msa_index as usize][col_index as usize];
        assert!((ix as usize) < e.ix_to_unique_ix.len());
        let unique_ix = e.ix_to_unique_ix[ix as usize];
        let conf = ensemble_get_conf(e, unique_ix);
        sum_conf += conf;
    }
    sum_conf
}

/// Returns the confidence of the column at `(msa_index, col_index)`.
#[track_caller]
pub fn ensemble_get_conf_msa_col(e: &Ensemble, msa_index: uint, col_index: uint) -> f64 {
    assert!((msa_index as usize) < e.msa_col_to_ix.len());
    assert!((col_index as usize) < e.msa_col_to_ix[msa_index as usize].len());
    let ix = e.msa_col_to_ix[msa_index as usize][col_index as usize];
    let unique_ix = e.ix_to_unique_ix[ix as usize];
    ensemble_get_conf(e, unique_ix)
}

/// Returns the fraction of MSAs that contain the unique column `unique_ix`.
#[track_caller]
pub fn ensemble_get_conf(e: &Ensemble, unique_ix: uint) -> f64 {
    let msa_count = e.msas.len() as uint;
    assert!((unique_ix as usize) < e.unique_ix_to_ixs.len());
    let ab = e.unique_ix_to_ixs[unique_ix as usize].len() as uint;
    f64::from(ab) / f64::from(msa_count)
}

/// Returns how many MSAs share the unique column found at `(msa_index, col_index)`.
#[track_caller]
pub fn ensemble_get_ab(e: &Ensemble, msa_index: uint, col_index: uint) -> uint {
    let unique_ix = ensemble_get_unique_ix(e, msa_index, col_index);
    assert!((unique_ix as usize) < e.unique_ix_to_ixs.len());
    let ab = e.unique_ix_to_ixs[unique_ix as usize].len() as uint;
    assert!(ab > 0);
    ab
}

/// Counts MSA `msa_index` columns that are unique to a single MSA (`ab == 1`).
#[track_caller]
pub fn ensemble_get_n1(e: &Ensemble, msa_index: uint) -> uint {
    assert!((msa_index as usize) < e.msas.len());
    let m = &e.msas[msa_index as usize];
    let col_count = multi_sequence_get_col_count(m);
    let mut n1 = 0_u32;
    for col_index in 0..col_count {
        let ab = ensemble_get_ab(e, msa_index, col_index);
        if ab == 1 {
            n1 += 1;
        }
    }
    n1
}

/// Returns dispersion `(D_letter_pairs, D_columns) = 1 - median(Q, TC)` over all MSA pairs.
#[track_caller]
pub fn ensemble_get_dispersion(e: &Ensemble, max_gap_fract: f64) -> (f64, f64) {
    let mut qs = QScorer {
        max_gap_fract,
        ..QScorer::default()
    };

    let mut q_vals = Vec::<f64>::new();
    let mut tc_vals = Vec::<f64>::new();
    let msa_count = e.msas.len() as uint;
    let pair_count = (msa_count * (msa_count - 1)) / 2;
    let mut pair_index = 0_u32;
    for i in 0..msa_count {
        let msa_i = &e.msas[i as usize];
        let name_i = &e.msa_names[i as usize];
        for j in (i + 1)..msa_count {
            assert!(pair_index < pair_count);
            pair_index += 1;

            let msa_j = &e.msas[j as usize];
            let name_j = &e.msa_names[j as usize];
            q_scorer_run_l337(&mut qs, name_i, msa_i, msa_j);
            let qij = qs.q as f64;
            let tcij = qs.tc as f64;

            q_scorer_run_l337(&mut qs, name_j, msa_j, msa_i);
            let qji = qs.q as f64;
            let tcji = qs.tc as f64;

            let q = (qij + qji) / 2.0;
            let tc = (tcij + tcji) / 2.0;
            assert!((0.0..=1.0).contains(&q));
            assert!((0.0..=1.0).contains(&tc));
            q_vals.push(q);
            tc_vals.push(tc);
        }
    }
    q_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    tc_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = q_vals.len();
    assert_eq!(tc_vals.len(), n);

    let median_q = q_vals[n / 2];
    let median_tc = tc_vals[n / 2];

    let d_letter_pairs = 1.0 - median_q;
    let d_columns = 1.0 - median_tc;
    assert!((0.0..=1.0).contains(&d_letter_pairs));
    assert!((0.0..=1.0).contains(&d_columns));
    (d_letter_pairs, d_columns)
}

/// Asserts that `ref_msa` shares the same sequences in the same order as the ensemble.
#[track_caller]
pub fn ensemble_check_ref_msa(e: &Ensemble, ref_msa: &MultiSequence) {
    let seq_count = ref_msa.seqs.len() as uint;
    let ref_seq_count = ref_msa.seqs.len() as uint;

    if ref_seq_count != seq_count {
        panic!("Different nr seqs");
    }

    for seq_index in 0..seq_count {
        let label = msa_get_seq_name(ref_msa, seq_index);
        if label != e.labels0[seq_index as usize] {
            panic!("GetRefUniqueIxs, not sorted");
        }
    }
}

/// Returns the set of ensemble unique-column IDs that match an upper-case column in `ref_msa`.
#[track_caller]
pub fn ensemble_get_ref_unique_ixs(
    e: &Ensemble,
    ref_msa: &MultiSequence,
    max_gap_fract: f64,
) -> std::collections::BTreeSet<uint> {
    let mut unique_ixs = std::collections::BTreeSet::<uint>::new();
    ensemble_check_ref_msa(e, ref_msa);

    let seq_count = ensemble_get_seq_count(e);
    let ref_col_count = multi_sequence_get_col_count(ref_msa);

    let mut col_to_pos_vec = Vec::<Vec<i32>>::new();
    for ref_seq_index in 0..seq_count {
        col_to_pos_vec.push(msa_get_col_to_pos1(ref_msa, ref_seq_index));
    }

    for ref_col_index in 0..ref_col_count {
        let is_upper = msa_col_is_upper(ref_msa, ref_col_index, max_gap_fract);
        if !is_upper {
            continue;
        }

        let mut pos_vec = Vec::<i32>::new();
        for seq_index in 0..seq_count {
            assert!((seq_index as usize) < col_to_pos_vec.len());
            assert!((ref_col_index as usize) < col_to_pos_vec[seq_index as usize].len());
            let pos = col_to_pos_vec[seq_index as usize][ref_col_index as usize];
            pos_vec.push(pos);
        }
        if let Some(unique_ix) = e.unique_col_map.get(&pos_vec).copied() {
            unique_ixs.insert(unique_ix);
        }
    }
    unique_ixs
}

/// Returns `(seq_index, pos)` pairs for every reference upper-case column position.
#[track_caller]
pub fn ensemble_get_ref_pos_set(
    e: &Ensemble,
    ref_msa: &MultiSequence,
    max_gap_fract: f64,
) -> std::collections::BTreeSet<(uint, i32)> {
    let mut pos_set = std::collections::BTreeSet::<(uint, i32)>::new();

    ensemble_check_ref_msa(e, ref_msa);
    let seq_count = ensemble_get_seq_count(e);
    let ref_col_count = multi_sequence_get_col_count(ref_msa);

    let mut col_to_pos_vec = Vec::<Vec<i32>>::new();
    for seq_index in 0..seq_count {
        col_to_pos_vec.push(msa_get_col_to_pos1(ref_msa, seq_index));
    }

    for ref_col_index in 0..ref_col_count {
        let is_upper = msa_col_is_upper(ref_msa, ref_col_index, max_gap_fract);
        if !is_upper {
            continue;
        }

        for seq_index in 0..seq_count {
            let pos = col_to_pos_vec[seq_index as usize][ref_col_index as usize];
            pos_set.insert((seq_index, pos));
        }
    }
    pos_set
}

/// Returns unique-column ids in `msa_index` matching a majority of reference positions, with confs.
#[track_caller]
pub fn ensemble_get_test_unique_ixs(
    e: &Ensemble,
    msa_index: uint,
    ref_pos_set: &std::collections::BTreeSet<(uint, i32)>,
) -> (Vec<uint>, Vec<f64>) {
    let mut unique_ixs = Vec::<uint>::new();
    let mut confs = Vec::<f64>::new();

    let msa_count = e.msas.len() as uint;
    let seq_count = ensemble_get_seq_count(e);
    assert!(msa_index < msa_count);

    let m = &e.msas[msa_index as usize];
    let col_count = multi_sequence_get_col_count(m);
    assert!((msa_index as usize) < e.msa_col_to_ix.len());
    assert_eq!(
        e.msa_col_to_ix[msa_index as usize].len(),
        col_count as usize
    );
    for col_index in 0..col_count {
        let ix = e.msa_col_to_ix[msa_index as usize][col_index as usize];
        assert!((ix as usize) < e.column_positions.len());
        let pos_vec = &e.column_positions[ix as usize];
        assert_eq!(pos_vec.len(), seq_count as usize);
        let mut found_count = 0_u32;
        for seq_index in 0..seq_count {
            let pos = pos_vec[seq_index as usize];
            if pos <= 0 {
                continue;
            }

            if ref_pos_set.contains(&(seq_index, pos)) {
                found_count += 1;
            }
        }
        if found_count >= seq_count / 2 {
            assert!((ix as usize) < e.ix_to_unique_ix.len());
            let unique_ix = e.ix_to_unique_ix[ix as usize];
            let conf = ensemble_get_conf(e, unique_ix);

            unique_ixs.push(unique_ix);
            confs.push(conf);
        }
    }
    (unique_ixs, confs)
}

/// Returns a reference to MSA `msa_index`.
#[track_caller]
pub fn ensemble_get_msa(e: &Ensemble, msa_index: uint) -> &MultiSequence {
    assert!((msa_index as usize) < e.msas.len());
    &e.msas[msa_index as usize]
}

/// Returns the name (e.g. file basename) of MSA `msa_index`.
#[track_caller]
pub fn ensemble_get_msa_name(e: &Ensemble, msa_index: uint) -> &str {
    assert!((msa_index as usize) < e.msa_names.len());
    &e.msa_names[msa_index as usize]
}
