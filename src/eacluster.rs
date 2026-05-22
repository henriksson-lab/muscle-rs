// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct EACluster {
    pub input_seqs: Option<MultiSequence>,
    pub us: USorter,
    pub centroid_seq_indexes: Vec<uint>,
    pub centroid_index_to_seq_indexes: Vec<Vec<uint>>,
    pub seq_index_to_centroid_index: Vec<uint>,
    pub cluster_mfas: Vec<MultiSequence>,
} // original: EACluster (muscle/src/eacluster.h)

/// Build a replicate file name by substituting `@` in `pattern` with `n` (or appending `n` if absent).
#[track_caller]
pub fn make_replicate_file_name_n(pattern: &str, n: uint) -> String {
    let mut file_name = String::new();
    let mut found = false;
    for c in pattern.chars() {
        if c == '@' {
            file_name.push_str(&format!("{n}"));
            found = true;
        } else {
            file_name.push(c);
        }
    }
    if !found {
        file_name.push_str(&format!("{n}"));
    }
    file_name
}

/// Reset EACluster state (centroids, members, cluster MFAs).
#[track_caller]
pub fn ea_cluster_clear(ec: &mut EACluster) {
    ec.centroid_seq_indexes.clear();
    ec.centroid_index_to_seq_indexes.clear();
    ec.seq_index_to_centroid_index.clear();
    ec.cluster_mfas.clear();
}

/// UCLUST-style EA-driven clustering: assign each input sequence to an existing centroid (via `align_seq_pair`) or create a new one.
#[track_caller]
pub fn ea_cluster_run<FAlignSeqPair>(
    ec: &mut EACluster,
    input_seqs: &MultiSequence,
    min_ea: f32,
    mut align_seq_pair: FAlignSeqPair,
) where
    FAlignSeqPair: FnMut(&str, &str) -> f32,
{
    assert_same_labels("eacluster.cpp", 40, input_seqs);
    ea_cluster_clear(ec);
    u_sorter_init(&mut ec.us);
    ec.input_seqs = Some(input_seqs.clone());
    let input_seq_count = input_seqs.seqs.len() as uint;
    assert!(input_seq_count > 0);
    ec.seq_index_to_centroid_index = vec![uint::MAX; input_seq_count as usize];
    let min_ee = 1.0 - min_ea;
    let mut cluster_count = 0_u32;
    let mut member_count = 0_u32;
    for seq_index in 0..input_seq_count {
        let _ = progress_step(
            seq_index,
            input_seq_count,
            &format!(
                "UCLUST {input_seq_count} seqs EE<{min_ee:.2}, {cluster_count} centroids, {member_count} members"
            ),
        );
        let mut best_ea = 0.0;
        let centroid_index =
            ea_cluster_get_best_centroid(ec, seq_index, min_ea, &mut best_ea, |label1, label2| {
                align_seq_pair(label1, label2)
            });
        ec.seq_index_to_centroid_index[seq_index as usize] = centroid_index;
        if centroid_index == uint::MAX {
            let cluster_index = cluster_count;
            cluster_count += 1;
            ec.seq_index_to_centroid_index[seq_index as usize] = cluster_index;
            ec.centroid_seq_indexes.push(seq_index);
            ec.centroid_index_to_seq_indexes.push(vec![seq_index]);

            let byte_seq =
                sequence_get_seq_as_string(&input_seqs.seqs[seq_index as usize]).into_bytes();
            u_sorter_add_seq(&mut ec.us, &byte_seq, seq_index);
        } else {
            member_count += 1;
            assert!((centroid_index as usize) < ec.centroid_index_to_seq_indexes.len());
            assert!((centroid_index as usize) < ec.centroid_seq_indexes.len());
            ec.centroid_index_to_seq_indexes[centroid_index as usize].push(seq_index);
        }
        ea_cluster_validate(ec);
    }
    ea_cluster_make_cluster_mf_as(ec);
}

/// Find the best matching centroid for `seq_index` using the word-index candidate list and `align_seq_pair`; returns `uint::MAX` if none qualifies.
#[track_caller]
pub fn ea_cluster_get_best_centroid<FAlignSeqPair>(
    ec: &EACluster,
    seq_index: uint,
    min_ea: f32,
    best_ea: &mut f32,
    mut align_seq_pair: FAlignSeqPair,
) -> uint
where
    FAlignSeqPair: FnMut(&str, &str) -> f32,
{
    let centroid_count = ec.centroid_seq_indexes.len() as uint;
    if centroid_count == 0 {
        return uint::MAX;
    }

    let input_seqs = ec
        .input_seqs
        .as_ref()
        .expect("EACluster input seqs not set");
    let byte_seq = sequence_get_seq_as_string(&input_seqs.seqs[seq_index as usize]).into_bytes();
    let (top_seq_indexes, top_word_counts) = u_sorter_search_seq(&ec.us, &byte_seq);
    let top_count = top_seq_indexes.len();
    assert_eq!(top_word_counts.len(), top_count);
    if top_count == 0 {
        return uint::MAX;
    }

    *best_ea = 0.0;
    let mut best_centroid_index = uint::MAX;
    let mut done = false;
    for top_index in 0..top_count {
        if done {
            continue;
        }
        let top_seq_index = top_seq_indexes[top_index];
        let label = &input_seqs.seqs[seq_index as usize].label;
        let top_label = &input_seqs.seqs[top_seq_index as usize].label;
        let ea = align_seq_pair(label, top_label);
        if ea > min_ea && ea > *best_ea {
            *best_ea = ea;
            assert!((top_seq_index as usize) < ec.seq_index_to_centroid_index.len());
            let centroid_index = ec.seq_index_to_centroid_index[top_seq_index as usize];
            assert!(centroid_index < centroid_count);
            best_centroid_index = centroid_index;
        }
        if *best_ea >= min_ea {
            if *best_ea > 0.9 {
                done = true;
            }
            if *best_ea - ea > 0.3 {
                done = true;
            }
        }
        if *best_ea < min_ea - 0.3 && top_index > 20 {
            done = true;
        }
    }
    best_centroid_index
}

/// Return a copy of all per-cluster MFAs computed by `ea_cluster_make_cluster_mf_as`.
#[track_caller]
pub fn ea_cluster_get_cluster_mf_as(ec: &EACluster) -> Vec<MultiSequence> {
    let n = ec.cluster_mfas.len();
    let mut mfas = Vec::new();
    for i in 0..n {
        let cluster_mfa = ec.cluster_mfas[i].clone();
        mfas.push(cluster_mfa);
    }
    mfas
}

/// Write each cluster MFA to disk using `file_name_pattern`; returns the generated file names.
#[track_caller]
pub fn ea_cluster_write_mf_as(ec: &EACluster, file_name_pattern: &str) -> Vec<String> {
    let centroid_count = ec.cluster_mfas.len();
    let mut file_names = Vec::new();
    for centroid_index in 0..centroid_count {
        let _ = progress_step(
            centroid_index as uint,
            centroid_count as uint,
            "Write cluster MFAs",
        );
        let mfa = &ec.cluster_mfas[centroid_index];
        let file_name = make_replicate_file_name_n(file_name_pattern, centroid_index as uint + 1);
        let mut out = String::new();
        for seq in &mfa.seqs {
            out.push_str(&seq_to_fasta_l2561(
                &sequence_get_seq_as_string(seq),
                &seq.label,
            ));
        }
        std::fs::write(&file_name, out).unwrap();
        file_names.push(file_name);
    }
    file_names
}

/// Materialize a `MultiSequence` per centroid containing all its member sequences.
#[track_caller]
pub fn ea_cluster_make_cluster_mf_as(ec: &mut EACluster) {
    let centroid_count = ec.centroid_seq_indexes.len();
    ec.cluster_mfas.clear();
    let input_seqs = ec
        .input_seqs
        .as_ref()
        .expect("EACluster input seqs not set");
    for centroid_index in 0..centroid_count {
        let _ = progress_step(
            centroid_index as uint,
            centroid_count as uint,
            "Make cluster MFAs",
        );
        let seq_indexes = &ec.centroid_index_to_seq_indexes[centroid_index];
        let member_count = seq_indexes.len();
        let mut cluster_mfa = MultiSequence::default();
        for seq_index in seq_indexes.iter().take(member_count) {
            let seq = input_seqs.seqs[*seq_index as usize].clone();
            cluster_mfa.seqs.push(seq);
            cluster_mfa.owners.push(false);
        }
        ec.cluster_mfas.push(cluster_mfa);
    }
    let refs = ec.cluster_mfas.iter().collect::<Vec<_>>();
    assert_same_seqs_vec_l91("eacluster.cpp", 198, input_seqs, &refs);
}

/// Default pairwise scorer: align the two labelled sequences with the flat aligner and return the EA score.
#[track_caller]
pub fn ea_cluster_align_seq_pair(label1: &str, label2: &str) -> f32 {
    let (ea, _path) = align_pair_flat(label1, label2);
    ea
}

/// Sanity-check the EACluster index tables (centroid ranges, member<->centroid back-references).
#[track_caller]
pub fn ea_cluster_validate(ec: &EACluster) {
    let input_seqs = ec
        .input_seqs
        .as_ref()
        .expect("EACluster input seqs not set");
    let seq_count = input_seqs.seqs.len() as uint;
    let centroid_count = ec.centroid_seq_indexes.len() as uint;
    assert_eq!(
        ec.centroid_index_to_seq_indexes.len() as uint,
        centroid_count
    );
    for centroid_index in 0..centroid_count {
        let centroid_seq_index = ec.centroid_seq_indexes[centroid_index as usize];
        assert!(centroid_seq_index < seq_count);
        let member_seq_indexes = &ec.centroid_index_to_seq_indexes[centroid_index as usize];
        let member_count = member_seq_indexes.len();
        for member_seq_index in member_seq_indexes.iter().take(member_count) {
            assert!(*member_seq_index < seq_count);
            let centroid_index2 = ec.seq_index_to_centroid_index[*member_seq_index as usize];
            assert_eq!(centroid_index2, centroid_index);
        }
    }
}

/// Driver: load sequences, initialize ProbCons, run EA clustering, and write cluster MFAs.
#[track_caller]
pub fn cmd_eacluster<FAlignSeqPair>(
    input_file_name: &str,
    min_ea: f32,
    output_file_name_pattern: &str,
    mut align_seq_pair: FAlignSeqPair,
) -> (EACluster, Vec<String>)
where
    FAlignSeqPair: FnMut(&str, &str) -> f32,
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    if is_nucleo {
        set_alpha_l209(ALPHA::ALPHA_Nucleo);
    } else {
        set_alpha_l209(ALPHA::ALPHA_Amino);
    }
    init_probcons();

    let mut ec = EACluster::default();
    ea_cluster_run(&mut ec, &input_seqs, min_ea, |label1, label2| {
        align_seq_pair(label1, label2)
    });
    let file_names = ea_cluster_write_mf_as(&ec, output_file_name_pattern);
    (ec, file_names)
}
