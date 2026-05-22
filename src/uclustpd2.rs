// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Pick refined centroid candidates per cluster by sampling members and keeping those with most hits.
#[track_caller]
pub fn u_clust_pd_select_candidate_good_centroids<F>(uc: &UClustPD, mut search_all: F) -> Vec<uint>
where
    F: FnMut(&UClustPD, uint) -> uint,
{
    let mut subset_indexes = Vec::new();
    let mut unsorted_selected_indexes = Vec::new();
    let cluster_count = uc.centroid_seq_indexes.len() as uint;
    assert_eq!(
        uc.centroid_index_to_member_subset_indexes.len() as uint,
        cluster_count
    );
    let mut selected_hit_counts = Vec::new();
    for cluster_index in 0..cluster_count {
        let cluster_size = u_clust_pd_get_cluster_size(uc, cluster_index);
        if cluster_size < 8 {
            continue;
        }

        let member_subset_indexes =
            &uc.centroid_index_to_member_subset_indexes[cluster_index as usize];
        let n = (f64::from(cluster_size).log2() / 2.0) as uint;
        assert!(n >= 1);
        let mut sample_set = std::collections::BTreeSet::new();
        for _i in 0..2 * n {
            let r = randu32() % cluster_size;
            let subset_index = member_subset_indexes[r as usize];
            sample_set.insert(subset_index);
            if sample_set.len() as uint == n {
                break;
            }
        }

        let mut top_hit_count = 0;
        let mut top_subset_index = uint::MAX;
        for subset_index in sample_set {
            assert!((subset_index as usize) < uc.subset_seq_indexes.len());
            let _seq_index = uc.subset_seq_indexes[subset_index as usize];
            let hit_count = search_all(uc, subset_index);
            if hit_count > top_hit_count {
                top_hit_count = hit_count;
                top_subset_index = subset_index;
            }
        }
        if top_hit_count > cluster_size {
            unsorted_selected_indexes.push(top_subset_index);
            selected_hit_counts.push(top_hit_count);
        } else {
            let current_centroid =
                uc.centroid_index_to_member_subset_indexes[cluster_index as usize][0];
            unsorted_selected_indexes.push(current_centroid);
            selected_hit_counts.push(cluster_size);
        }
    }
    let n = unsorted_selected_indexes.len();
    // C++ uses unstable QuickSortOrderDesc on SelectedHitCounts
    // (uclustpd2.cpp); match it to keep tie-broken indexes parity-stable.
    let order = quick_sort_order_desc_by(n, |a, b| {
        selected_hit_counts[a].cmp(&selected_hit_counts[b])
    });
    for k in 0..n {
        let i = order[k] as usize;
        let index = unsorted_selected_indexes[i];
        let _hit_count = selected_hit_counts[i];
        subset_indexes.push(index);
    }
    subset_indexes
}

/// CLI entry: two-pass UClustPD that re-seeds with better centroids before the final clustering.
#[track_caller]
pub fn cmd_uclustpd2<FGetProtDistPair, FSearchAll>(
    input_file_name: &str,
    output1_file_name: &str,
    output2_file_name: &str,
    centroids_file_name: &str,
    max_pd: f64,
    thread_count: uint,
    mut get_prot_dist_pair: FGetProtDistPair,
    mut search_all: FSearchAll,
) -> (
    UClustPD,
    Vec<uint>,
    Vec<uint>,
    String,
    String,
    String,
    String,
)
where
    FGetProtDistPair: FnMut(&UClustPD, uint, uint) -> f64,
    FSearchAll: FnMut(&UClustPD, uint) -> uint,
{
    let mut input = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input, input_file_name, true);

    let is_nucleo = multi_sequence_guess_is_nucleo(&input);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let seq_count = input.seqs.len() as uint;
    let mut all_seq_indexes = Vec::new();
    for i in 0..seq_count {
        all_seq_indexes.push(i);
    }

    let mut ud = UClustPD::default();
    let run_log1 = u_clust_pd_run(
        &mut ud,
        &input,
        &all_seq_indexes,
        max_pd,
        thread_count,
        |uc, seq_index, centroids| {
            u_clust_pd_search(uc, seq_index, centroids, |seq_indexi, seq_indexj| {
                get_prot_dist_pair(uc, seq_indexi, seq_indexj)
            })
        },
    );
    let stats1 = u_clust_pd_log_stats(&ud);
    u_clust_pd_to_tsv_l260(&ud, output1_file_name);

    let sis = u_clust_pd_select_candidate_good_centroids(&ud, |uc, subset_index| {
        search_all(uc, subset_index)
    });

    let mut si_set = std::collections::BTreeSet::new();
    let mut all_seq_indexes2 = Vec::new();
    for index in &sis {
        let seq_index = all_seq_indexes[*index as usize];
        assert_eq!(*index, seq_index);
        si_set.insert(*index);
        all_seq_indexes2.push(seq_index);
    }
    for i in 0..seq_count {
        if !si_set.contains(&i) {
            all_seq_indexes2.push(i);
        }
    }

    let run_log2 = u_clust_pd_run(
        &mut ud,
        &input,
        &all_seq_indexes2,
        max_pd,
        thread_count,
        |uc, seq_index, centroids| {
            u_clust_pd_search(uc, seq_index, centroids, |seq_indexi, seq_indexj| {
                get_prot_dist_pair(uc, seq_indexi, seq_indexj)
            })
        },
    );
    let stats2 = u_clust_pd_log_stats(&ud);
    u_clust_pd_to_tsv_l260(&ud, output2_file_name);
    u_clust_pd_centroids_to_fasta(&ud, centroids_file_name);

    (
        ud,
        sis,
        all_seq_indexes2,
        run_log1,
        stats1,
        run_log2,
        stats2,
    )
}
