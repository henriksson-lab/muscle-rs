// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// C++ `DEFAULT_CONSISTENCY_ITERS_FLAT` (mpcflat.h:12).
pub const DEFAULT_CONSISTENCY_ITERS_FLAT: uint = 2;
/// C++ `DEFAULT_REFINE_ITERS_FLAT` (mpcflat.h:13).
pub const DEFAULT_REFINE_ITERS_FLAT: uint = 100;

#[derive(Clone, Debug)]
pub struct MPCFlat {
    pub original_input_seqs: Option<MultiSequence>,
    pub my_input_seqs: Option<MultiSequence>,
    pub msa: Option<MultiSequence>,
    pub consistency_iter_count: uint,
    pub refine_iter_count: uint,
    pub tree_perm: TREEPERM,
    pub labels: Vec<String>,
    pub label_to_index: std::collections::BTreeMap<String, uint>,
    pub guide_tree: Tree,
    pub prog_msas: Vec<Option<MultiSequence>>,
    pub d: Derep,
    pub weights: Vec<f32>,
    pub dist_mx: Vec<Vec<f32>>,
    pub pairs: Vec<(uint, uint)>,
    pub pair_to_index: std::collections::BTreeMap<(uint, uint), uint>,
    pub join_indexes1: Vec<uint>,
    pub join_indexes2: Vec<uint>,
    pub sparse_posts1: Vec<Option<MySparseMx>>,
    pub sparse_posts2: Vec<Option<MySparseMx>>,
} // original: MPCFlat (muscle/src/mpcflat.h)

impl Default for MPCFlat {
    /// Match the C++ struct member initialisers: 2 consistency iters and
    /// 100 refinement iters by default (mpcflat.h:26-27). Rust's automatic
    /// `#[derive(Default)]` would leave these at 0, silently skipping the
    /// consistency and refinement passes that are normally enabled.
    fn default() -> Self {
        Self {
            original_input_seqs: None,
            my_input_seqs: None,
            msa: None,
            consistency_iter_count: DEFAULT_CONSISTENCY_ITERS_FLAT,
            refine_iter_count: DEFAULT_REFINE_ITERS_FLAT,
            tree_perm: TREEPERM::default(),
            labels: Vec::new(),
            label_to_index: std::collections::BTreeMap::new(),
            guide_tree: Tree::default(),
            prog_msas: Vec::new(),
            d: Derep::default(),
            weights: Vec::new(),
            dist_mx: Vec::new(),
            pairs: Vec::new(),
            pair_to_index: std::collections::BTreeMap::new(),
            join_indexes1: Vec::new(),
            join_indexes2: Vec::new(),
            sparse_posts1: Vec::new(),
            sparse_posts2: Vec::new(),
        }
    }
}

/// Reset an MPCFlat to an empty state, freeing per-pair buffers.
#[track_caller]
pub fn mpc_flat_clear(mpc: &mut MPCFlat) {
    mpc.my_input_seqs = None;
    mpc.msa = None;
    mpc.labels.clear();
    mpc.label_to_index.clear();
    mpc.guide_tree = Tree::default();
    mpc.dist_mx.clear();
    mpc.pairs.clear();
    mpc.pair_to_index.clear();
    mpc.join_indexes1.clear();
    mpc.join_indexes2.clear();
    mpc.weights.clear();
    mpc_flat_free_sparse_posts(mpc);
    mpc_flat_free_prog_ms_as(mpc);
}

/// Return the unique-sequence index for a label, panicking if missing.
#[track_caller]
pub fn mpc_flat_get_my_input_seq_index(mpc: &MPCFlat, label: &str) -> uint {
    *mpc.label_to_index
        .get(label)
        .unwrap_or_else(|| panic!("MPCFlat::GetMyInputSeqIndex missing >{label}"))
}

/// Return the label of the input sequence at the given index.
#[track_caller]
pub fn mpc_flat_get_label(mpc: &MPCFlat, seq_index: uint) -> String {
    mpc.my_input_seqs
        .as_ref()
        .expect("MPCFlat::GetLabel, no input seqs")
        .seqs[seq_index as usize]
        .label
        .clone()
}

/// Return the length of the input sequence at the given index.
#[track_caller]
pub fn mpc_flat_get_seq_length(mpc: &MPCFlat, seq_index: uint) -> uint {
    mpc.my_input_seqs
        .as_ref()
        .expect("MPCFlat::GetSeqLength, no input seqs")
        .seqs[seq_index as usize]
        .char_vec
        .len() as uint
}

/// Return a clone of the input sequence at the given index.
#[track_caller]
pub fn mpc_flat_get_sequence(mpc: &MPCFlat, seq_index: uint) -> Sequence {
    mpc.my_input_seqs
        .as_ref()
        .expect("MPCFlat::GetSequence, no input seqs")
        .seqs[seq_index as usize]
        .clone()
}

/// Return the input sequence at the given index as a byte buffer.
#[track_caller]
pub fn mpc_flat_get_byte_ptr(mpc: &MPCFlat, seq_index: uint) -> Vec<byte> {
    sequence_get_seq_as_string(&mpc_flat_get_sequence(mpc, seq_index)).into_bytes()
}

/// Return the linear pair index for an ordered pair of sequence indexes (`smi1 < smi2`).
#[inline(always)]
pub fn mpc_flat_get_pair_index(mpc: &MPCFlat, smi1: uint, smi2: uint) -> uint {
    assert!(smi1 < smi2);
    let seq_count = mpc.labels.len() as uint;
    assert!(smi2 < seq_count);

    let pair_index = smi1 * seq_count - (smi1 * (smi1 + 1)) / 2 + (smi2 - smi1 - 1);

    debug_assert_eq!(
        mpc.pair_to_index.get(&(smi1, smi2)).copied(),
        Some(pair_index),
        "MPCFlat::GetPairIndex formula mismatch for ({smi1},{smi2})",
    );
    pair_index
}

/// Return the ordered sequence index pair for a given pair index.
#[track_caller]
pub fn mpc_flat_get_pair(mpc: &MPCFlat, pair_index: uint) -> (uint, uint) {
    assert!((pair_index as usize) < mpc.pairs.len());
    mpc.pairs[pair_index as usize]
}

/// Ensure the per-pair sparse-posterior buffers are sized for at least `pair_count` pairs.
#[track_caller]
pub fn mpc_flat_alloc_pair_count(mpc: &mut MPCFlat, pair_count: uint) {
    assert!(pair_count > 0);
    if pair_count as usize <= mpc.sparse_posts1.len() {
        return;
    }
    mpc.sparse_posts1.resize(pair_count as usize, None);
    mpc.sparse_posts2.resize(pair_count as usize, None);
}

/// Lazily allocate and return the current sparse posterior for one pair.
#[track_caller]
pub fn mpc_flat_get_sparse_post(mpc: &mut MPCFlat, pair_index: uint) -> &mut MySparseMx {
    assert!((pair_index as usize) < mpc.sparse_posts1.len());
    if mpc.sparse_posts1[pair_index as usize].is_none() {
        mpc.sparse_posts1[pair_index as usize] = Some(MySparseMx::default());
    }
    mpc.sparse_posts1[pair_index as usize].as_mut().unwrap()
}

/// Lazily allocate and return the updated (next-iteration) sparse posterior for one pair.
#[track_caller]
pub fn mpc_flat_get_updated_sparse_post(mpc: &mut MPCFlat, pair_index: uint) -> &mut MySparseMx {
    assert!((pair_index as usize) < mpc.sparse_posts2.len());
    if mpc.sparse_posts2[pair_index as usize].is_none() {
        mpc.sparse_posts2[pair_index as usize] = Some(MySparseMx::default());
    }
    mpc.sparse_posts2[pair_index as usize].as_mut().unwrap()
}

/// Return the number of (unique) input sequences.
#[track_caller]
pub fn mpc_flat_get_seq_count(mpc: &MPCFlat) -> uint {
    mpc.my_input_seqs
        .as_ref()
        .expect("MPCFlat::GetSeqCount, no input seqs")
        .seqs
        .len() as uint
}

/// Initialise input sequences, labels, and label-to-index map (panics on duplicate label).
#[track_caller]
pub fn mpc_flat_init_seqs(mpc: &mut MPCFlat, input_seqs: &MultiSequence) {
    mpc.my_input_seqs = Some(input_seqs.clone());
    let seq_count = mpc_flat_get_seq_count(mpc);

    mpc.labels.clear();
    mpc.label_to_index.clear();
    for i in 0..seq_count {
        let seq = &input_seqs.seqs[i as usize];
        let label = seq.label.clone();
        mpc.labels.push(label.clone());

        if mpc.label_to_index.contains_key(&label) {
            panic!("Duplicate label >{label}");
        }
        mpc.label_to_index.insert(label, i);
    }
}

/// Enumerate every unordered sequence pair and assign each a linear pair index.
#[track_caller]
pub fn mpc_flat_init_pairs(mpc: &mut MPCFlat) {
    let seq_count = mpc_flat_get_seq_count(mpc);
    mpc.pairs.clear();
    mpc.pair_to_index.clear();
    let mut pair_index = 0_u32;
    for seq_index1 in 0..seq_count {
        for seq_index2 in seq_index1 + 1..seq_count {
            let pair = (seq_index1, seq_index2);
            mpc.pairs.push(pair);
            assert!(!mpc.pair_to_index.contains_key(&pair));
            mpc.pair_to_index.insert(pair, pair_index);
            let pair_index2 = mpc_flat_get_pair_index(mpc, seq_index1, seq_index2);
            assert!(pair_index2 == pair_index);
            pair_index += 1;
        }
    }
    let pair_count = (seq_count * (seq_count - 1)) / 2;
    assert!(pair_count == mpc.pairs.len() as uint);
}

/// Initialise the pairwise distance matrix to a fully-disconnected default.
#[track_caller]
pub fn mpc_flat_init_dist_mx(mpc: &mut MPCFlat) {
    let seq_count = mpc_flat_get_seq_count(mpc);
    mpc.dist_mx.clear();
    mpc.dist_mx = vec![vec![f32::MAX; seq_count as usize]; seq_count as usize];
    for i in 0..seq_count as usize {
        mpc.dist_mx[i][i] = 0.0;
    }
}

/// Run the configured number of consistency iterations over all pairs.
#[track_caller]
pub fn mpc_flat_consistency<FConsPair>(mpc: &mut MPCFlat, mut cons_pair: FConsPair)
where
    FConsPair: FnMut(&mut MPCFlat, uint),
{
    let seq_count = mpc_flat_get_seq_count(mpc);
    if seq_count < 3 {
        return;
    }
    for iter in 0..mpc.consistency_iter_count {
        mpc_flat_cons_iter(mpc, iter, |mpc, pair_index| cons_pair(mpc, pair_index));
    }
}

/// Build the guide tree from the current distance matrix using UPGMA5 and permute it.
#[track_caller]
pub fn mpc_flat_calc_guide_tree<FRunUpgma>(mpc: &mut MPCFlat, mut run_upgma: FRunUpgma)
where
    FRunUpgma: FnMut(&mut UPGMA5, &mut Tree),
{
    // C++ mpcflat.cpp:185-189: `-randomchaintree` bypasses UPGMA entirely.
    if randomchaintree_enabled() {
        mpc_flat_calc_guide_tree_random_chain(mpc);
    } else {
        let mut upgma = UPGMA5::default();
        upgma5_init(&mut upgma, &mpc.labels, &mpc.dist_mx);
        upgma5_fix_ea_dist_mx(&mut upgma);
        run_upgma(&mut upgma, &mut mpc.guide_tree);
    }
    perm_tree(&mut mpc.guide_tree, mpc.tree_perm);

    // C++ mpcflat.cpp:196-204: when -guidetreeout is set, write the freshly
    // built (permuted) guide tree and exit before any alignment work.
    if let Some(file_name) = guidetreeout_path() {
        tree_to_file_l13(&mpc.guide_tree, &file_name);
        std::process::exit(0);
    }
}

/// Derive and validate the progressive-alignment join order from the guide tree.
#[track_caller]
pub fn mpc_flat_calc_join_order(mpc: &mut MPCFlat) {
    let label_to_index = mpc
        .label_to_index
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<std::collections::HashMap<_, _>>();
    let (indexes1, indexes2) = get_guide_tree_join_order(&mpc.guide_tree, &label_to_index);
    validate_join_order(&indexes1, &indexes2);
    mpc.join_indexes1 = indexes1;
    mpc.join_indexes2 = indexes2;
}

/// Compute the sparse pair posterior for every sequence pair.
#[track_caller]
pub fn mpc_flat_calc_posteriors(mpc: &mut MPCFlat) {
    let pair_count = mpc.pairs.len() as uint;
    assert!(pair_count > 0);
    let thread_count = get_requested_thread_count().min(pair_count).max(1);
    if thread_count == 1 {
        for pair_index in 0..pair_count {
            mpc_flat_calc_posterior(mpc, pair_index);
        }
        return;
    }

    let input_seqs = mpc
        .my_input_seqs
        .as_ref()
        .expect("MPCFlat::CalcPosteriors, no input seqs");
    let (tx, rx) = std::sync::mpsc::sync_channel(thread_count as usize);
    std::thread::scope(|scope| {
        for thread_index in 0..thread_count {
            let start = (pair_count * thread_index) / thread_count;
            let end = (pair_count * (thread_index + 1)) / thread_count;
            let tx = tx.clone();
            let pairs = &mpc.pairs;
            scope.spawn(move || {
                let mut scratch = PosteriorScratch::default();
                for pair_index in start..end {
                    let pair = pairs[pair_index as usize];
                    let result = mpc_flat_calc_posterior_result_for_pair_with_scratch(
                        pair,
                        input_seqs,
                        &mut scratch,
                    );
                    tx.send((pair_index, result)).unwrap();
                }
            });
        }
        drop(tx);

        for _ in 0..pair_count {
            let (pair_index, (sparse_post, ea, seq_index_x, seq_index_y)) = rx.recv().unwrap();
            mpc.sparse_posts1[pair_index as usize] = Some(sparse_post);
            mpc.dist_mx[seq_index_x as usize][seq_index_y as usize] = ea;
            mpc.dist_mx[seq_index_y as usize][seq_index_x as usize] = ea;
        }
    });
}

/// Run the configured number of randomised refinement iterations.
#[track_caller]
pub fn mpc_flat_refine<FRand, FAlignAlns>(
    mpc: &mut MPCFlat,
    mut rand_value: FRand,
    mut align_alns: FAlignAlns,
) where
    FRand: FnMut() -> uint,
    FAlignAlns: FnMut(&mut MPCFlat, &MultiSequence, &MultiSequence) -> MultiSequence,
{
    let seq_count = mpc_flat_get_seq_count(mpc);
    if seq_count < 3 {
        return;
    }
    for iter in 0..mpc.refine_iter_count {
        let _ = progress_step(iter, mpc.refine_iter_count, "Refining");
        mpc_flat_refine_iter(mpc, &mut rand_value, &mut align_alns);
    }
}

/// Super4 variant: posteriors + consistency + guide tree over a set of consensus seqs.
#[track_caller]
pub fn mpc_flat_run_super4<FCalcPosteriors, FConsistency, FCalcGuideTree>(
    mpc: &mut MPCFlat,
    consensus_seqs: &MultiSequence,
    mut calc_posteriors: FCalcPosteriors,
    mut consistency: FConsistency,
    mut calc_guide_tree: FCalcGuideTree,
) where
    FCalcPosteriors: FnMut(&mut MPCFlat),
    FConsistency: FnMut(&mut MPCFlat),
    FCalcGuideTree: FnMut(&mut MPCFlat),
{
    mpc_flat_clear(mpc);
    let seq_count = consensus_seqs.seqs.len() as uint;
    assert!(seq_count > 1);

    let pair_count = (seq_count * (seq_count - 1)) / 2;
    mpc_flat_alloc_pair_count(mpc, pair_count);
    mpc_flat_init_seqs(mpc, consensus_seqs);
    mpc_flat_init_pairs(mpc);
    mpc_flat_init_dist_mx(mpc);
    calc_posteriors(mpc);
    consistency(mpc);
    calc_guide_tree(mpc);
}

/// Top-level MPCFlat pipeline: dereplicate, posteriors, consistency, progressive align, refine, sort.
#[track_caller]
pub fn mpc_flat_run<FCalcPosteriors, FCalcGuideTree, FConsistency, FProgressiveAlign, FRefine>(
    mpc: &mut MPCFlat,
    original_input_seqs: &MultiSequence,
    input_order: bool,
    mut calc_posteriors: FCalcPosteriors,
    mut calc_guide_tree: FCalcGuideTree,
    mut consistency: FConsistency,
    mut progressive_align: FProgressiveAlign,
    mut refine: FRefine,
) where
    FCalcPosteriors: FnMut(&mut MPCFlat),
    FCalcGuideTree: FnMut(&mut MPCFlat),
    FConsistency: FnMut(&mut MPCFlat),
    FProgressiveAlign: FnMut(&mut MPCFlat),
    FRefine: FnMut(&mut MPCFlat),
{
    mpc_flat_clear(mpc);
    mpc.original_input_seqs = Some(original_input_seqs.clone());

    derep_run(&mut mpc.d, original_input_seqs, true);
    derep_validate(&mpc.d);

    let mut unique_seqs = MultiSequence::default();
    derep_get_unique_seqs(&mpc.d, &mut unique_seqs);
    let seq_count = unique_seqs.seqs.len() as uint;
    if seq_count == 1 {
        mpc.msa = Some(original_input_seqs.clone());
        return;
    }

    let rep_seq_label_to_dupe_labels = derep_get_rep_label_to_dupe_labels(&mpc.d);
    let dupe_count = rep_seq_label_to_dupe_labels.len();

    let pair_count = (seq_count * (seq_count - 1)) / 2;
    mpc_flat_alloc_pair_count(mpc, pair_count);
    mpc_flat_init_seqs(mpc, &unique_seqs);
    mpc_flat_init_pairs(mpc);
    mpc_flat_init_dist_mx(mpc);
    calc_posteriors(mpc);
    calc_guide_tree(mpc);

    let mut cw = ClustalWeights::default();
    mpc.weights = clustal_weights_run(
        &mut cw,
        mpc.my_input_seqs.as_ref().unwrap(),
        &mpc.guide_tree,
    );
    assert_eq!(mpc.weights.len() as uint, seq_count);
    let mut sum = 0.0_f64;
    for i in 0..seq_count {
        let w = mpc.weights[i as usize] * seq_count as f32;
        mpc.weights[i as usize] = w;
        sum += f64::from(w);
        mpc.weights[i as usize] = 1.0_f32;
    }
    assert!(myfeq(sum, f64::from(seq_count)));

    consistency(mpc);
    mpc_flat_calc_join_order(mpc);
    progressive_align(mpc);
    refine(mpc);
    assert!(mpc.msa.is_some());
    mpc_flat_sort_msa(mpc, input_order);

    if dupe_count > 0 {
        mpc_flat_insert_dupes(mpc, &rep_seq_label_to_dupe_labels);
    }
}

/// Sort the produced MSA either by original input order or by guide-tree leaf order.
#[track_caller]
pub fn mpc_flat_sort_msa(mpc: &mut MPCFlat, input_order: bool) {
    set_cmd_opt_used("input_order");
    if input_order {
        mpc_flat_sort_msa_by_input_order(mpc);
    } else {
        set_cmd_opt_used("tree_order");
        mpc_flat_sort_msa_by_guide_tree(mpc);
    }
}

/// Build a label-to-row-index map for the current MSA, panicking on duplicate labels.
#[track_caller]
pub fn mpc_flat_get_label_to_msa_seq_index(
    mpc: &MPCFlat,
) -> std::collections::BTreeMap<String, uint> {
    let mut label_to_msa_seq_index = std::collections::BTreeMap::new();
    let seq_count = mpc_flat_get_seq_count(mpc);
    let msa = mpc
        .msa
        .as_ref()
        .expect("MPCFlat::GetLabelToMSASeqIndex no MSA");
    assert_eq!(msa.seqs.len() as uint, seq_count);
    for i in 0..seq_count {
        let label = msa.seqs[i as usize].label.clone();
        if label_to_msa_seq_index.contains_key(&label) {
            die(&format!("Duplicate label >{label}"));
        }
        label_to_msa_seq_index.insert(label, i);
    }
    label_to_msa_seq_index
}

/// Reorder MSA rows to follow the guide-tree leaf order.
/// Mirrors C++ `MPCFlat::SortMSA_ByGuideTree` (mpcflat.cpp:364) which walks
/// the guide tree depth-first and emits leaves in visit order. The earlier
/// implementation used `tree_get_leaf_labels` (storage order), which produces
/// input order instead of tree-traversal order.
#[track_caller]
pub fn mpc_flat_sort_msa_by_guide_tree(mpc: &mut MPCFlat) {
    let seq_count = mpc_flat_get_seq_count(mpc);
    let label_to_msa_seq_index = mpc_flat_get_label_to_msa_seq_index(mpc);
    let t = &mpc.guide_tree;
    let old_msa = mpc
        .msa
        .as_ref()
        .expect("MPCFlat::SortMSA_ByGuideTree no MSA");
    let mut sorted_seqs = Vec::with_capacity(seq_count as usize);
    let mut node = tree_first_depth_first_node(t);
    while node != uint::MAX {
        let neighbor_count = (t.neighbor1[node as usize] != NULL_NEIGHBOR) as uint
            + (t.neighbor2[node as usize] != NULL_NEIGHBOR) as uint
            + (t.neighbor3[node as usize] != NULL_NEIGHBOR) as uint;
        let is_leaf = t.node_count == 1 || neighbor_count == 1;
        if is_leaf {
            let label = t.names[node as usize].clone().unwrap_or_default();
            let msa_seq_index = *label_to_msa_seq_index
                .get(&label)
                .unwrap_or_else(|| panic!("MPCFlat::SortMSA_ByGuideTree not found >{label}"));
            sorted_seqs.push(old_msa.seqs[msa_seq_index as usize].clone());
        }
        node = tree_next_depth_first_node(t, node);
    }
    assert_eq!(sorted_seqs.len() as uint, seq_count);
    let msa = mpc.msa.as_mut().unwrap();
    for i in 0..seq_count {
        msa.seqs[i as usize] = sorted_seqs[i as usize].clone();
    }
}

/// Reorder MSA rows to match the original input sequence order.
#[track_caller]
pub fn mpc_flat_sort_msa_by_input_order(mpc: &mut MPCFlat) {
    let seq_count = mpc_flat_get_seq_count(mpc);
    let label_to_msa_seq_index = mpc_flat_get_label_to_msa_seq_index(mpc);
    let input = mpc
        .my_input_seqs
        .as_ref()
        .expect("MPCFlat::SortMSA_ByInputOrder no input seqs");
    let old_msa = mpc
        .msa
        .as_ref()
        .expect("MPCFlat::SortMSA_ByInputOrder no MSA");
    let mut sorted_seqs = Vec::with_capacity(seq_count as usize);
    for i in 0..seq_count {
        let label = &input.seqs[i as usize].label;
        let msa_seq_index = *label_to_msa_seq_index
            .get(label)
            .unwrap_or_else(|| panic!("MPCFlat::SortMSA_ByInputOrder(), missing >{label}"));
        sorted_seqs.push(old_msa.seqs[msa_seq_index as usize].clone());
    }
    let msa = mpc.msa.as_mut().unwrap();
    for i in 0..seq_count {
        msa.seqs[i as usize] = sorted_seqs[i as usize].clone();
    }
}

/// Insert duplicate sequences (collapsed by `derep`) back into the final MSA next to their reps.
#[track_caller]
pub fn mpc_flat_insert_dupes(
    mpc: &mut MPCFlat,
    rep_seq_label_to_dupe_labels: &std::collections::BTreeMap<String, Vec<String>>,
) {
    let msa = mpc.msa.as_ref().expect("MPCFlat::InsertDupes no MSA");
    let seq_count = msa.seqs.len();
    let mut updated_msa = MultiSequence::default();
    for seq_index in 0..seq_count {
        let old_seq = &msa.seqs[seq_index];
        let mut new_seq = Sequence::default();
        new_seq.label = old_seq.label.clone();
        new_seq.char_vec = old_seq.char_vec.clone();
        updated_msa.seqs.push(new_seq);
        updated_msa.owners.push(false);

        if let Some(dupe_labels) = rep_seq_label_to_dupe_labels.get(&old_seq.label) {
            for dupe_label in dupe_labels {
                let mut dupe_seq = Sequence::default();
                dupe_seq.label = dupe_label.clone();
                dupe_seq.char_vec = old_seq.char_vec.clone();
                updated_msa.seqs.push(dupe_seq);
                updated_msa.owners.push(true);
            }
        }
    }
    mpc.msa = Some(updated_msa);
}
