// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Normalize a vector of sequence weights so they sum to 1.
pub fn normalize_weights(seq_weights: &mut [f32]) {
    let n = seq_weights.len();
    assert!(n > 0);
    let mut sum = 0.0;
    for &weight in seq_weights.iter() {
        sum += weight;
    }
    for weight in seq_weights {
        *weight /= sum;
    }
}

/// Randomly split the integer range `0..n` into three contiguous index groups.
pub fn split_indexes3(n: uint) -> Vec<Vec<uint>> {
    let mut index_vec = vec![Vec::new(), Vec::new(), Vec::new()];

    let mut ix0 = randu32() % (n - 1);
    let mut ix1 = randu32() % (n - 1);
    if ix1 == ix0 {
        ix1 = (ix1 + 1) % (n - 1);
    }
    if ix0 > ix1 {
        std::mem::swap(&mut ix0, &mut ix1);
    }

    for i in 0..=ix0 {
        index_vec[0].push(i);
    }
    for i in ix0 + 1..=ix1 {
        index_vec[1].push(i);
    }
    for i in ix1 + 1..n {
        index_vec[2].push(i);
    }

    index_vec
}

/// Iteratively refine an MSA by splitting it into three subsets and realigning.
#[track_caller]
pub fn m3_refine<FNwSmall3>(
    input_msa: &MultiSequence,
    ap: &M3AlnParams,
    seq_weights: &[f32],
    refined_msa: &mut MultiSequence,
    mut nw_small3: FNwSmall3,
) -> String
where
    FNwSmall3: FnMut(&mut CacheMem3, &Profile3, &Profile3) -> String,
{
    let subst_mx_letter = &ap.subst_mx_letter;
    let gap_open = ap.gap_open;

    let seq_count = input_msa.seqs.len() as uint;
    let mut current_best_msa = MultiSequence::default();
    multi_sequence_copy(&mut current_best_msa, input_msa);

    let mut prof3 = Profile3::default();
    profile3_from_msa(
        &mut prof3,
        &current_best_msa,
        subst_mx_letter,
        gap_open,
        seq_weights,
    );
    let _current_best_self_score = profile3_get_self_score(&prof3);

    let mut cm = CacheMem3::default();
    let mut out = String::new();
    for _iter in 0..32 {
        let index_vec = split_indexes3(seq_count);

        let sub_msa0 = multi_sequence_project_l3(&current_best_msa, &index_vec[0]);
        let sub_msa1 = multi_sequence_project_l3(&current_best_msa, &index_vec[1]);
        let sub_msa2 = multi_sequence_project_l3(&current_best_msa, &index_vec[2]);

        let mut seq_weights0 = Vec::new();
        let mut seq_weights1 = Vec::new();
        let mut seq_weights2 = Vec::new();

        for &seq_index in &index_vec[0] {
            seq_weights0.push(seq_weights[seq_index as usize]);
        }
        for &seq_index in &index_vec[1] {
            seq_weights1.push(seq_weights[seq_index as usize]);
        }
        for &seq_index in &index_vec[2] {
            seq_weights2.push(seq_weights[seq_index as usize]);
        }

        normalize_weights(&mut seq_weights0);
        normalize_weights(&mut seq_weights1);
        normalize_weights(&mut seq_weights2);

        let mut prof0 = Profile3::default();
        let mut prof1 = Profile3::default();
        let mut prof2 = Profile3::default();
        profile3_from_msa(
            &mut prof0,
            &sub_msa0,
            subst_mx_letter,
            gap_open,
            &seq_weights0,
        );
        profile3_from_msa(
            &mut prof1,
            &sub_msa1,
            subst_mx_letter,
            gap_open,
            &seq_weights1,
        );
        profile3_from_msa(
            &mut prof2,
            &sub_msa2,
            subst_mx_letter,
            gap_open,
            &seq_weights2,
        );

        let path01 = nw_small3(&mut cm, &prof0, &prof1);
        let _path02 = nw_small3(&mut cm, &prof0, &prof2);
        let _path12 = nw_small3(&mut cm, &prof1, &prof2);

        let iter_log = format!("\nPath01={path01}\nPath02={path01}\nPath12={path01}\n");
        log(&iter_log);
        out.push_str(&iter_log);
    }

    let _ = refined_msa;
    out
}

/// Command implementation with injectable internals for focused helper tests.
#[track_caller]
pub fn cmd_m3refine_with_hooks<FRunUpgma, FSetParams, FRefine>(
    input_file_name: &str,
    mut run_upgma: FRunUpgma,
    mut set_params: FSetParams,
    mut refine: FRefine,
) -> (
    MultiSequence,
    Vec<String>,
    Vec<f32>,
    Tree,
    M3AlnParams,
    MultiSequence,
    String,
)
where
    FRunUpgma: FnMut(&mut UPGMA5, &mut Tree),
    FSetParams: FnMut(&mut M3AlnParams),
    FRefine: FnMut(&MultiSequence, &M3AlnParams, &[f32], &mut MultiSequence) -> String,
{
    let mut msa = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa, input_file_name, false);
    assert!(multi_sequence_is_aligned(&msa));
    let seq_count = msa.seqs.len() as uint;

    let mut labels = Vec::new();
    let mut seq_weights = Vec::new();
    let w = 1.0 / seq_count as f32;
    for i in 0..seq_count {
        labels.push(msa.seqs[i as usize].label.clone());
        seq_weights.push(w);
    }

    let dist_mx = get_kimura_dist_mx(&msa);
    let mut u5 = UPGMA5::default();
    let mut t = Tree::default();
    upgma5_init(&mut u5, &labels, &dist_mx);
    run_upgma(&mut u5, &mut t);

    let mut cw = ClustalWeights::default();
    seq_weights = clustal_weights_run(&mut cw, &msa, &t);

    let mut ap = M3AlnParams::default();
    set_params(&mut ap);

    let mut refined_msa = MultiSequence::default();
    let log = refine(&msa, &ap, &seq_weights, &mut refined_msa);
    (msa, labels, seq_weights, t, ap, refined_msa, log)
}

/// Command implementation: load an MSA, build the C++ command guide tree, and run M3 refinement.
#[track_caller]
pub fn cmd_m3refine<FRefine>(
    input_file_name: &str,
    subst_mx_file_name: Option<&str>,
    gap_open: Option<f32>,
    center: Option<f32>,
    blosum_pct: Option<uint>,
    blosum_param_set: Option<uint>,
    perturb_seed: Option<uint>,
    linkage: Option<&str>,
    kmer_dist: Option<&str>,
    tree_iters: Option<uint>,
    refine: FRefine,
) -> (
    MultiSequence,
    Vec<String>,
    Vec<f32>,
    Tree,
    M3AlnParams,
    MultiSequence,
    String,
)
where
    FRefine: FnMut(&MultiSequence, &M3AlnParams, &[f32], &mut MultiSequence) -> String,
{
    cmd_m3refine_with_hooks(
        input_file_name,
        |u5, tree| upgma5_run_l75(u5, "biased", tree),
        |ap| {
            let _ = m3_aln_params_set_from_cmd_line(
                ap,
                false,
                false,
                subst_mx_file_name,
                gap_open,
                center,
                blosum_pct,
                blosum_param_set,
                perturb_seed,
                linkage,
                kmer_dist,
                tree_iters,
            );
        },
        refine,
    )
}
