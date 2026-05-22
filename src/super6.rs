// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Super6 {
    pub max_pd_pass1: f64,
    pub max_cluster_size: uint,
    pub target_pair_count_cluster_dist: uint,
    pub target_pair_count: uint,
    pub input_seqs: Option<MultiSequence>,
    pub guide_tree: Tree,
    pub msa: MultiSequence,
    pub ucpd: UClustPD,
    pub cluster_mfas: Vec<MultiSequence>,
    pub cluster_labels: Vec<String>,
    pub mpc: MPCFlat,
    pub pp: PProg,
    pub cluster_msas: Vec<MultiSequence>,
    pub cluster_dist_mx: Vec<Vec<f32>>,
} // original: Super6 (muscle/src/super6.h)

/// Set Super6 tunables (max PD pass1, max cluster size, target pair counts).
#[track_caller]
pub fn super6_set_opts(s6: &mut Super6, super6_maxpd1: Option<f64>) {
    s6.max_pd_pass1 = super6_maxpd1.unwrap_or(1.5);
    s6.max_cluster_size = 500;
    s6.target_pair_count_cluster_dist = 8;
    s6.target_pair_count = 2000;
}

/// Run the full Super6 pipeline: UClust cluster, intra-align clusters,
/// build cluster distance matrix and guide tree, then progressive join.
#[track_caller]
pub fn super6_run<FUClustPD, FDist, FGuideTree, FMPC, FPProg>(
    s6: &mut Super6,
    input_seqs: &MultiSequence,
    mut run_ucpd: FUClustPD,
    mut get_prot_dist_mfa_pair: FDist,
    mut make_guide_tree: FGuideTree,
    mut run_pp_guide_tree: FPProg,
    mut run_mpc: FMPC,
) -> String
where
    FUClustPD: FnMut(&mut UClustPD, &MultiSequence, &[uint], f64) -> Vec<MultiSequence>,
    FDist: FnMut(&MultiSequence, &MultiSequence, uint) -> f32,
    FGuideTree: FnMut(&mut Tree, &[String], &[Vec<f32>]),
    FPProg: FnMut(&mut PProg, &Tree),
    FMPC: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    s6.input_seqs = Some(input_seqs.clone());
    let seq_count = input_seqs.seqs.len() as uint;
    let mut all_seq_indexes = Vec::new();
    for i in 0..seq_count {
        all_seq_indexes.push(i);
    }

    s6.cluster_mfas = run_ucpd(&mut s6.ucpd, input_seqs, &all_seq_indexes, s6.max_pd_pass1);
    assert_same_seqs_vec_l111("super6.cpp", 22, input_seqs, &s6.cluster_mfas);

    let mut out = super6_prepare_clusters(s6);
    super6_calc_cluster_dist_mx(s6, |mfa1, mfa2, target_pair_count| {
        get_prot_dist_mfa_pair(mfa1, mfa2, target_pair_count)
    });
    super6_make_guide_tree(s6, |u, guide_tree| {
        make_guide_tree(guide_tree, &u.labels, &u.dist_mx)
    });
    out.push_str(&super6_align_clusters(s6, |mpc, cluster_mfa| {
        run_mpc(mpc, cluster_mfa)
    }));
    super6_init_pp(s6);
    run_pp_guide_tree(&mut s6.pp, &s6.guide_tree);
    out
}

/// MPC-align each cluster MFA in turn and collect the resulting MSAs.
#[track_caller]
pub fn super6_align_clusters<F>(s6: &mut Super6, mut run_mpc: F) -> String
where
    F: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    s6.cluster_msas.clear();

    let cluster_count = s6.cluster_mfas.len();
    let mut out = String::new();
    for cluster_index in 0..cluster_count {
        let cluster_mfa = s6.cluster_mfas[cluster_index].clone();
        assert_same_labels("super6.cpp", 39, &cluster_mfa);
        let seq_count = cluster_mfa.seqs.len() as uint;

        if seq_count == 1 {
            out.push_str(&format!(
                "Align cluster {} / {} (1 seq)\n",
                cluster_index + 1,
                cluster_count
            ));
        } else {
            out.push('\n');
            out.push_str(&format!(
                "Align cluster {} / {} ({} seqs)\n",
                cluster_index + 1,
                cluster_count,
                seq_count
            ));
            out.push('\n');
        }

        s6.mpc.tree_perm = TREEPERM::TP_None;
        let cluster_msa = run_mpc(&mut s6.mpc, &cluster_mfa);
        assert_same_labels("super6.cpp", 56, &cluster_msa);
        s6.cluster_msas.push(cluster_msa);
    }
    out
}

/// Split an oversized MFA into chunks of at most `max_size` sequences,
/// in input order.
#[track_caller]
pub fn super6_split_big_mfa_random(
    input_mfa: &MultiSequence,
    max_size: uint,
    split_mfas: &mut Vec<MultiSequence>,
) {
    split_mfas.clear();
    let input_seq_count = input_mfa.seqs.len() as uint;
    assert!(input_seq_count > max_size);
    let mut output_seq_count = 0;
    loop {
        assert!(output_seq_count <= input_seq_count);
        let remaining_seq_count = input_seq_count - output_seq_count;
        if remaining_seq_count == 0 {
            break;
        }

        let mut n = remaining_seq_count;
        if n > max_size {
            n = max_size;
        }

        let mut split_mfa = MultiSequence::default();
        for i in 0..n {
            let seq = &input_mfa.seqs[(output_seq_count + i) as usize];
            split_mfa.seqs.push(seq.clone());
            split_mfa.owners.push(false);
        }
        split_mfas.push(split_mfa);
        output_seq_count += n;
    }
    assert_same_seqs_vec_l111("super6.cpp", 91, input_mfa, split_mfas);
}

/// Enforce per-cluster size cap by splitting oversize clusters, then
/// label each cluster.
#[track_caller]
pub fn super6_prepare_clusters(s6: &mut Super6) -> String {
    let mut out = String::new();
    let mut cluster_count = s6.cluster_mfas.len() as uint;
    out.push_str(&format!("{cluster_count} clusters pass 1\n"));
    for cluster_index in 0..cluster_count {
        let seq_count = {
            let mfa = &s6.cluster_mfas[cluster_index as usize];
            assert_same_labels("super6.cpp", 102, mfa);
            mfa.seqs.len() as uint
        };
        assert!(seq_count > 0);
        if seq_count > s6.max_cluster_size {
            let mut split_mfas = Vec::<MultiSequence>::new();
            super6_split_big_mfa_random(
                &s6.cluster_mfas[cluster_index as usize],
                s6.max_cluster_size,
                &mut split_mfas,
            );
            let n = split_mfas.len();
            assert!(n > 1);

            s6.cluster_mfas[cluster_index as usize] = split_mfas[0].clone();
            for split_mfa in split_mfas.iter().take(n).skip(1) {
                s6.cluster_mfas.push(split_mfa.clone());
                assert_same_labels("super6.cpp", 121, split_mfa);
            }
        }
    }

    cluster_count = s6.cluster_mfas.len() as uint;
    out.push_str(&format!("{cluster_count} clusters pass 2\n"));

    let input_seqs = s6
        .input_seqs
        .as_ref()
        .expect("Super6::PrepareClusters requires input_seqs");
    assert_same_seqs_vec_l111("super6.cpp", 128, input_seqs, &s6.cluster_mfas);

    s6.cluster_labels.clear();
    for cluster_index in 0..cluster_count {
        let mfa = &s6.cluster_mfas[cluster_index as usize];
        let seq_count = mfa.seqs.len() as uint;
        assert!(seq_count <= s6.max_cluster_size);
        s6.cluster_labels.push(format!("Cluster{cluster_index}"));
    }
    out
}

/// Fill the symmetric cluster-to-cluster distance matrix using the supplied
/// MFA-pair distance closure.
#[track_caller]
pub fn super6_calc_cluster_dist_mx<F>(s6: &mut Super6, mut get_prot_dist_mfa_pair: F)
where
    F: FnMut(&MultiSequence, &MultiSequence, uint) -> f32,
{
    let cluster_count = s6.cluster_mfas.len();
    s6.cluster_dist_mx.clear();
    s6.cluster_dist_mx = vec![vec![f32::MAX; cluster_count]; cluster_count];
    for i in 0..cluster_count {
        s6.cluster_dist_mx[i][i] = 0.0;
    }

    let pair_count = (cluster_count * (cluster_count - 1)) / 2;
    let mut cluster_index1 = 1usize;
    let mut cluster_index2 = 0usize;
    for _pair_index in 0..pair_count {
        let mfa1 = &s6.cluster_mfas[cluster_index1];
        let mfa2 = &s6.cluster_mfas[cluster_index2];
        let d = get_prot_dist_mfa_pair(mfa1, mfa2, 8);
        s6.cluster_dist_mx[cluster_index1][cluster_index2] = d;
        s6.cluster_dist_mx[cluster_index2][cluster_index1] = d;

        cluster_index2 += 1;
        if cluster_index2 == cluster_index1 {
            cluster_index1 += 1;
            cluster_index2 = 0;
        }
    }
}

/// Build the guide tree over clusters; trivial for a single cluster,
/// otherwise UPGMA on the cluster distance matrix.
#[track_caller]
pub fn super6_make_guide_tree<FRunUpgma>(s6: &mut Super6, mut run_upgma: FRunUpgma)
where
    FRunUpgma: FnMut(&mut UPGMA5, &mut Tree),
{
    if s6.cluster_labels.len() == 1 {
        tree_create_rooted(&mut s6.guide_tree);
        tree_set_leaf_id(&mut s6.guide_tree, 0, 0);
        tree_set_leaf_name(&mut s6.guide_tree, 0, &s6.cluster_labels[0]);
        return;
    }

    let mut u = UPGMA5::default();
    upgma5_init(&mut u, &s6.cluster_labels, &s6.cluster_dist_mx);
    run_upgma(&mut u, &mut s6.guide_tree);
}

/// Seed the progressive-aligner state with the cluster MSAs and labels.
#[track_caller]
pub fn super6_init_pp(s6: &mut Super6) {
    let n = s6.cluster_msas.len();
    let mut msas = Vec::<MultiSequence>::new();
    for i in 0..n {
        msas.push(s6.cluster_msas[i].clone());
    }

    s6.pp.target_pair_count = s6.target_pair_count;
    p_prog_set_ms_as(&mut s6.pp, &msas, &s6.cluster_labels);
}

/// CLI entry point for `super6`: load input, configure alphabet, run the
/// pipeline, and write the final MSA.
#[track_caller]
pub fn cmd_super6<FRunSuper6>(
    input_file_name: &str,
    output_file_name: &str,
    force_nucleo: Option<bool>,
    super6_maxpd1: Option<f64>,
    mut run_super6: FRunSuper6,
) -> (Super6, String)
where
    FRunSuper6: FnMut(&mut Super6, &MultiSequence) -> String,
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);

    if output_file_name.is_empty() {
        die("Must set -output");
    }

    let input_seq_count = get_global_ms_seq_count();
    let _ = input_seq_count;

    let nucleo = force_nucleo.unwrap_or_else(|| multi_sequence_guess_is_nucleo(&input_seqs));
    set_alpha_l209(if nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    init_probcons();
    set_alpha_l209(if nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let mut s6 = Super6::default();
    super6_set_opts(&mut s6, super6_maxpd1);
    let log = run_super6(&mut s6, &input_seqs);
    let final_msa = p_prog_get_final_msa(&s6.pp).clone();
    multi_sequence_write_mfa(&final_msa, output_file_name);
    (s6, log)
}
