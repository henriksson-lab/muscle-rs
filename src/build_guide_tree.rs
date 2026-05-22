// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `build_guide_tree` subcommand: clusters input sequences with UClustPD to construct a guide tree
/// (stub mirroring the original C++ which is `#if 0`'d out).
#[track_caller]
pub fn cmd_build_guide_tree() -> String {
    let input_file_name = String::new();
    let tree_file_name = String::new();
    let max_pd = 0.0_f64;
    let mut input = MultiSequence::default();
    let seq_count = input.seqs.len() as uint;
    let is_nucleo = if input.seqs.is_empty() {
        false
    } else {
        multi_sequence_guess_is_nucleo(&input)
    };
    if !input.seqs.is_empty() {
        set_alphab(is_nucleo);
    }

    let mut all_seq_indexes = Vec::<uint>::new();
    for i in 0..seq_count {
        all_seq_indexes.push(i);
    }

    let mut ud = UClustPD::default();
    ud.input_seqs = Some(std::mem::take(&mut input));
    ud.subset_seq_indexes = all_seq_indexes;
    ud.max_pd = max_pd;

    let max_cluster_size = 512_u32;
    let cluster_count = ud.centroid_index_to_member_subset_indexes.len() as uint;
    let mut ucs = Vec::<Option<UClustPD>>::new();
    ucs.resize_with(cluster_count as usize, || None);
    for cluster_index in 0..cluster_count {
        let cluster_size = u_clust_pd_get_cluster_size(&ud, cluster_index);
        if cluster_size <= max_cluster_size {
            let mut sub_uc = UClustPD::default();
            let cluster_seq_indexes =
                ud.centroid_index_to_member_subset_indexes[cluster_index as usize].clone();
            sub_uc.input_seqs = ud.input_seqs.clone();
            sub_uc.subset_seq_indexes = cluster_seq_indexes;
            sub_uc.max_pd = max_pd / 2.0;
            ucs[cluster_index as usize] = Some(sub_uc);
        }
    }

    assert!(input_file_name.is_empty());
    assert!(tree_file_name.is_empty());
    String::new()
}
