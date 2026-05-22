// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Super4 {
    pub target_pair_count: uint,
    pub max_cluster_size: uint,
    pub min_ea_pass1: f32,
    pub min_ea_pass2: f32,
    pub input_seqs: Option<MultiSequence>,
    pub ec: EACluster,
    pub cluster_mfas: Vec<MultiSequence>,
    pub cluster_labels: Vec<String>,
    pub mpc: MPCFlat,
    pub cluster_msas: Vec<MultiSequence>,
    pub consensus_seqs: MultiSequence,
    pub dist_mx: Vec<Vec<f32>>,
    pub pp: PProg,
    pub final_msa: MultiSequence,
    pub guide_tree_none: Tree,
    pub guide_tree_abc: Tree,
    pub guide_tree_acb: Tree,
    pub guide_tree_bca: Tree,
    pub final_msa_none: MultiSequence,
    pub final_msa_abc: MultiSequence,
    pub final_msa_acb: MultiSequence,
    pub final_msa_bca: MultiSequence,
} // original: Super4 (muscle/src/super4.h)

/// Reset the final MSA and all guide-tree permutation slots before a rerun.
#[track_caller]
pub fn super4_clear_trees_and_ms_as(s4: &mut Super4) {
    multi_sequence_clear(&mut s4.final_msa);

    s4.guide_tree_none = Tree::default();
    s4.guide_tree_abc = Tree::default();
    s4.guide_tree_acb = Tree::default();
    s4.guide_tree_bca = Tree::default();

    multi_sequence_clear(&mut s4.final_msa_none);
    multi_sequence_clear(&mut s4.final_msa_abc);
    multi_sequence_clear(&mut s4.final_msa_acb);
    multi_sequence_clear(&mut s4.final_msa_bca);
}

/// Build the unpermuted guide tree from the cluster distance matrix using
/// biased UPGMA.
#[track_caller]
pub fn super4_make_guide_tree<FRunUpgma>(s4: &mut Super4, mut run_upgma: FRunUpgma)
where
    FRunUpgma: FnMut(&mut UPGMA5, &mut Tree),
{
    if s4.cluster_labels.len() == 1 {
        tree_create_rooted(&mut s4.guide_tree_none);
        tree_set_leaf_id(&mut s4.guide_tree_none, 0, 0);
        tree_set_leaf_name(&mut s4.guide_tree_none, 0, &s4.cluster_labels[0]);
        return;
    }

    let mut u = UPGMA5::default();
    upgma5_init(&mut u, &s4.cluster_labels, &s4.dist_mx);
    upgma5_fix_ea_dist_mx(&mut u);
    run_upgma(&mut u, &mut s4.guide_tree_none);
}

/// Split an oversized MFA into chunks of at most `max_size` sequences,
/// in input order.
#[track_caller]
pub fn super4_split_big_mfa_random(
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
    assert_same_seqs_vec_l111("super4.cpp", 63, input_mfa, split_mfas);
}

/// Split an oversize MFA by re-clustering at the stricter `min_ea`
/// threshold, then chopping any still-too-large subclusters.
#[track_caller]
pub fn super4_split_big_mfa<F>(
    s4: &mut Super4,
    big_mfa: &MultiSequence,
    max_size: uint,
    min_ea: f32,
    split_mfas: &mut Vec<MultiSequence>,
    mut run_ea_cluster: F,
) where
    F: FnMut(&mut EACluster, &MultiSequence, f32) -> Vec<MultiSequence>,
{
    split_mfas.clear();
    let input_seq_count = big_mfa.seqs.len() as uint;
    assert!(input_seq_count > max_size);

    let mut cluster_mfas = run_ea_cluster(&mut s4.ec, big_mfa, min_ea);
    assert!(!cluster_mfas.is_empty());
    assert_same_seqs_vec_l111("super4.cpp", 76, big_mfa, &cluster_mfas);

    let mut cluster_index = 0usize;
    while cluster_index < cluster_mfas.len() {
        let seq_count = cluster_mfas[cluster_index].seqs.len() as uint;
        if seq_count > max_size {
            let mut sub_mfas = Vec::new();
            super4_split_big_mfa_random(&cluster_mfas[cluster_index], max_size, &mut sub_mfas);
            assert_same_seqs_vec_l111("super4.cpp", 85, &cluster_mfas[cluster_index], &sub_mfas);
            let n = sub_mfas.len();
            assert!(n > 1);
            cluster_mfas[cluster_index] = sub_mfas[0].clone();
            for sub_mfa in sub_mfas.iter().take(n).skip(1) {
                cluster_mfas.push(sub_mfa.clone());
            }
        }
        cluster_index += 1;
    }

    *split_mfas = cluster_mfas;
    assert_same_seqs_vec_l111("super4.cpp", 98, big_mfa, split_mfas);
}

/// Run pass-1 EA clustering on the input, split any oversize clusters,
/// and assign labels.
#[track_caller]
pub fn super4_cluster_input<F>(s4: &mut Super4, mut run_ea_cluster: F) -> String
where
    F: FnMut(&mut EACluster, &MultiSequence, f32) -> Vec<MultiSequence>,
{
    let input_seqs = s4
        .input_seqs
        .clone()
        .expect("Super4::ClusterInput, m_InputSeqs is null");
    let _input_seq_count = input_seqs.seqs.len() as uint;

    s4.cluster_mfas = run_ea_cluster(&mut s4.ec, &input_seqs, s4.min_ea_pass1);
    assert_same_seqs_vec_l111("super4.cpp", 111, &input_seqs, &s4.cluster_mfas);

    let mut out = String::new();
    let mut cluster_count = s4.cluster_mfas.len() as uint;
    out.push_str(&format!("{cluster_count} clusters pass 1\n"));

    let mut cluster_index = 0usize;
    while cluster_index < s4.cluster_mfas.len() {
        let mfa = s4.cluster_mfas[cluster_index].clone();
        assert_same_labels("super4.cpp", 117, &mfa);
        let seq_count = mfa.seqs.len() as uint;
        assert!(seq_count > 0);
        if seq_count > s4.max_cluster_size {
            let mut split_mfas = Vec::new();
            super4_split_big_mfa(
                s4,
                &mfa,
                s4.max_cluster_size,
                s4.min_ea_pass2,
                &mut split_mfas,
                |ec, big, min_ea| run_ea_cluster(ec, big, min_ea),
            );
            let n = split_mfas.len();
            assert!(n > 1);
            s4.cluster_mfas[cluster_index] = split_mfas[0].clone();
            for split_mfa in split_mfas.iter().take(n).skip(1) {
                s4.cluster_mfas.push(split_mfa.clone());
                assert_same_labels("super4.cpp", 131, split_mfa);
            }
        }
        cluster_index += 1;
    }

    cluster_count = s4.cluster_mfas.len() as uint;
    out.push_str(&format!("{cluster_count} clusters pass 2\n"));
    assert_same_seqs_vec_l111("super4.cpp", 139, &input_seqs, &s4.cluster_mfas);

    s4.cluster_labels.clear();
    for cluster_index in 0..cluster_count {
        let mfa = &s4.cluster_mfas[cluster_index as usize];
        let seq_count = mfa.seqs.len() as uint;
        assert!(seq_count <= s4.max_cluster_size);
        s4.cluster_labels.push(format!("Cluster{cluster_index}"));
    }
    out
}

/// MPC-align each cluster MFA in turn and collect the resulting MSAs.
#[track_caller]
pub fn super4_align_clusters<F>(s4: &mut Super4, mut run_mpc: F) -> String
where
    F: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    s4.cluster_msas.clear();

    let cluster_count = s4.cluster_mfas.len();
    let mut out = String::new();
    for cluster_index in 0..cluster_count {
        let cluster_mfa = s4.cluster_mfas[cluster_index].clone();
        assert_same_labels("super4.cpp", 164, &cluster_mfa);
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

        s4.mpc.tree_perm = TREEPERM::TP_None;
        let cluster_msa = run_mpc(&mut s4.mpc, &cluster_mfa);
        assert_same_labels("super4.cpp", 181, &cluster_msa);
        s4.cluster_msas.push(cluster_msa);
    }
    out
}

/// Free the per-cluster MSAs once they are no longer needed.
#[track_caller]
pub fn super4_delete_cluster_ms_as(s4: &mut Super4) {
    let n = s4.cluster_msas.len();
    for i in 0..n {
        multi_sequence_clear(&mut s4.cluster_msas[i]);
    }
    s4.cluster_msas.clear();
}

/// Compute the consensus sequence of each cluster MSA and stash it in
/// `consensus_seqs` for downstream distance and tree work.
#[track_caller]
pub fn super4_get_consensus_seqs(s4: &mut Super4) {
    multi_sequence_clear(&mut s4.consensus_seqs);
    let cluster_count = s4.cluster_msas.len();
    for cluster_index in 0..cluster_count {
        let cluster_msa = &s4.cluster_msas[cluster_index];
        let label = format!("Cluster{cluster_index}");
        let cons_seq = get_consensus_sequence(cluster_msa);

        let mut seq = Sequence::default();
        sequence_from_string(&mut seq, &label, &cons_seq);
        add_global_tmp_seq(&seq);
        s4.consensus_seqs.seqs.push(seq);
        s4.consensus_seqs.owners.push(true);

        assert!(cluster_index < s4.cluster_labels.len());
        let cluster_label = &s4.cluster_labels[cluster_index];
        let _ = cluster_label;
    }
}

/// Seed the progressive aligner with the per-cluster MSAs and labels.
#[track_caller]
pub fn super4_init_pp(s4: &mut Super4) {
    let n = s4.cluster_msas.len();
    let mut msas = Vec::<MultiSequence>::new();
    for i in 0..n {
        msas.push(s4.cluster_msas[i].clone());
    }

    s4.pp.target_pair_count = s4.target_pair_count;
    p_prog_set_ms_as(&mut s4.pp, &msas, &s4.cluster_labels);
}

/// Apply CLI overrides for Super4 tunables, falling back to defaults.
#[track_caller]
pub fn super4_set_opts(
    s4: &mut Super4,
    pair_count: Option<uint>,
    super4_minea1: Option<f32>,
    super4_minea2: Option<f32>,
    consistency_iter_count: Option<uint>,
    refine_iter_count: Option<uint>,
) {
    s4.target_pair_count = pair_count.unwrap_or(2000);
    s4.max_cluster_size = pair_count.unwrap_or(500);
    s4.min_ea_pass1 = super4_minea1.unwrap_or(0.7);
    s4.min_ea_pass2 = super4_minea2.unwrap_or(0.9);
    s4.mpc.consistency_iter_count = consistency_iter_count.unwrap_or(2);
    s4.mpc.refine_iter_count = refine_iter_count.unwrap_or(100);
}

/// Compute the all-pairs EA distance matrix over the cluster consensus
/// sequences via the supplied closure.
#[track_caller]
pub fn super4_calc_consensus_seqs_dist_mx<FCalcEADistMx>(
    s4: &mut Super4,
    mut calc_ea_dist_mx: FCalcEADistMx,
) where
    FCalcEADistMx: FnMut(&MultiSequence) -> Vec<Vec<f32>>,
{
    s4.dist_mx = calc_ea_dist_mx(&s4.consensus_seqs);
}

/// Coarse-alignment phase: cluster, MSA each cluster, take consensus,
/// build cluster distance matrix and guide tree, prepare progressive align.
#[track_caller]
pub fn super4_coarse_align<FCluster, FMPC, FDist, FGuideTree>(
    s4: &mut Super4,
    mut run_ea_cluster: FCluster,
    mut run_mpc: FMPC,
    mut calc_ea_dist_mx: FDist,
    mut make_guide_tree: FGuideTree,
) -> String
where
    FCluster: FnMut(&mut EACluster, &MultiSequence, f32) -> Vec<MultiSequence>,
    FMPC: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
    FDist: FnMut(&MultiSequence) -> Vec<Vec<f32>>,
    FGuideTree: FnMut(&mut Tree, &[String], &[Vec<f32>]),
{
    let input_seqs = s4
        .input_seqs
        .as_ref()
        .expect("Super4::CoarseAlign requires input_seqs");
    assert_same_labels("super4.cpp", 264, input_seqs);

    let mut out = super4_cluster_input(s4, |ec, big_mfa, min_ea| {
        run_ea_cluster(ec, big_mfa, min_ea)
    });
    out.push_str(&super4_align_clusters(s4, |mpc, cluster_mfa| {
        run_mpc(mpc, cluster_mfa)
    }));
    super4_get_consensus_seqs(s4);
    super4_calc_consensus_seqs_dist_mx(s4, |consensus_seqs| calc_ea_dist_mx(consensus_seqs));
    super4_make_guide_tree(s4, |u, guide_tree| {
        make_guide_tree(guide_tree, &u.labels, &u.dist_mx)
    });
    super4_init_pp(s4);
    out
}

/// Run the full Super4 pipeline; optionally evaluates the ABC/ACB/BCA
/// guide-tree permutations and stores all final MSAs.
#[track_caller]
pub fn super4_run<FSetOpts, FCluster, FMPC, FDist, FGuideTree, FPermuteTree, FPProg>(
    s4: &mut Super4,
    input_seqs: &MultiSequence,
    tree_perm: TREEPERM,
    mut set_opts: FSetOpts,
    mut run_ea_cluster: FCluster,
    mut run_mpc: FMPC,
    mut calc_ea_dist_mx: FDist,
    mut make_guide_tree: FGuideTree,
    mut permute_tree: FPermuteTree,
    mut run_pp_guide_tree: FPProg,
) -> String
where
    FSetOpts: FnMut(&mut Super4),
    FCluster: FnMut(&mut EACluster, &MultiSequence, f32) -> Vec<MultiSequence>,
    FMPC: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
    FDist: FnMut(&MultiSequence) -> Vec<Vec<f32>>,
    FGuideTree: FnMut(&mut Tree, &[String], &[Vec<f32>]),
    FPermuteTree: FnMut(
        &Tree,
        &mut Tree,
        &mut Tree,
        &mut Tree,
        &mut Vec<String>,
        &mut Vec<String>,
        &mut Vec<String>,
    ),
    FPProg: FnMut(&mut PProg, &Tree) -> MultiSequence,
{
    s4.input_seqs = Some(input_seqs.clone());
    set_opts(s4);
    let mut out = super4_coarse_align(
        s4,
        |ec, big_mfa, min_ea| run_ea_cluster(ec, big_mfa, min_ea),
        |mpc, cluster_mfa| run_mpc(mpc, cluster_mfa),
        |consensus_seqs| calc_ea_dist_mx(consensus_seqs),
        |guide_tree, labels, dist_mx| make_guide_tree(guide_tree, labels, dist_mx),
    );

    if tree_perm == TREEPERM::TP_None {
        s4.final_msa = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_none);
        super4_delete_cluster_ms_as(s4);
        return out;
    }

    let mut labels_a = Vec::new();
    let mut labels_b = Vec::new();
    let mut labels_c = Vec::new();
    permute_tree(
        &s4.guide_tree_none,
        &mut s4.guide_tree_abc,
        &mut s4.guide_tree_acb,
        &mut s4.guide_tree_bca,
        &mut labels_a,
        &mut labels_b,
        &mut labels_c,
    );

    match tree_perm {
        TREEPERM::TP_ABC => {
            out.push_str("Guide tree ABC\n");
            s4.final_msa = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_abc);
        }
        TREEPERM::TP_ACB => {
            out.push_str("Guide tree ACB\n");
            s4.final_msa = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_acb);
        }
        TREEPERM::TP_BCA => {
            out.push_str("Guide tree BCA\n");
            s4.final_msa = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_bca);
        }
        TREEPERM::TP_All => {
            out.push_str("Guide tree (default)\n");
            s4.final_msa_none = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_none);

            out.push_str("Guide tree ABC\n");
            s4.final_msa_abc = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_abc);

            out.push_str("Guide tree ACB\n");
            s4.final_msa_acb = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_acb);

            out.push_str("Guide tree BCA\n");
            s4.final_msa_bca = run_pp_guide_tree(&mut s4.pp, &s4.guide_tree_bca);
        }
        TREEPERM::TP_None => unreachable!(),
    }
    super4_delete_cluster_ms_as(s4);
    out
}

/// CLI entry point for `super4`: load input, set alphabet, run the
/// pipeline, and write the final MSA.
#[track_caller]
pub fn cmd_super4<FRunSuper4>(
    input_file_name: &str,
    output_file_name: &str,
    force_nucleo: Option<bool>,
    tree_perm: Option<TREEPERM>,
    mega: bool,
    mut run_super4: FRunSuper4,
) -> (Super4, String)
where
    FRunSuper4: FnMut(&mut Super4, &MultiSequence, TREEPERM) -> String,
{
    if mega {
        die("-super4 does not support -mega, use -super7");
    }

    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);

    let nucleo = force_nucleo.unwrap_or_else(|| multi_sequence_guess_is_nucleo(&input_seqs));
    let tp = tree_perm.unwrap_or(TREEPERM::TP_None);
    set_alpha_l209(if nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    init_probcons();

    let mut s4 = Super4::default();
    let log = run_super4(&mut s4, &input_seqs, tp);
    multi_sequence_write_mfa(&s4.final_msa, output_file_name);
    (s4, log)
}
