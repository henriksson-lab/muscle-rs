// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub struct UClust {
    pub input_seqs: Option<MultiSequence>,
    pub min_ea: f32,
    pub us: USorter,
    pub centroid_seq_indexes: Vec<uint>,
    pub seq_index_to_centroid_seq_index: Vec<uint>,
    pub seq_index_to_path: Vec<String>,
} // original: UClust (muscle/src/uclust.h)

impl Default for UClust {
    fn default() -> Self {
        Self {
            input_seqs: None,
            min_ea: 0.99,
            us: USorter::default(),
            centroid_seq_indexes: Vec::new(),
            seq_index_to_centroid_seq_index: Vec::new(),
            seq_index_to_path: Vec::new(),
        }
    }
}

/// Align two input sequences by index via the supplied pair-alignment callback.
#[track_caller]
pub fn u_clust_align_seq_pair<F>(
    u: &UClust,
    seq_index1: uint,
    seq_index2: uint,
    mut align_pair_flat: F,
) -> (f32, String)
where
    F: FnMut(&str, &str) -> (f32, String),
{
    let input_seqs = u
        .input_seqs
        .as_ref()
        .expect("UClust::AlignSeqPair null input seqs");
    let seq1 = &input_seqs.seqs[seq_index1 as usize];
    let seq2 = &input_seqs.seqs[seq_index2 as usize];
    let label1 = &seq1.label;
    let label2 = &seq2.label;
    align_pair_flat(label1, label2)
}

/// Index the sequence at `seq_index` so future searches can find it as a candidate centroid.
#[track_caller]
pub fn u_clust_add_seq_to_index(u: &mut UClust, seq_index: uint) {
    let input_seqs = u
        .input_seqs
        .as_ref()
        .expect("UClust::AddSeqToIndex null input seqs");
    let seq = &input_seqs.seqs[seq_index as usize];
    let byte_seq = sequence_get_seq_as_string(seq).into_bytes();
    u_sorter_add_seq(&mut u.us, &byte_seq, seq_index);
}

/// Search the index for an accepting centroid for `seq_index`; returns the centroid index and alignment path.
#[track_caller]
pub fn u_clust_search<F>(u: &UClust, seq_index: uint, mut align_pair_flat: F) -> (uint, String)
where
    F: FnMut(&str, &str) -> (f32, String),
{
    let input_seqs = u
        .input_seqs
        .as_ref()
        .expect("UClust::Search null input seqs");
    let seq = &input_seqs.seqs[seq_index as usize];
    let byte_seq = sequence_get_seq_as_string(seq).into_bytes();

    let (top_seq_indexes, top_word_counts) = u_sorter_search_seq(&u.us, &byte_seq);
    let mut top_count = top_seq_indexes.len();
    assert_eq!(top_word_counts.len(), top_count);
    if top_count == 0 {
        return (uint::MAX, String::new());
    }
    const MAX_REJECTS: usize = 8;
    if top_count > MAX_REJECTS {
        top_count = MAX_REJECTS;
    }

    let mut centroid_seq_index = uint::MAX;
    let mut found_path = String::new();
    for top_index in 0..top_count {
        let top_seq_index = top_seq_indexes[top_index];
        let (ea, path) = u_clust_align_seq_pair(u, seq_index, top_seq_index, |label1, label2| {
            align_pair_flat(label1, label2)
        });
        if ea >= u.min_ea {
            centroid_seq_index = top_seq_index;
            found_path = path;
            break;
        }
    }
    (centroid_seq_index, found_path)
}

/// Run the UCLUST loop: assign each input sequence to a centroid or promote it to a new one.
#[track_caller]
pub fn u_clust_run<F>(
    u: &mut UClust,
    input_seqs: &MultiSequence,
    min_ea: f32,
    mut align_pair_flat: F,
) -> String
where
    F: FnMut(&str, &str) -> (f32, String),
{
    u.input_seqs = Some(input_seqs.clone());
    u.min_ea = min_ea;
    u_sorter_init(&mut u.us);

    let input_seq_count = input_seqs.seqs.len() as uint;
    let gsi_count = get_global_ms_seq_count();
    let mut gsi_to_input_seq_index = vec![uint::MAX; gsi_count as usize];
    for seq_index in 0..input_seq_count {
        let label = input_seqs.seqs[seq_index as usize].label.clone();
        let gsi = get_gsi_by_label(&label);
        assert!(gsi < gsi_count);
        assert_eq!(gsi_to_input_seq_index[gsi as usize], uint::MAX);
        gsi_to_input_seq_index[gsi as usize] = seq_index;
    }

    let mut centroid_count = 0;
    let mut member_count = 0;

    u.seq_index_to_centroid_seq_index.clear();
    u.seq_index_to_path.clear();
    u.seq_index_to_centroid_seq_index
        .resize(input_seq_count as usize, uint::MAX);
    u.seq_index_to_path
        .resize(input_seq_count as usize, String::new());

    let order = multi_sequence_get_length_order(input_seqs);
    let mut last_length = uint::MAX;
    let min_ee = 1.0 - u.min_ea;
    let mut out = String::new();
    for k in 0..input_seq_count {
        let seq_index = order[k as usize];
        assert!(seq_index < input_seq_count);
        let l = multi_sequence_get_seq_length(input_seqs, seq_index);
        assert!(l <= last_length);
        last_length = l;

        let progress_msg = format!(
            "UCLUST {input_seq_count} seqs EE<{min_ee:.2}, {centroid_count} centroids, {member_count} members"
        );
        let _ = progress_step(k, input_seq_count, &progress_msg);
        out.push_str(&progress_msg);
        out.push('\n');

        let (mut rep_seq_index, path) = u_clust_search(u, seq_index, |label1, label2| {
            align_pair_flat(label1, label2)
        });
        u.seq_index_to_path[seq_index as usize] = path;
        if rep_seq_index == uint::MAX {
            u.centroid_seq_indexes.push(seq_index);
            u_clust_add_seq_to_index(u, seq_index);
            centroid_count += 1;
            rep_seq_index = seq_index;
            u.seq_index_to_path[seq_index as usize].clear();
        } else {
            member_count += 1;
        }

        u.seq_index_to_centroid_seq_index[seq_index as usize] = rep_seq_index;
    }
    out
}

/// Append the cluster centroid sequences into `centroid_seqs`.
#[track_caller]
pub fn u_clust_get_centroid_seqs(u: &UClust, centroid_seqs: &mut MultiSequence) {
    let input_seqs = u
        .input_seqs
        .as_ref()
        .expect("UClust::GetCentroidSeqs null input seqs");
    let centroid_count = u.centroid_seq_indexes.len();
    for i in 0..centroid_count {
        let seq_index = u.centroid_seq_indexes[i];
        let seq = &input_seqs.seqs[seq_index as usize];
        centroid_seqs.seqs.push(seq.clone());
        centroid_seqs.owners.push(false);
    }
    assert_same_labels("uclust.cpp", 132, centroid_seqs);
}

/// Return centroid/member GSIs and the alignment path from each member to its centroid.
#[track_caller]
pub fn u_clust_get_gs_is(u: &UClust) -> (Vec<uint>, Vec<uint>, Vec<uint>, Vec<String>) {
    let mut centroid_gs_is = Vec::<uint>::new();
    let mut member_gs_is = Vec::<uint>::new();
    let mut member_centroid_gs_is = Vec::<uint>::new();
    let mut gsi_to_member_centroid_path = Vec::<String>::new();

    let input_seqs = u
        .input_seqs
        .as_ref()
        .expect("UClust::GetGSIs null input seqs");
    let input_seq_count = input_seqs.seqs.len();
    let gsi_count = get_gsi_count();

    gsi_to_member_centroid_path.resize(gsi_count as usize, String::new());

    let cluster_count = u.centroid_seq_indexes.len();
    for cluster_index in 0..cluster_count {
        let centroid_seq_index = u.centroid_seq_indexes[cluster_index];
        let seq = &input_seqs.seqs[centroid_seq_index as usize];
        let centroid_gsi = get_gsi_by_label(&seq.label);
        centroid_gs_is.push(centroid_gsi);
    }

    assert_eq!(u.seq_index_to_centroid_seq_index.len(), input_seq_count);
    for member_seq_index in 0..input_seq_count {
        let centroid_seq_index = u.seq_index_to_centroid_seq_index[member_seq_index];
        if centroid_seq_index == member_seq_index as uint {
            continue;
        }

        let path = &u.seq_index_to_path[member_seq_index];
        let member_seq = &input_seqs.seqs[member_seq_index];
        let centroid_seq = &input_seqs.seqs[centroid_seq_index as usize];

        let member_gsi = get_gsi_by_label(&member_seq.label);
        let member_centroid_gsi = get_gsi_by_label(&centroid_seq.label);

        member_gs_is.push(member_gsi);
        member_centroid_gs_is.push(member_centroid_gsi);

        assert!(!path.is_empty());
        gsi_to_member_centroid_path[member_gsi as usize] = path.clone();
    }

    (
        centroid_gs_is,
        member_gs_is,
        member_centroid_gs_is,
        gsi_to_member_centroid_path,
    )
}

/// CLI entry: run UCLUST on `input_file_name` and optionally write centroid FASTA to `output_file_name`.
#[track_caller]
pub fn cmd_uclust<FAlignPairFlat>(
    input_file_name: &str,
    output_file_name: &str,
    min_ea: f32,
    mut align_pair_flat: FAlignPairFlat,
) -> (UClust, MultiSequence)
where
    FAlignPairFlat: FnMut(&str, &str) -> (f32, String),
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

    let mut u = UClust::default();
    u_clust_run(&mut u, &input_seqs, min_ea, |label1, label2| {
        align_pair_flat(label1, label2)
    });

    let mut centroid_seqs = MultiSequence::default();
    u_clust_get_centroid_seqs(&u, &mut centroid_seqs);
    let mut out = String::new();
    for seq in &centroid_seqs.seqs {
        out.push_str(&seq_to_fasta_l2561(
            &sequence_get_seq_as_string(seq),
            &seq.label,
        ));
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write UClust centroids FASTA");
    }
    (u, centroid_seqs)
}
