// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Literal C++ command path: do not install global input before `Muscle3::Run`.
#[track_caller]
pub fn cmd_m3select(
    input_file_name: &str,
    output_file_name: &str,
    replicates: Option<uint>,
    threads: Option<uint>,
) -> String {
    with_quiet(true, || {
        let _global_input_guard = ScopedEmptyGlobalInput::new();
        cmd_m3select_quiet(
            input_file_name,
            output_file_name,
            replicates,
            threads,
            false,
        )
    })
}

#[track_caller]
pub fn cmd_m3select_reusable(
    input_file_name: &str,
    output_file_name: &str,
    replicates: Option<uint>,
    threads: Option<uint>,
) -> String {
    with_quiet(true, || {
        cmd_m3select_quiet(input_file_name, output_file_name, replicates, threads, true)
    })
}

struct ScopedEmptyGlobalInput {
    saved: GlobalInputState,
}

impl ScopedEmptyGlobalInput {
    fn new() -> Self {
        let mut state = global_input_state_lock();
        let saved = state.clone();
        *state = GlobalInputState::default();
        Self { saved }
    }
}

impl Drop for ScopedEmptyGlobalInput {
    fn drop(&mut self) {
        let mut state = global_input_state_lock();
        *state = self.saved.clone();
    }
}

#[track_caller]
fn cmd_m3select_quiet(
    input_file_name: &str,
    output_file_name: &str,
    replicates: Option<uint>,
    threads: Option<uint>,
    install_global_input: bool,
) -> String {
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    if install_global_input {
        set_global_input_ms(&input_seqs);
    }

    let mut master_ap = M3AlnParams {
        linkage: "min".to_string(),
        tree_iters: 1,
        kmer_dist: "66".to_string(),
        ..M3AlnParams::default()
    };
    m3_aln_params_set_blosum(&mut master_ap, 62, 0, f32::MAX, f32::MAX, 0, 0.0, 0.0, 0.0);
    master_ap.linkage = "min".to_string();
    master_ap.tree_iters = 1;
    master_ap.kmer_dist = "66".to_string();
    let master_subst_mx_letter = master_ap.subst_mx_letter;
    let master_gap_open = master_ap.gap_open;

    let delta = 0.1_f32;
    let replicates = replicates.unwrap_or(64);
    assert!(replicates > 0);
    let thread_count = threads.unwrap_or_else(get_requested_thread_count).max(1);

    let next_replicate = std::sync::atomic::AtomicU32::new(0);
    let best = std::sync::Mutex::new((0.0_f32, None::<MultiSequence>));
    std::thread::scope(|scope| {
        for _thread_index in 0..thread_count {
            let next_replicate = &next_replicate;
            let best = &best;
            let input_seqs = &input_seqs;
            let master_subst_mx_letter = &master_subst_mx_letter;
            scope.spawn(move || {
                loop {
                    let ii = next_replicate.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if ii >= replicates {
                        break;
                    }
                    let (final_msa, self_score) = m3select_run_replicate(
                        input_seqs,
                        master_subst_mx_letter,
                        master_gap_open,
                        delta,
                        ii,
                    );
                    let mut best = best.lock().unwrap();
                    if self_score > best.0 {
                        *best = (self_score, Some(final_msa));
                    }
                }
            });
        }
    });
    let (_best_self_score, best_msa) = best.into_inner().unwrap();
    let best_msa = best_msa.expect("BestMSA != 0");
    let out = msa_to_fasta_file_l112(&best_msa);
    if !output_file_name.is_empty() {
        multi_sequence_write_mfa(&best_msa, output_file_name);
    }
    out
}

#[track_caller]
fn m3select_run_replicate(
    input_seqs: &MultiSequence,
    master_subst_mx_letter: &[[f32; 20]; 20],
    master_gap_open: f32,
    delta: f32,
    perturb_seed: uint,
) -> (MultiSequence, f32) {
    let param_group = 0;
    let pct_id = 62;

    let perturb_subst_mx_delta = 0.0_f32;
    let perturb_gap_params_delta = 0.0_f32;
    let perturb_dist_mx_delta = delta;

    let mut ap = M3AlnParams {
        linkage: "min".to_string(),
        tree_iters: 1,
        kmer_dist: "66".to_string(),
        ..M3AlnParams::default()
    };
    m3_aln_params_set_blosum(
        &mut ap,
        pct_id,
        param_group,
        f32::MAX,
        f32::MAX,
        perturb_seed,
        perturb_subst_mx_delta,
        perturb_gap_params_delta,
        perturb_dist_mx_delta,
    );
    ap.linkage = "min".to_string();
    ap.tree_iters = 1;
    ap.kmer_dist = "66".to_string();

    let mut m3 = Muscle3::default();
    let final_msa = muscle3_run(
        &mut m3,
        &ap,
        input_seqs,
        |u, linkage, tree| upgma5_run_l75(u, linkage, tree),
        |pp, input, weights, tree| {
            p_prog3_run(
                pp,
                input,
                weights,
                tree,
                |cm, prof_a, prof_b| nw_small3(cm, prof_a, prof_b).1,
                |prof_a, weight_a, prof_b, weight_b, subst_mx_letter, gap_open, path| {
                    align_two_profs_given_path(
                        prof_a,
                        weight_a,
                        prof_b,
                        weight_b,
                        subst_mx_letter,
                        gap_open,
                        path,
                    )
                },
            );
            pp.msa.clone()
        },
    );
    let mut prof = Profile3::default();
    profile3_from_msa(
        &mut prof,
        &final_msa,
        master_subst_mx_letter,
        master_gap_open,
        &m3.input_seq_weights,
    );
    let self_score = profile3_get_self_score(&prof);
    (final_msa, self_score)
}
