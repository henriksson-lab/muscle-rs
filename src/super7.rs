// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Super7 {
    pub input_seqs: Option<MultiSequence>,
    pub guide_tree: Option<Tree>,
    pub shrub_tree: Tree,
    pub mpc: Option<MPCFlat>,
    pub shrub_msas: Vec<MultiSequence>,
    pub shrub_labels: Vec<String>,
    pub pp: PProg,
    pub final_msa: MultiSequence,
    pub node_to_seq_index: Vec<uint>,
    pub seq_index_to_node: Vec<uint>,
    pub shrub_lcas: Vec<uint>,
} // original: Super7 (muscle/src/super7.h)

#[derive(Clone, Debug, Default)]
pub struct Super7Mega {
    pub base: Super7,
} // original: Super7_mega (muscle/src/super7.h)

/// Drive the Super7 pipeline: split the guide tree into shrubs, align each
/// shrub with MPC, then progressively join them on the shrub tree.
#[track_caller]
pub fn super7_run<FMPC, FPProg>(
    s7: &mut Super7,
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
    if s7.mpc.is_none() {
        s7.mpc = Some(MPCFlat::default());
    }
    s7.mpc.as_mut().expect("Super7::Run null MPC").d.disable = true;

    s7.input_seqs = Some(input_seqs.clone());
    s7.guide_tree = Some(guide_tree.clone());
    super7_map_labels(s7);
    super7_set_shrubs(s7, shrub_size);
    let shrub_count = s7.shrub_lcas.len() as uint;
    if shrub_count == 1 {
        let mpc = s7.mpc.as_mut().expect("Super7::Run null MPC");
        s7.final_msa = run_mpc(mpc, input_seqs);
        String::new()
    } else {
        super7_set_shrub_tree(s7);
        let out = super7_intra_align_shrubs(s7, |mpc, shrub_input| run_mpc(mpc, shrub_input));
        super7_prog_align(s7, |pp, shrub_tree| run_pp_guide_tree(pp, shrub_tree));
        out
    }
}

/// Progressively align the previously built shrub MSAs along `shrub_tree`.
#[track_caller]
pub fn super7_prog_align<F>(s7: &mut Super7, mut run_pp_guide_tree: F)
where
    F: FnMut(&mut PProg, &Tree) -> MultiSequence,
{
    if s7.pp.target_pair_count == 0 {
        s7.pp.target_pair_count = 2000;
    }
    p_prog_set_ms_as(&mut s7.pp, &s7.shrub_msas, &s7.shrub_labels);
    s7.final_msa = run_pp_guide_tree(&mut s7.pp, &s7.shrub_tree);
}

/// Build `shrub_tree` by pruning the guide tree down to the shrub LCAs.
#[track_caller]
pub fn super7_set_shrub_tree(s7: &mut Super7) {
    let shrub_count = s7.shrub_lcas.len() as uint;
    assert!(shrub_count > 1);
    let guide_tree = s7
        .guide_tree
        .as_ref()
        .expect("Super7::SetShrubTree null guide tree");
    s7.shrub_labels = tree_prune_tree(&mut s7.shrub_tree, guide_tree, &s7.shrub_lcas, "Shrub_");
}

/// Find the LCA nodes that partition the guide tree into shrubs of at most
/// `shrub_size` leaves.
#[track_caller]
pub fn super7_set_shrubs(s7: &mut Super7, shrub_size: uint) {
    let guide_tree = s7
        .guide_tree
        .as_ref()
        .expect("Super7::SetShrubs null guide tree");
    s7.shrub_lcas = get_shrubs(guide_tree, shrub_size);
}

/// Build bidirectional maps between guide-tree leaf nodes and input seq
/// indices, asserting uniqueness of labels.
#[track_caller]
pub fn super7_map_labels(s7: &mut Super7) {
    assert!(s7.input_seqs.is_some());
    assert!(s7.guide_tree.is_some());

    s7.seq_index_to_node.clear();
    s7.node_to_seq_index.clear();

    let seqs = s7.input_seqs.as_ref().unwrap();
    let mut label_to_seq_index = std::collections::BTreeMap::new();
    let seq_count = seqs.seqs.len() as uint;
    for seq_index in 0..seq_count {
        let label = &seqs.seqs[seq_index as usize].label;
        if label_to_seq_index.contains_key(label) {
            panic!("Duplicate label in sequences >{label}");
        }
        label_to_seq_index.insert(label.clone(), seq_index);
    }

    let t = s7.guide_tree.as_ref().unwrap();
    let node_count = t.node_count;
    s7.node_to_seq_index.resize(node_count as usize, uint::MAX);
    s7.seq_index_to_node.resize(seq_count as usize, uint::MAX);
    for node in 0..node_count {
        let i = node as usize;
        let neighbor_count = (t.neighbor1[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[i] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[i] != NULL_NEIGHBOR) as uint;
        if !(t.node_count == 1 || neighbor_count == 1) {
            continue;
        }
        let label = t.names[i].clone().unwrap_or_default();
        let Some(seq_index) = label_to_seq_index.get(&label).copied() else {
            panic!("Tree label not found in sequences >{label}");
        };
        s7.node_to_seq_index[i] = seq_index;
        s7.seq_index_to_node[seq_index as usize] = node;
    }
}

/// Gather the input sequences for the shrub rooted at `lca` into
/// `shrub_input`.
#[track_caller]
pub fn super7_make_shrub_input(s7: &Super7, lca: uint, shrub_input: &mut MultiSequence) {
    shrub_input.seqs.clear();
    shrub_input.owners.clear();

    assert!(s7.input_seqs.is_some());
    assert!(s7.guide_tree.is_some());
    let input = s7.input_seqs.as_ref().unwrap();
    let seq_count = input.seqs.len() as uint;
    let leaf_nodes = tree_get_subtree_leaf_nodes(s7.guide_tree.as_ref().unwrap(), lca);
    let n = leaf_nodes.len() as uint;
    assert!(n > 0);
    for node in leaf_nodes {
        assert!((node as usize) < s7.node_to_seq_index.len());
        let seq_index = s7.node_to_seq_index[node as usize];
        assert!(seq_index < seq_count);

        let s = input.seqs[seq_index as usize].clone();
        shrub_input.seqs.push(s);
        shrub_input.owners.push(false);
    }
}

/// Run MPC on the sequences inside one shrub and append the resulting MSA.
#[track_caller]
pub fn super7_intra_align_shrub<F>(s7: &mut Super7, shrub_index: uint, mut run_mpc: F)
where
    F: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    let lca = s7.shrub_lcas[shrub_index as usize];
    let mut shrub_input = MultiSequence::default();
    super7_make_shrub_input(s7, lca, &mut shrub_input);
    let mpc = s7.mpc.as_mut().expect("Super7::IntraAlignShrub null MPC");
    mpc.tree_perm = TREEPERM::TP_None;
    let shrub_msa = run_mpc(mpc, &shrub_input);
    s7.shrub_msas.push(shrub_msa);
}

/// Intra-align every shrub in order, emitting progress messages.
#[track_caller]
pub fn super7_intra_align_shrubs<F>(s7: &mut Super7, mut run_mpc: F) -> String
where
    F: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    assert!(s7.shrub_msas.is_empty());
    let shrub_count = s7.shrub_lcas.len() as uint;
    let mut out = String::new();
    for shrub_index in 0..shrub_count {
        out.push_str(&format!(
            "Aligning shrub {} / {}\n",
            shrub_index + 1,
            shrub_count
        ));
        super7_intra_align_shrub(s7, shrub_index, |mpc, shrub_input| {
            run_mpc(mpc, shrub_input)
        });
    }
    out
}

/// CLI entry point for `super7`: load input, obtain a guide tree (file,
/// distance matrix or SW BLOSUM62), run the pipeline, and write the MSA.
#[track_caller]
pub fn cmd_super7<FRunUpgma, FCalcGuideTree, FRunSuper7>(
    input_file_name: &str,
    output_file_name: &str,
    shrub_size: Option<uint>,
    guide_tree_file_name: Option<&str>,
    dist_mx_file_name: Option<&str>,
    mega_loaded: bool,
    mut run_upgma: FRunUpgma,
    mut calc_guide_tree_sw_blosum62: FCalcGuideTree,
    mut run_super7: FRunSuper7,
) -> (Super7, Tree, String)
where
    FRunUpgma: FnMut(&mut UPGMA5, &mut Tree),
    FCalcGuideTree: FnMut(&MultiSequence, &mut Tree),
    FRunSuper7: FnMut(&mut Super7, &MultiSequence, &Tree, uint) -> String,
{
    let shrub_size = shrub_size.unwrap_or(32);
    if shrub_size < 3 {
        die("-shrub_size must be >= 3");
    }

    // Match loadinput.cpp:5 — treat both `-mega` and an input path ending in
    // `.mega` as a mega profile; otherwise read FASTA.
    let input_seqs = load_input(input_file_name, mega_loaded);

    let nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    let _ = nucleo;
    let input_seq_count = get_global_ms_seq_count();
    let _ = input_seq_count;

    let mut guide_tree = Tree::default();
    if let Some(file_name) = guide_tree_file_name {
        tree_from_file_l143(&mut guide_tree, file_name);
    } else if let Some(file_name) = dist_mx_file_name {
        let mut u = UPGMA5::default();
        upgma5_read_dist_mx2(&mut u, file_name);
        // C++ super7.cpp:160 uses default `InputIsSimilarity = true`.
        upgma5_scale_dist_mx(&mut u, true);
        run_upgma(&mut u, &mut guide_tree);
    } else {
        if mega_loaded {
            die("Must specify -guidetreein or -distmxin with mega");
        }
        calc_guide_tree_sw_blosum62(&input_seqs, &mut guide_tree);
    }

    set_alpha_l209(ALPHA::ALPHA_Amino);
    init_probcons();

    let mut s7 = Super7 {
        mpc: Some(MPCFlat::default()),
        ..Super7::default()
    };
    let mut log = run_super7(&mut s7, &input_seqs, &guide_tree, shrub_size);
    multi_sequence_write_mfa(&s7.final_msa, output_file_name);
    log.push_str("Done.\n");
    (s7, guide_tree, log)
}
