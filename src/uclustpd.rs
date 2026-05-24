// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct UClustPD {
    pub input_seqs: Option<MultiSequence>,
    pub subset_seq_indexes: Vec<uint>,
    pub max_pd: f64,
    pub thread_count: uint,
    pub pending_subset_indexes: Vec<uint>,
    pub centroid_seq_indexes: Vec<uint>,
    pub centroid_index_to_member_subset_indexes: Vec<Vec<uint>>,
    pub subset_index_to_centroid_index: Vec<uint>,
    pub subset_index_to_dist: Vec<f64>,
} // original: UClustPD (muscle/src/uclustpd.h)

pub(crate) static UCLUST_PD_DP_MEMS: std::sync::Mutex<Vec<XDPMem>> =
    std::sync::Mutex::new(Vec::new());
thread_local! {
    static UCLUST_PD_THREAD_DP_MEM: std::cell::RefCell<XDPMem> =
        std::cell::RefCell::new(XDPMem::default());
}

// One Rust stub per C++ function found by code-complexity-comparator.

fn uclust_pd_ensure_dp_mems(mems: &mut Vec<XDPMem>) {
    if mems.is_empty() {
        let n = get_requested_thread_count();
        assert!(n > 0);
        for _i in 0..n {
            mems.push(XDPMem::default());
        }
    }
}

/// Return the per-thread DP scratch memory, initializing the pool on first use.
#[track_caller]
pub fn get_dp_mem_l17() -> XDPMem {
    let mut mems = UCLUST_PD_DP_MEMS.lock().unwrap();
    uclust_pd_ensure_dp_mems(&mut mems);
    let thread_index = get_thread_index();
    assert!(thread_index < mems.len() as uint);
    mems[thread_index as usize].clone()
}

/// Run `f` with the per-thread DP scratch memory used by C++ `GetDPMem`.
#[track_caller]
pub fn with_dp_mem_l17<R, F>(f: F) -> R
where
    F: FnOnce(&mut XDPMem) -> R,
{
    UCLUST_PD_THREAD_DP_MEM.with(|mem| f(&mut mem.borrow_mut()))
}

/// Return the input sequence label at `seq_index`.
#[track_caller]
pub fn u_clust_pd_get_label(uc: &UClustPD, seq_index: uint) -> String {
    let input = uc.input_seqs.as_ref().expect("UClustPD input seqs not set");
    input.seqs[seq_index as usize].label.clone()
}

/// Return the raw byte sequence and its length for input sequence `seq_index`.
#[track_caller]
pub fn u_clust_pd_get_byte_seq(uc: &UClustPD, seq_index: uint) -> (Vec<byte>, uint) {
    let input = uc.input_seqs.as_ref().expect("UClustPD input seqs not set");
    let seq = &input.seqs[seq_index as usize];
    let bytes: Vec<byte> = sequence_get_seq_as_string(seq).into_bytes();
    let l = bytes.len() as uint;
    (bytes, l)
}

/// Compute the protein distance between two input sequences via Viterbi alignment.
#[track_caller]
pub fn u_clust_pd_get_prot_dist_pair<FViterbi, FDist>(
    uc: &UClustPD,
    seq_indexi: uint,
    seq_indexj: uint,
    path: Option<&mut String>,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> f64
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let (seqi, li) = u_clust_pd_get_byte_seq(uc, seq_indexi);
    let _labeli = u_clust_pd_get_label(uc, seq_indexi);
    let (seqj, lj) = u_clust_pd_get_byte_seq(uc, seq_indexj);
    let _labelj = u_clust_pd_get_label(uc, seq_indexj);

    get_prot_dist_seq_pair(
        &seqi,
        li,
        &seqj,
        lj,
        path,
        |seqi, li, seqj, lj| viterbi_fast_mem(seqi, li, seqj, lj),
        |row_x, row_y, col_count| get_prot_dist(row_x, row_y, col_count),
    )
}

/// Count subset sequences within `max_pd` distance of `seq_index`.
#[track_caller]
pub fn u_clust_pd_search_all<F>(uc: &UClustPD, seq_index: uint, mut get_prot_dist_pair: F) -> uint
where
    F: FnMut(uint, uint) -> f64,
{
    let n = uc.subset_seq_indexes.len() as uint;
    let mut hit_count = 0;
    for i in 0..n {
        let seq_index2 = uc.subset_seq_indexes[i as usize];
        if seq_index2 == seq_index {
            continue;
        }
        let d = get_prot_dist_pair(seq_index, seq_index2);
        if d < uc.max_pd {
            hit_count += 1;
        }
    }
    hit_count
}

/// Find the best matching centroid (within `max_pd`) for `seq_index`; returns its index and distance.
#[track_caller]
pub fn u_clust_pd_search<F>(
    uc: &UClustPD,
    seq_index: uint,
    centroids: &[uint],
    mut get_prot_dist_pair: F,
) -> (uint, f64)
where
    F: FnMut(uint, uint) -> f64,
{
    let n = centroids.len() as uint;
    let mut best_centroid_index = uint::MAX;
    let mut best_dist = f64::from(f32::MAX);
    for i in 0..n {
        let centroid_index = centroids[i as usize];
        assert!((centroid_index as usize) < uc.centroid_seq_indexes.len());
        let centroid_seq_index = uc.centroid_seq_indexes[centroid_index as usize];
        let d = get_prot_dist_pair(seq_index, centroid_seq_index);
        if d > uc.max_pd {
            continue;
        }
        if d < best_dist {
            best_dist = d;
            best_centroid_index = centroid_index;
        }
    }
    (best_centroid_index, best_dist)
}

/// Write the centroid sequences of this UClustPD run to a FASTA file.
#[track_caller]
pub fn u_clust_pd_centroids_to_fasta(uc: &UClustPD, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    let input = uc.input_seqs.as_ref().expect("UClustPD input seqs not set");
    let cluster_count = uc.centroid_seq_indexes.len();
    assert_eq!(uc.centroid_seq_indexes.len(), cluster_count);
    let mut out = String::new();
    for cluster_index in 0..cluster_count {
        let centroid_seq_index = uc.centroid_seq_indexes[cluster_index] as usize;
        let seq = &input.seqs[centroid_seq_index];
        out.push_str(&seq_to_fasta_l2561(
            &sequence_get_seq_as_string(seq),
            &seq.label,
        ));
    }
    std::fs::write(file_name, out).expect("failed to write UClustPD centroids FASTA");
}

/// Run iterative protein-distance clustering: pick seeds, assign members, repeat until done.
#[track_caller]
pub fn u_clust_pd_run<F>(
    uc: &mut UClustPD,
    input_seqs: &MultiSequence,
    subset_seq_indexes: &[uint],
    max_pd: f64,
    thread_count: uint,
    mut search: F,
) -> String
where
    F: FnMut(&UClustPD, uint, &[uint]) -> (uint, f64),
{
    uc.thread_count = 1;
    uc.input_seqs = None;
    uc.subset_seq_indexes.clear();
    uc.max_pd = -1.0;
    uc.pending_subset_indexes.clear();
    uc.centroid_seq_indexes.clear();
    uc.centroid_index_to_member_subset_indexes.clear();
    uc.subset_index_to_centroid_index.clear();
    uc.subset_index_to_dist.clear();

    uc.thread_count = thread_count;
    uc.max_pd = max_pd;
    uc.input_seqs = Some(input_seqs.clone());
    uc.subset_seq_indexes = subset_seq_indexes.to_vec();

    let subset_size = uc.subset_seq_indexes.len() as uint;
    for i in 0..subset_size {
        uc.pending_subset_indexes.push(i);
    }

    uc.subset_index_to_centroid_index
        .resize(subset_size as usize, uint::MAX);
    uc.subset_index_to_dist.resize(subset_size as usize, 0.0);
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let mut iter = 0;
    let mut out = String::new();
    loop {
        iter += 1;
        let mut pending_count = uc.pending_subset_indexes.len() as uint;
        let done_count = subset_size - pending_count;
        let done_pct = if subset_size == 0 {
            0.0
        } else {
            100.0 * f64::from(done_count) / f64::from(subset_size)
        };
        let done_pct_s = format_g3(done_pct);
        out.push_str(&format!(
            "Iter [{iter}] {} clusters, (assigned {done_pct_s}%, remaining {pending_count})\n",
            uc.centroid_seq_indexes.len()
        ));
        let begin_size = uc.pending_subset_indexes.len() as uint;
        if begin_size == 0 {
            break;
        }

        let mut new_seeds = Vec::new();
        let mut done_vec1 = Vec::new();
        for p_index in 0..uc.pending_subset_indexes.len() {
            let subset_index = uc.pending_subset_indexes[p_index];
            assert!((subset_index as usize) < uc.subset_seq_indexes.len());
            let seq_index = uc.subset_seq_indexes[subset_index as usize];

            let (centroid_index, _d) = search(uc, seq_index, &new_seeds);
            if centroid_index == uint::MAX {
                let centroid_index = uc.centroid_seq_indexes.len() as uint;
                new_seeds.push(centroid_index);
                uc.subset_index_to_centroid_index[subset_index as usize] = centroid_index;
                uc.subset_index_to_dist[subset_index as usize] = 0.0;
                uc.centroid_seq_indexes.push(seq_index);

                uc.centroid_index_to_member_subset_indexes
                    .push(vec![subset_index]);
                assert_eq!(
                    uc.centroid_seq_indexes.len(),
                    uc.centroid_index_to_member_subset_indexes.len()
                );
                done_vec1.push(p_index);
            }
            if new_seeds.len() as uint >= uc.thread_count {
                break;
            }
        }
        assert!(!done_vec1.is_empty());
        if done_vec1.len() == uc.pending_subset_indexes.len() {
            uc.pending_subset_indexes.clear();
            break;
        }
        for i in done_vec1.iter().rev() {
            uc.pending_subset_indexes.remove(*i);
        }

        pending_count = uc.pending_subset_indexes.len() as uint;
        if pending_count == 0 {
            break;
        }

        let mut new_hit_count = 0;
        let mut done_vec2 = Vec::new();
        for p_index in 0..uc.pending_subset_indexes.len() {
            let subset_index = uc.pending_subset_indexes[p_index];
            assert!((subset_index as usize) < uc.subset_seq_indexes.len());
            let seq_index = uc.subset_seq_indexes[subset_index as usize];

            let (centroid_index, d) = search(uc, seq_index, &new_seeds);
            if centroid_index != uint::MAX {
                assert!((centroid_index as usize) < uc.centroid_seq_indexes.len());
                let centroid_seq_index = uc.centroid_seq_indexes[centroid_index as usize];
                let _label_q = u_clust_pd_get_label(uc, seq_index);
                let _label_c = u_clust_pd_get_label(uc, centroid_seq_index);
                uc.subset_index_to_centroid_index[subset_index as usize] = centroid_index;
                uc.subset_index_to_dist[subset_index as usize] = d;
                assert!(
                    (centroid_index as usize) < uc.centroid_index_to_member_subset_indexes.len()
                );
                uc.centroid_index_to_member_subset_indexes[centroid_index as usize]
                    .push(subset_index);
                new_hit_count += 1;
                done_vec2.push(p_index);
            }
        }
        let _ = new_hit_count;

        if done_vec2.len() == uc.pending_subset_indexes.len() {
            uc.pending_subset_indexes.clear();
            break;
        }
        for i in done_vec2.iter().rev() {
            uc.pending_subset_indexes.remove(*i);
        }
        let end_size = uc.pending_subset_indexes.len() as uint;
        assert!(end_size < begin_size);
    }
    out
}

/// Return the number of members assigned to the given cluster.
#[track_caller]
pub fn u_clust_pd_get_cluster_size(uc: &UClustPD, cluster_index: uint) -> uint {
    assert!((cluster_index as usize) < uc.centroid_index_to_member_subset_indexes.len());
    uc.centroid_index_to_member_subset_indexes[cluster_index as usize].len() as uint
}

/// Write the cluster-membership TSV to `file_name`.
#[track_caller]
pub fn u_clust_pd_to_tsv_l260(uc: &UClustPD, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    std::fs::write(file_name, u_clust_pd_to_tsv_l269(uc)).expect("failed to write UClustPD TSV");
}

/// Format cluster membership as a tab-separated `<centroid_index>\t<label>` table.
#[track_caller]
pub fn u_clust_pd_to_tsv_l269(uc: &UClustPD) -> String {
    let centroid_count = uc.centroid_seq_indexes.len();
    assert_eq!(
        uc.centroid_index_to_member_subset_indexes.len(),
        centroid_count
    );
    let subset_size = uc.subset_seq_indexes.len();
    assert_eq!(uc.subset_index_to_centroid_index.len(), subset_size);

    let mut out = String::new();
    let mut done_count = 0usize;
    for centroid_index in 0..centroid_count {
        let centroid_seq_index = uc.centroid_seq_indexes[centroid_index];
        let _centroid_label = u_clust_pd_get_label(uc, centroid_seq_index);
        let member_subset_indexes = &uc.centroid_index_to_member_subset_indexes[centroid_index];
        let n = member_subset_indexes.len();
        assert!(n > 0);
        for subset_index in member_subset_indexes {
            let subset_index = *subset_index as usize;
            let seq_index = uc.subset_seq_indexes[subset_index];
            assert!(subset_index < uc.subset_index_to_dist.len());
            let _d = uc.subset_index_to_dist[subset_index];
            let label = u_clust_pd_get_label(uc, seq_index);
            out.push_str(&format!("{centroid_index}\t{label}\n"));
        }
        done_count += n;
    }
    assert_eq!(done_count, subset_size);
    out
}

/// Populate `mfa` with the member sequences of the given cluster.
#[track_caller]
pub fn u_clust_pd_get_cluster_mfa(uc: &UClustPD, cluster_index: uint, mfa: &mut MultiSequence) {
    multi_sequence_clear(mfa);
    let input = uc.input_seqs.as_ref().expect("UClustPD input seqs not set");
    let cluster_size = u_clust_pd_get_cluster_size(uc, cluster_index);
    assert!((cluster_index as usize) < uc.centroid_index_to_member_subset_indexes.len());
    let subset_indexes = &uc.centroid_index_to_member_subset_indexes[cluster_index as usize];
    assert_eq!(subset_indexes.len() as uint, cluster_size);
    for i in 0..cluster_size {
        let subset_index = subset_indexes[i as usize] as usize;
        assert!(subset_index < uc.subset_seq_indexes.len());
        let seq_index = uc.subset_seq_indexes[subset_index] as usize;
        mfa.seqs.push(sequence_clone(&input.seqs[seq_index]));
        mfa.owners.push(false);
    }
}

/// Return one MFA per cluster containing its member sequences.
#[track_caller]
pub fn u_clust_pd_get_cluster_mf_as(uc: &UClustPD) -> Vec<MultiSequence> {
    let mut mfas = Vec::new();
    let cluster_count = uc.centroid_seq_indexes.len() as uint;
    for cluster_index in 0..cluster_count {
        let mut cluster_mfa = MultiSequence::default();
        u_clust_pd_get_cluster_mfa(uc, cluster_index, &mut cluster_mfa);
        mfas.push(cluster_mfa);
    }
    mfas
}

/// Return the size of every cluster.
#[track_caller]
pub fn u_clust_pd_get_cluster_sizes(uc: &UClustPD) -> Vec<uint> {
    let mut sizes = Vec::new();
    let cluster_count = uc.centroid_seq_indexes.len() as uint;
    for i in 0..cluster_count {
        sizes.push(u_clust_pd_get_cluster_size(uc, i));
    }
    sizes
}

/// Produce a human-readable summary of cluster count, average/median size, and largest clusters.
#[track_caller]
pub fn u_clust_pd_log_stats(uc: &UClustPD) -> String {
    let cluster_count = uc.centroid_seq_indexes.len() as uint;
    let subset_size = uc.subset_seq_indexes.len() as uint;
    let avg_size = (subset_size / cluster_count) as f64;
    let cluster_sizes = u_clust_pd_get_cluster_sizes(uc);
    let mut singleton_count = 0;
    for i in 0..cluster_count {
        if cluster_sizes[i as usize] == 1 {
            singleton_count += 1;
        }
    }

    let mut order: Vec<uint> = (0..cluster_count).collect();
    if cluster_count > 0 {
        let mut stack = vec![(0_i32, cluster_count as i32 - 1)];
        while let Some((left, right)) = stack.pop() {
            let mut i = left;
            let mut j = right;
            let mid = (left + right) / 2;
            let pivot = cluster_sizes[order[mid as usize] as usize];
            while i <= j {
                while cluster_sizes[order[i as usize] as usize] > pivot {
                    i += 1;
                }
                while cluster_sizes[order[j as usize] as usize] < pivot {
                    j -= 1;
                }
                if i <= j {
                    order.swap(i as usize, j as usize);
                    i += 1;
                    j -= 1;
                }
            }
            if left < j {
                stack.push((left, j));
            }
            if i < right {
                stack.push((i, right));
            }
        }
    }
    let median_size = cluster_sizes[order[(cluster_count / 2) as usize] as usize];

    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!(
        "{} seqs, {} clusters, avg size {:.1}, median {}, singletons {}\n",
        subset_size, cluster_count, avg_size, median_size, singleton_count
    ));
    for i in 0..std::cmp::min(10, cluster_count) {
        let k = order[i as usize];
        out.push_str(&format!(
            "    Cluster  [{:5}]   size {}\n",
            k, cluster_sizes[k as usize]
        ));
    }
    out
}

/// CLI entry: cluster `input_file_name` by protein distance and emit a TSV plus stats.
#[track_caller]
pub fn cmd_uclustpd<FGetProtDistPair>(
    input_file_name: &str,
    tsv_out_file_name: &str,
    max_pd: f64,
    thread_count: uint,
    mut get_prot_dist_pair: FGetProtDistPair,
) -> (UClustPD, String, String)
where
    FGetProtDistPair: FnMut(&UClustPD, uint, uint) -> f64,
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
    let run_log = u_clust_pd_run(
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

    u_clust_pd_to_tsv_l260(&ud, tsv_out_file_name);
    let stats = u_clust_pd_log_stats(&ud);
    (ud, run_log, stats)
}
