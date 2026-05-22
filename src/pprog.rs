// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct PProg {
    pub input_msa_count: uint,
    pub join_count: uint,
    pub node_count: uint,
    pub target_pair_count: uint,
    pub max_coarse_seqs: uint,
    pub msa_label_to_index: std::collections::BTreeMap<String, uint>,
    pub msa_labels: Vec<String>,
    pub msas: Vec<Option<MultiSequence>>,
    pub pending: Vec<uint>,
    pub score_mx: Vec<Vec<f32>>,
    pub path_mx: Vec<Vec<String>>,
    pub join_index: uint,
    pub join_msa_indexes1: Vec<uint>,
    pub join_msa_indexes2: Vec<uint>,
} // original: PProg (muscle/src/pprog.h)

/// Reads `file_name` as text and returns one entry per line.
#[track_caller]
pub fn read_strings_from_file(file_name: &str) -> Vec<String> {
    let bytes = std::fs::read(file_name).unwrap();
    let mut strings = Vec::new();
    let mut line = String::new();
    for b in bytes {
        if b == b'\r' {
            continue;
        }
        if b == b'\n' {
            strings.push(line);
            line = String::new();
        } else {
            line.push(b as char);
        }
    }
    if !line.is_empty() {
        strings.push(line);
    }
    strings
}

/// Swaps `X` and `Y` columns of a `B/X/Y` PProg path (leaving `B` unchanged).
pub fn invert_path(path: &str) -> String {
    let mut inverted_path = String::new();
    for c in path.bytes() {
        if c == b'B' {
            inverted_path.push('B');
        } else if c == b'X' {
            inverted_path.push('Y');
        } else if c == b'Y' {
            inverted_path.push('X');
        } else {
            panic!("Invalid path char '{}'", c as char);
        }
    }
    inverted_path
}

/// Asserts that a `B/X/Y` path consumes exactly `lx` X-columns and `ly` Y-columns.
pub fn validate_path(path: &str, lx: uint, ly: uint) {
    let mut nx = 0;
    let mut ny = 0;
    for c in path.bytes() {
        match c.to_ascii_uppercase() {
            b'X' => nx += 1,
            b'Y' => ny += 1,
            b'B' => {
                nx += 1;
                ny += 1;
            }
            _ => panic!("invalid path char"),
        }
    }
    assert_eq!(nx, lx);
    assert_eq!(ny, ly);
}

/// Materialises the joined MSA by inserting gaps into each sequence according to `path`.
#[track_caller]
pub fn align_ms_as_by_path(
    msa1: &MultiSequence,
    msa2: &MultiSequence,
    path: &str,
    msa12: &mut MultiSequence,
) {
    let lx = multi_sequence_get_col_count(msa1);
    let ly = multi_sequence_get_col_count(msa2);
    validate_path(path, lx, ly);
    let seq_count1 = msa1.seqs.len();
    let seq_count2 = msa2.seqs.len();

    for seq_index in 0..seq_count1 {
        let seq1 = &msa1.seqs[seq_index];
        let aligned_seq1 = sequence_add_gaps_path(seq1, path, 'X');
        msa12.seqs.push(aligned_seq1);
        msa12.owners.push(true);
    }

    for seq_index in 0..seq_count2 {
        let seq2 = &msa2.seqs[seq_index];
        let aligned_seq2 = sequence_add_gaps_path(seq2, path, 'Y');
        msa12.seqs.push(aligned_seq2);
        msa12.owners.push(true);
    }
}

/// Removes both `index1` and `index2` from the pending list, asserting each appears once.
#[track_caller]
pub fn p_prog_delete_indexes_from_pending(pp: &mut PProg, index1: uint, index2: uint) {
    let mut found1 = false;
    let mut found2 = false;
    let mut new_pending = Vec::new();
    for i in 0..pp.pending.len() {
        let index = pp.pending[i];
        if index == index1 {
            assert!(!found1);
            found1 = true;
            continue;
        }
        if index == index2 {
            assert!(!found2);
            found2 = true;
            continue;
        }
        new_pending.push(index);
    }
    assert!(found1);
    assert!(found2);
    assert_eq!(new_pending.len() + 2, pp.pending.len());
    pp.pending = new_pending;
}

/// Returns the pair `(i, j)` from the pending list with the highest score-matrix entry.
#[track_caller]
pub fn p_prog_find_best_pair(pp: &PProg) -> (uint, uint) {
    let n = pp.pending.len();
    assert!(n >= 2);
    let mut best_index1 = pp.pending[0];
    let mut best_index2 = pp.pending[1];
    assert!(best_index1 < pp.node_count);
    assert!(best_index2 < pp.node_count);
    let mut best_score = pp.score_mx[best_index1 as usize][best_index2 as usize];
    for i in 0..n {
        let indexi = pp.pending[i];
        for j in i + 1..n {
            let indexj = pp.pending[j];
            assert!((indexi as usize) < pp.score_mx.len());
            assert!((indexj as usize) < pp.score_mx[indexi as usize].len());
            let score = pp.score_mx[indexi as usize][indexj as usize];
            if score > best_score {
                best_score = score;
                best_index1 = indexi;
                best_index2 = indexj;
            }
        }
    }
    (best_index1, best_index2)
}

/// Returns the label associated with input/joined MSA `index`.
#[track_caller]
pub fn p_prog_get_msa_label(pp: &PProg, index: uint) -> &str {
    assert!((index as usize) < pp.msa_labels.len());
    let label = &pp.msa_labels[index as usize];
    assert!(!label.is_empty());
    label
}

/// Stores `msa` as MSA `index`.
#[track_caller]
pub fn p_prog_set_msa(pp: &mut PProg, index: uint, msa: &MultiSequence) {
    assert!((index as usize) < pp.msas.len());
    pp.msas[index as usize] = Some(msa.clone());
}

/// Sets the label for MSA `index`, registering it in the label-to-index map.
#[track_caller]
pub fn p_prog_set_msa_label(pp: &mut PProg, index: uint, label: &str) {
    assert!((index as usize) < pp.msa_labels.len());
    assert!(pp.msa_labels[index as usize].is_empty());
    pp.msa_labels[index as usize] = label.to_string();
    assert!(!pp.msa_label_to_index.contains_key(label));
    pp.msa_label_to_index.insert(label.to_string(), index);
}

/// Returns the stored MSA at slot `index` (panics if missing).
#[track_caller]
pub fn p_prog_get_msa(pp: &PProg, index: uint) -> &MultiSequence {
    assert!((index as usize) < pp.msas.len());
    pp.msas[index as usize]
        .as_ref()
        .expect("PProg::GetMSA null MSA")
}

/// Returns the final joined MSA at the last slot.
#[track_caller]
pub fn p_prog_get_final_msa(pp: &PProg) -> &MultiSequence {
    assert!(pp.input_msa_count > 0);
    let final_index = 2 * (pp.input_msa_count - 1);
    assert_eq!(final_index + 1, pp.msas.len() as uint);
    pp.msas[final_index as usize]
        .as_ref()
        .expect("PProg::GetFinalMSA null final MSA")
}

/// Initialises the program from in-memory `msas` and their labels (resizes slots).
#[track_caller]
pub fn p_prog_set_ms_as(pp: &mut PProg, msas: &[MultiSequence], msa_labels: &[String]) {
    pp.msa_label_to_index.clear();
    pp.input_msa_count = msas.len() as uint;
    assert_eq!(msa_labels.len() as uint, pp.input_msa_count);
    pp.msas = msas.iter().cloned().map(Some).collect();
    pp.msa_labels = msa_labels.to_vec();

    let total_msa_count = 2 * pp.input_msa_count - 1;
    pp.msas.resize(total_msa_count as usize, None);
    pp.msa_labels
        .resize(total_msa_count as usize, String::new());

    for msa_index in 0..pp.input_msa_count {
        let msa_label = &msa_labels[msa_index as usize];
        assert!(!pp.msa_label_to_index.contains_key(msa_label));
        pp.msa_label_to_index.insert(msa_label.clone(), msa_index);
    }
}

/// Loads MSAs from `file_names`, returns whether the data is nucleotide.
#[track_caller]
pub fn p_prog_load_ms_as(pp: &mut PProg, file_names: &[String]) -> bool {
    pp.msa_label_to_index.clear();
    pp.msas.clear();
    pp.msa_labels.clear();

    pp.input_msa_count = file_names.len() as uint;
    assert!(pp.input_msa_count > 1);

    let total_msa_count = 2 * pp.input_msa_count - 1;
    pp.msas.resize(total_msa_count as usize, None);
    pp.msa_labels
        .resize(total_msa_count as usize, String::new());

    pp.join_count = pp.input_msa_count - 1;
    pp.node_count = pp.input_msa_count + pp.join_count;

    let mut is_nucleo = false;
    let mut global_input = MultiSequence::default();
    for msa_index in 0..pp.input_msa_count {
        let file_name = &file_names[msa_index as usize];
        let mut msa = MultiSequence::default();
        multi_sequence_load_mfa_l8(&mut msa, file_name, false);
        let is_nuc = multi_sequence_guess_is_nucleo(&msa);
        if msa_index == 0 {
            is_nucleo = is_nuc;
        } else {
            assert_eq!(is_nucleo, is_nuc);
        }

        let msa_label = get_base_name(file_name);
        p_prog_set_msa_label(pp, msa_index, &msa_label);
        for seq in &msa.seqs {
            let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
            let mut global_seq = Sequence::default();
            sequence_from_string(&mut global_seq, &seq.label, &ungapped);
            global_input.seqs.push(global_seq);
            global_input.owners.push(false);
        }
        p_prog_set_msa(pp, msa_index, &msa);
    }
    set_global_input_ms(&global_input);
    is_nucleo
}

/// Runs the progressive join: aligns all input pairs, then repeatedly joins the best pair.
#[track_caller]
pub fn p_prog_run<FAlignMSAsFlat>(pp: &mut PProg, mut align_msas_flat: FAlignMSAsFlat)
where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    pp.join_msa_indexes1.clear();
    pp.join_msa_indexes2.clear();
    pp.score_mx.clear();
    pp.path_mx.clear();

    pp.pending.clear();
    for i in 0..pp.input_msa_count {
        pp.pending.push(i);
    }

    pp.join_count = pp.input_msa_count - 1;
    pp.node_count = pp.input_msa_count + pp.join_count;
    p_prog_align_all_input_pairs(pp, |label, msa1, msa2, target_pair_count, path| {
        align_msas_flat(label, msa1, msa2, target_pair_count, path)
    });

    pp.join_index = 0;
    while pp.join_index < pp.join_count {
        let (index1, index2) = p_prog_find_best_pair(pp);
        assert_ne!(index1, index2);
        p_prog_join_by_precomputed_path(pp, index1, index2);
        p_prog_align_new_to_pending(pp, |label, msa1, msa2, target_pair_count, path| {
            align_msas_flat(label, msa1, msa2, target_pair_count, path)
        });
        pp.join_index += 1;
    }
}

/// Aligns every input-MSA pair, filling `score_mx` and `path_mx`.
#[track_caller]
pub fn p_prog_align_all_input_pairs<FAlignMSAsFlat>(
    pp: &mut PProg,
    mut align_msas_flat: FAlignMSAsFlat,
) where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let _pair_count = (pp.input_msa_count * (pp.input_msa_count - 1)) / 2;
    pp.score_mx = vec![vec![0.0; pp.node_count as usize]; pp.node_count as usize];
    pp.path_mx = vec![vec![String::new(); pp.node_count as usize]; pp.node_count as usize];

    for msa_index1 in 0..pp.input_msa_count {
        let msa_label1 = p_prog_get_msa_label(pp, msa_index1).to_string();
        let msa1 = p_prog_get_msa(pp, msa_index1).clone();
        for msa_index2 in msa_index1 + 1..pp.input_msa_count {
            let msa_label2 = p_prog_get_msa_label(pp, msa_index2).to_string();
            let msa2 = p_prog_get_msa(pp, msa_index2).clone();
            let mut path = String::new();
            let score = align_msas_flat(
                &format!("{msa_label1}+{msa_label2}"),
                &msa1,
                &msa2,
                pp.target_pair_count,
                &mut path,
            );

            let col_count1 = multi_sequence_get_col_count(&msa1);
            let col_count2 = multi_sequence_get_col_count(&msa2);
            validate_path(&path, col_count1, col_count2);
            let inverted_path = invert_path(&path);
            validate_path(&inverted_path, col_count2, col_count1);

            pp.score_mx[msa_index1 as usize][msa_index2 as usize] = score;
            pp.score_mx[msa_index2 as usize][msa_index1 as usize] = score;
            pp.path_mx[msa_index1 as usize][msa_index2 as usize] = path;
            pp.path_mx[msa_index2 as usize][msa_index1 as usize] = inverted_path;
        }
    }
}

/// Returns a multi-line dump of the current pending MSAs for debugging.
#[track_caller]
pub fn p_prog_log_pending(pp: &PProg, s: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "\nLogPending({s}) m_JoinIndex={}\n",
        pp.join_index
    ));
    for i in 0..pp.pending.len() {
        let index = pp.pending[i];
        let msa = p_prog_get_msa(pp, index);
        let seq_count = msa.seqs.len() as uint;
        let col_count = multi_sequence_get_col_count(msa);
        out.push_str(&format!(
            "  [{index:4}]  seqs={seq_count},cols={col_count} {}\n",
            p_prog_get_msa_label(pp, index)
        ));
    }
    out
}

/// Joins MSAs `index1` and `index2` using the precomputed path, appending a new join slot.
#[track_caller]
pub fn p_prog_join_by_precomputed_path(pp: &mut PProg, index1: uint, index2: uint) {
    assert_eq!(pp.join_msa_indexes1.len() as uint, pp.join_index);
    assert_eq!(pp.join_msa_indexes2.len() as uint, pp.join_index);
    pp.join_msa_indexes1.push(index1);
    pp.join_msa_indexes2.push(index2);

    let new_msa_index = pp.input_msa_count + pp.join_index;
    let new_msa_label = format!("Join{}", pp.join_index + 1);

    let msa1 = p_prog_get_msa(pp, index1).clone();
    let msa2 = p_prog_get_msa(pp, index2).clone();
    let path = pp.path_mx[index1 as usize][index2 as usize].clone();
    let mut msa12 = MultiSequence::default();
    align_ms_as_by_path(&msa1, &msa2, &path, &mut msa12);

    p_prog_set_msa(pp, new_msa_index, &msa12);
    p_prog_set_msa_label(pp, new_msa_index, &new_msa_label);

    let _joined_seq_count = msa12.seqs.len() as uint;
    let _joined_col_count = multi_sequence_get_col_count(&msa12);

    pp.pending.push(new_msa_index);
    let pending_count_before_join = pp.pending.len();
    p_prog_delete_indexes_from_pending(pp, index1, index2);
    let pending_count_after_join = pp.pending.len();
    assert_eq!(pending_count_after_join + 2, pending_count_before_join);
}

/// Aligns the most recently joined MSA against each remaining pending MSA.
#[track_caller]
pub fn p_prog_align_new_to_pending<FAlignMSAsFlat>(
    pp: &mut PProg,
    mut align_msas_flat: FAlignMSAsFlat,
) where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let pending_count = pp.pending.len();
    assert!(pending_count > 0);
    let new_index = pp.pending[pending_count - 1];

    let new_msa_label = pp.msa_labels[new_index as usize].clone();
    let new_msa = p_prog_get_msa(pp, new_index).clone();
    for i in 0..pending_count - 1 {
        let index = pp.pending[i];
        let msa = p_prog_get_msa(pp, index).clone();
        let msa_labeli = pp.msa_labels[i].clone();
        let mut path = String::new();
        let score = align_msas_flat(
            &format!("{new_msa_label}+{msa_labeli}"),
            &new_msa,
            &msa,
            pp.target_pair_count,
            &mut path,
        );

        let inverted_path = invert_path(&path);
        pp.score_mx[new_index as usize][index as usize] = score;
        pp.score_mx[index as usize][new_index as usize] = score;
        pp.path_mx[new_index as usize][index as usize] = path;
        pp.path_mx[index as usize][new_index as usize] = inverted_path;
    }
}

/// Constructs the implicit guide tree from join order and writes it to `file_name`.
#[track_caller]
pub fn p_prog_write_guide_tree(pp: &PProg, file_name: &str) -> Option<Tree> {
    if file_name.is_empty() {
        return None;
    }
    let mut guide_tree = Tree::default();
    let label_to_index = pp
        .msa_label_to_index
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<std::collections::HashMap<_, _>>();
    make_guide_tree_from_join_order(
        &pp.join_msa_indexes1,
        &pp.join_msa_indexes2,
        &label_to_index,
        &mut guide_tree,
    );
    tree_to_file_l13(&guide_tree, file_name);
    Some(guide_tree)
}

/// Implements the `pprog` subcommand: loads MSAs from a list file, runs PProg, writes outputs.
#[track_caller]
pub fn cmd_pprog<FAlignMSAsFlat>(
    list_file_name: &str,
    output_file_name: &str,
    guide_tree_output_file_name: Option<&str>,
    target_pair_count: Option<uint>,
    mut align_msas_flat: FAlignMSAsFlat,
) -> (PProg, Option<Tree>)
where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let text = std::fs::read_to_string(list_file_name).expect("failed to read PProg file list");
    let msa_file_names = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(msa_file_names.len() > 1);

    let mut pp = PProg {
        target_pair_count: 2000,
        ..PProg::default()
    };
    if let Some(pair_count) = target_pair_count {
        pp.target_pair_count = pair_count;
    }

    let _is_nucleo = p_prog_load_ms_as(&mut pp, &msa_file_names);
    set_alpha_l209(if _is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    init_probcons();
    p_prog_run(&mut pp, |label, msa1, msa2, pair_count, path| {
        align_msas_flat(label, msa1, msa2, pair_count, path)
    });

    assert_eq!(pp.pending.len(), 1);
    let _index = pp.pending[0];
    let final_msa = p_prog_get_final_msa(&pp).clone();
    multi_sequence_write_mfa(&final_msa, output_file_name);
    let guide_tree =
        guide_tree_output_file_name.and_then(|file_name| p_prog_write_guide_tree(&pp, file_name));
    (pp, guide_tree)
}
