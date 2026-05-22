// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Derep {
    pub input_seqs: Option<MultiSequence>,
    pub seq_index_to_rep_seq_index: Vec<uint>,
    pub rep_seq_indexes: Vec<uint>,
    pub rep_seq_index_to_seq_indexes: Vec<Vec<uint>>,
    pub slot_count: uint,
    pub hash_to_seq_indexes: Vec<Vec<uint>>,
    pub disable: bool,
} // original: Derep (muscle/src/derep.h)

/// Reset the dereplication state, dropping all per-sequence mappings and the hash table.
#[track_caller]
pub fn derep_clear(d: &mut Derep) {
    d.seq_index_to_rep_seq_index.clear();
    d.rep_seq_indexes.clear();
    d.rep_seq_index_to_seq_indexes.clear();
    d.hash_to_seq_indexes.clear();
}

/// FNV64 hash of `seq` (case-insensitive), reduced modulo the hash-table slot count.
#[track_caller]
pub fn derep_calc_hash(d: &Derep, seq: &Sequence) -> uint {
    let mut hash: uint64 = 0xcbf29ce484222325_u64;
    for &c in &seq.char_vec {
        let b = (c as byte).to_ascii_lowercase();
        hash = hash.wrapping_mul(1099511628211_u64);
        hash ^= b as uint64;
    }
    (hash % d.slot_count as uint64) as uint
}

/// Hash every input sequence and build the rep/duplicate index tables.
#[track_caller]
pub fn derep_run(d: &mut Derep, input_seqs: &MultiSequence, show_progress: bool) {
    derep_clear(d);
    d.input_seqs = Some(input_seqs.clone());
    let input_seq_count = input_seqs.seqs.len() as uint;
    d.slot_count = 3 * input_seq_count + 7;

    d.hash_to_seq_indexes = vec![Vec::new(); d.slot_count as usize];
    d.seq_index_to_rep_seq_index = vec![uint::MAX; input_seq_count as usize];
    d.rep_seq_index_to_seq_indexes = vec![Vec::new(); input_seq_count as usize];

    let mut unique_count = 0_u32;
    let mut dupe_count = 0_u32;
    for seq_index in 0..input_seq_count {
        let rep_seq_index = derep_search(d, seq_index);
        if rep_seq_index == uint::MAX {
            derep_add_to_hash(d, seq_index);
            assert!(d.rep_seq_indexes.len() as uint == unique_count);
            d.rep_seq_indexes.push(seq_index);
            d.rep_seq_index_to_seq_indexes[seq_index as usize].push(seq_index);
            d.seq_index_to_rep_seq_index[seq_index as usize] = seq_index;
            unique_count += 1;
        } else {
            d.rep_seq_index_to_seq_indexes[rep_seq_index as usize].push(seq_index);
            d.seq_index_to_rep_seq_index[seq_index as usize] = rep_seq_index;
            dupe_count += 1;
        }
        if show_progress {
            let _ = (seq_index, input_seq_count, unique_count, dupe_count);
        }
    }
}

/// Case-insensitive sequence equality; always false when dereplication is disabled.
#[track_caller]
pub fn derep_seqs_eq(d: &Derep, seq_index1: uint, seq_index2: uint) -> bool {
    if d.disable {
        return false;
    }
    let input_seqs = d.input_seqs.as_ref().expect("Derep::SeqsEq, no input seqs");
    let seq1 = &input_seqs.seqs[seq_index1 as usize];
    let seq2 = &input_seqs.seqs[seq_index2 as usize];
    let l = seq1.char_vec.len();
    let l2 = seq2.char_vec.len();
    if l2 != l {
        return false;
    }
    for i in 0..l {
        if seq1.char_vec[i].to_ascii_uppercase() != seq2.char_vec[i].to_ascii_uppercase() {
            return false;
        }
    }
    true
}

/// Return the rep-seq index of an existing duplicate of `seq_index`, or `uint::MAX` if none.
#[track_caller]
pub fn derep_search(d: &Derep, seq_index: uint) -> uint {
    if d.disable {
        return uint::MAX;
    }
    let input_seqs = d.input_seqs.as_ref().expect("Derep::Search, no input seqs");
    let seq = &input_seqs.seqs[seq_index as usize];
    let h = derep_calc_hash(d, seq);
    assert!((h as usize) < d.hash_to_seq_indexes.len());
    let row = &d.hash_to_seq_indexes[h as usize];
    for &seq_index2 in row {
        if derep_seqs_eq(d, seq_index, seq_index2) {
            return seq_index2;
        }
    }
    uint::MAX
}

/// Insert `seq_index` into the hash bucket of its current sequence.
#[track_caller]
pub fn derep_add_to_hash(d: &mut Derep, seq_index: uint) {
    let input_seqs = d
        .input_seqs
        .as_ref()
        .expect("Derep::AddToHash, no input seqs");
    let seq = &input_seqs.seqs[seq_index as usize];
    let h = derep_calc_hash(d, seq);
    assert!((h as usize) < d.hash_to_seq_indexes.len());
    d.hash_to_seq_indexes[h as usize].push(seq_index);
}

/// Copy the representative (unique) sequences into `unique_seqs`.
#[track_caller]
pub fn derep_get_unique_seqs(d: &Derep, unique_seqs: &mut MultiSequence) {
    assert!(unique_seqs.seqs.is_empty());
    let input_seqs = d
        .input_seqs
        .as_ref()
        .expect("Derep::GetUniqueSeqs, no input seqs");
    for &seq_index in &d.rep_seq_indexes {
        unique_seqs
            .seqs
            .push(input_seqs.seqs[seq_index as usize].clone());
        unique_seqs.owners.push(false);
    }
}

/// Build a map from each representative label to the labels of its non-rep duplicates.
#[track_caller]
pub fn derep_get_rep_label_to_dupe_labels(
    d: &Derep,
) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut rep_label_to_member_labels = std::collections::BTreeMap::new();
    let input_seqs = d
        .input_seqs
        .as_ref()
        .expect("Derep::GetRepLabelToDupeLabels, no input seqs");
    for &rep_seq_index in &d.rep_seq_indexes {
        let rep_label = input_seqs.seqs[rep_seq_index as usize].label.clone();
        let member_seq_indexes = &d.rep_seq_index_to_seq_indexes[rep_seq_index as usize];
        if member_seq_indexes.len() == 1 {
            continue;
        }
        let row = rep_label_to_member_labels
            .entry(rep_label.clone())
            .or_insert_with(Vec::new);
        for &member_seq_index in member_seq_indexes {
            let member_label = input_seqs.seqs[member_seq_index as usize].label.clone();
            if member_label != rep_label {
                row.push(member_label);
            }
        }
    }
    rep_label_to_member_labels
}

/// Sanity-check the internal dereplication tables.
#[track_caller]
pub fn derep_validate(d: &Derep) {
    let input_seqs = d
        .input_seqs
        .as_ref()
        .expect("Derep::Validate, no input seqs");
    let input_seq_count = input_seqs.seqs.len();
    assert!(d.seq_index_to_rep_seq_index.len() == input_seq_count);
    assert!(d.rep_seq_index_to_seq_indexes.len() == input_seq_count);

    let mut rep_seq_index_set = std::collections::BTreeSet::new();
    for seq_index in 0..input_seq_count {
        let rep_seq_index = d.seq_index_to_rep_seq_index[seq_index];
        rep_seq_index_set.insert(rep_seq_index);
    }

    let rep_seq_index_count = d.rep_seq_indexes.len();
    assert!(rep_seq_index_set.len() == rep_seq_index_count);
    for &rep_seq_index in &d.rep_seq_indexes {
        assert!(rep_seq_index_set.contains(&rep_seq_index));
        let member_seq_indexes = &d.rep_seq_index_to_seq_indexes[rep_seq_index as usize];
        assert!(!member_seq_indexes.is_empty());
        for &member_seq_index in member_seq_indexes {
            let member_rep_seq_index = d.seq_index_to_rep_seq_index[member_seq_index as usize];
            assert!(member_rep_seq_index == rep_seq_index);
        }
    }
}

/// Return parallel vectors of global-sequence indexes: (duplicate GSI, corresponding representative GSI).
#[track_caller]
pub fn derep_get_dupe_gs_is(d: &Derep) -> (Vec<uint>, Vec<uint>) {
    let mut gsis = Vec::new();
    let mut global_rep_seq_indexes = Vec::new();
    let input_seqs = d
        .input_seqs
        .as_ref()
        .expect("Derep::GetDupeGSIs, no input seqs");
    let input_seq_count = input_seqs.seqs.len() as uint;
    let global_ms_seq_count = get_global_ms_seq_count();

    for &rep_seq_index in &d.rep_seq_indexes {
        assert!(rep_seq_index < input_seq_count);
        let member_seq_indexes = &d.rep_seq_index_to_seq_indexes[rep_seq_index as usize];
        let seq = &input_seqs.seqs[rep_seq_index as usize];
        let global_rep_seq_index = get_gsi_by_label(&seq.label);
        assert!(global_rep_seq_index < global_ms_seq_count);
        assert!(member_seq_indexes[0] == rep_seq_index);
        for &member_seq_index in member_seq_indexes.iter().skip(1) {
            let seq = &input_seqs.seqs[member_seq_index as usize];
            let global_member_seq_index = get_gsi_by_label(&seq.label);
            assert!(global_member_seq_index < global_ms_seq_count);
            gsis.push(global_member_seq_index);
            global_rep_seq_indexes.push(global_rep_seq_index);
        }
    }
    (gsis, global_rep_seq_indexes)
}

/// Driver: read MFA input, dereplicate, and write the unique sequences as FASTA.
#[track_caller]
pub fn cmd_derep(input_file_name: &str, output_file_name: &str) {
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, false);

    let mut d = Derep::default();
    derep_run(&mut d, &input_seqs, true);
    derep_validate(&d);

    let mut unique_seqs = MultiSequence::default();
    derep_get_unique_seqs(&d, &mut unique_seqs);
    let mut out = String::new();
    for seq in &unique_seqs.seqs {
        out.push_str(&seq_to_fasta_l2561(
            &sequence_get_seq_as_string(seq),
            &seq.label,
        ));
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write derep output");
    }
}
