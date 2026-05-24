// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Align the input sequences belonging to one shrub using a mega-profile
/// MPC run with tree permutation disabled.
#[track_caller]
pub fn super7_mega_intra_align_shrub<F>(s7m: &mut Super7Mega, shrub_index: uint, mut run_mpc: F)
where
    F: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    let lca = s7m.base.shrub_lcas[shrub_index as usize];
    let mut shrub_input = MultiSequence::default();
    super7_make_shrub_input(&s7m.base, lca, &mut shrub_input);
    let _profile_ptr_vec = super7_mega_get_shrub_profiles(s7m, lca);
    let mpc = s7m
        .base
        .mpc
        .as_mut()
        .expect("Super7_mega::IntraAlignShrub null MPC");
    mpc.tree_perm = TREEPERM::TP_None;
    let shrub_msa = run_mpc(mpc, &shrub_input);
    s7m.base.shrub_msas.push(shrub_msa);
}

/// Collect the mega-profile of every leaf in the subtree rooted at `lca`.
#[track_caller]
pub fn super7_mega_get_shrub_profiles(s7: &Super7Mega, lca: uint) -> Vec<Vec<Vec<byte>>> {
    let mut profile_ptr_vec = Vec::new();
    let guide_tree = s7
        .base
        .guide_tree
        .as_ref()
        .expect("Super7_mega::GetShrubProfiles null guide tree");
    let leaf_nodes = tree_get_subtree_leaf_nodes(guide_tree, lca);
    let n = leaf_nodes.len() as uint;
    assert!(n > 0);
    for node in leaf_nodes {
        assert!((node as usize) < s7.base.node_to_seq_index.len());
        let _seq_index = s7.base.node_to_seq_index[node as usize];
        let label = guide_tree.names[node as usize].clone().unwrap_or_default();
        let ptr_profile = mega_get_profile_by_label(&label);
        profile_ptr_vec.push(ptr_profile);
    }
    profile_ptr_vec
}

/// Drive the Super7 mega pipeline: partition into shrubs, intra-align each
/// shrub with mega profiles, then progressively join them.
#[track_caller]
pub fn super7_mega_run<FMPC, FPProg>(
    s7m: &mut Super7Mega,
    input_seqs: &MultiSequence,
    guide_tree: &Tree,
    shrub_size: uint,
    mut run_mpc: FMPC,
    mut run_pp_guide_tree: FPProg,
) -> String
where
    FMPC: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
    FPProg: FnMut(&mut PProg, &Tree) -> MultiSequence,
{
    if s7m.base.mpc.is_none() {
        s7m.base.mpc = Some(MPCFlat::default());
    }
    s7m.base
        .mpc
        .as_mut()
        .expect("Super7_mega::Run null MPC")
        .d
        .disable = true;

    s7m.base.input_seqs = Some(input_seqs.clone());
    s7m.base.guide_tree = Some(guide_tree.clone());
    super7_map_labels(&mut s7m.base);
    super7_set_shrubs(&mut s7m.base, shrub_size);
    let shrub_count = s7m.base.shrub_lcas.len() as uint;
    if shrub_count == 1 {
        let seq_count = input_seqs.seqs.len() as uint;
        let mega = MEGA_STATE.lock().unwrap();
        for i in 0..seq_count {
            let l = input_seqs.seqs[i as usize].char_vec.len() as uint;
            let profile = &mega.profiles[i as usize];
            let pl = profile.len() as uint;
            assert_eq!(pl, l);
        }
        drop(mega);
        let mpc = s7m.base.mpc.as_mut().expect("Super7_mega::Run null MPC");
        s7m.base.final_msa = run_mpc(mpc, input_seqs);
        String::new()
    } else {
        super7_set_shrub_tree(&mut s7m.base);
        assert!(s7m.base.shrub_msas.is_empty());
        let shrub_count = s7m.base.shrub_lcas.len() as uint;
        let mut out = String::new();
        for shrub_index in 0..shrub_count {
            let msg = format!("Aligning shrub {} / {}\n", shrub_index + 1, shrub_count);
            out.push_str(&progress_log(&msg));
            super7_mega_intra_align_shrub(s7m, shrub_index, |mpc, shrub_input| {
                run_mpc(mpc, shrub_input)
            });
        }
        super7_prog_align(&mut s7m.base, |pp, shrub_tree| {
            run_pp_guide_tree(pp, shrub_tree)
        });
        out
    }
}

/// C++ command entry point parity: `cmd_super7_mega()` begins with
/// `Die("_mega")`, so the exposed command is disabled even though the
/// translated pipeline below is kept available for focused tests/helpers.
#[track_caller]
pub fn cmd_super7_mega(
    _input_file_name: &str,
    _output_file_name: &str,
    _shrub_size: Option<uint>,
    _guide_tree_file_name: Option<&str>,
    _dist_mx_file_name: Option<&str>,
) -> (Super7Mega, Tree, String) {
    die("_mega");
}

/// Translated Super7 mega pipeline body after C++'s disabled command guard.
#[track_caller]
pub fn super7_mega_command_body(
    input_file_name: &str,
    output_file_name: &str,
    shrub_size: Option<uint>,
    guide_tree_file_name: Option<&str>,
    dist_mx_file_name: Option<&str>,
) -> (Super7Mega, Tree, String) {
    let shrub_size = shrub_size.unwrap_or(32);
    if shrub_size < 3 {
        die("-shrub_size must be >= 3");
    }

    let input_seqs = load_input(input_file_name, true);
    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut guide_tree = Tree::default();
    if let Some(file_name) = guide_tree_file_name {
        tree_from_file_l143(&mut guide_tree, file_name);
    } else if let Some(file_name) = dist_mx_file_name {
        let mut u = UPGMA5::default();
        upgma5_read_dist_mx2(&mut u, file_name);
        // C++ super7_mega.cpp:103 uses default `InputIsSimilarity = true`.
        upgma5_scale_dist_mx(&mut u, true);
        upgma5_run_l75(&mut u, "avg", &mut guide_tree);
    } else {
        let _ = calc_guide_tree_sw_blosum62(&input_seqs, &mut guide_tree);
    }

    let _ = init_probcons();
    let mut s7 = Super7Mega::default();
    s7.base.mpc = Some(MPCFlat::default());
    let mut log = super7_mega_run(
        &mut s7,
        &input_seqs,
        &guide_tree,
        shrub_size,
        |mpc, shrub_input| {
            mpc_flat_run(
                mpc,
                shrub_input,
                false,
                |mpc| mpc_flat_calc_posteriors(mpc),
                |mpc| {
                    mpc_flat_calc_guide_tree(mpc, |u, guide_tree| {
                        upgma5_run_l75(u, "biased", guide_tree)
                    })
                },
                |mpc| mpc_flat_consistency_parallel_pairs(mpc),
                |mpc| {
                    mpc_flat_progressive_align(mpc, |mpc, msa1, msa2| {
                        mpc_flat_align_alns(mpc, msa1, msa2).0
                    })
                },
                |mpc| {
                    mpc_flat_refine(
                        mpc,
                        || randu32(),
                        |mpc, msa1, msa2| mpc_flat_align_alns(mpc, msa1, msa2).0,
                    )
                },
            );
            mpc.msa
                .as_ref()
                .expect("cmd_super7_mega missing shrub MSA")
                .clone()
        },
        |pp, shrub_tree| {
            p_prog_run_guide_tree(pp, shrub_tree, |pp, index1, index2| {
                p_prog_align_and_join(pp, index1, index2, |label, msa1, msa2, pair_count, path| {
                    align_ms_as_flat_mega(label, msa1, msa2, pair_count, path)
                })
            });
            p_prog_get_final_msa(pp).clone()
        },
    );
    multi_sequence_write_mfa(&s7.base.final_msa, output_file_name);
    let done_msg = "Done.\n";
    let _ = progress_log(done_msg);
    log.push_str(done_msg);
    (s7, guide_tree, log)
}
