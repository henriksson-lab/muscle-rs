// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Substitutes the `@` placeholder in `pattern` with `<TP>.<seed>` to produce a per-replicate file name.
pub fn make_replicate_file_name(pattern: &str, tp: TREEPERM, perturb_seed: uint) -> String {
    let pos = pattern
        .find('@')
        .unwrap_or_else(|| panic!("'@' not found in '{}'", pattern));
    let mut file_name = String::new();
    file_name.push_str(&pattern[..pos]);
    file_name.push_str(&format!("{}.{}", treeperm_to_str(tp), perturb_seed));
    file_name.push_str(&pattern[pos + 1..]);
    file_name
}

/// Runs a single MPC alignment replicate (with optional probability perturbation) and returns the MSA in MFA format.
#[track_caller]
pub fn align<FCmdLineUpdate, FRunMpc>(
    m: &mut MPCFlat,
    input_seqs: &MultiSequence,
    perturb_seed: uint,
    tp: TREEPERM,
    write_efa_hdr: bool,
    output_enabled: bool,
    mut cmd_line_update: FCmdLineUpdate,
    mut run_mpc: FRunMpc,
) -> (HMMParams, String)
where
    FCmdLineUpdate: FnMut(&mut HMMParams),
    FRunMpc: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
{
    let nucleo = ALPHA_STATE.lock().unwrap().alpha == ALPHA::ALPHA_Nucleo;
    // Mirror C++ setprobconsparams.cpp:13-19: `-hmmin` replaces the built-in
    // defaults *before* CmdLineUpdate / perturb / publish-to-pair-hmm.
    let mut hp = if let Some(path) = HMMIN_PATH.lock().unwrap().clone() {
        hmm_params_from_file(&path)
    } else {
        hmm_params_from_defaults(nucleo)
    };
    if !output_enabled {
        return (hp, String::new());
    }

    cmd_line_update(&mut hp);
    if perturb_seed > 0 {
        reset_rand(perturb_seed);
        hmm_params_perturb_probs(&mut hp, perturb_seed);
    }
    // C++ setprobconsparams.cpp:37-38: optional dump of (possibly perturbed)
    // params before publishing them to the global pair-HMM tables.
    if let Some(path) = HMMOUT_PATH.lock().unwrap().clone() {
        let _ = hmm_params_to_file(&hp, &path);
    }
    hmm_params_to_pair_hmm(&hp);

    m.tree_perm = tp;
    let msa = run_mpc(m, input_seqs);
    m.msa = Some(msa);
    let msa = m.msa.as_ref().expect("Align missing MSA");

    let mut out = String::new();
    if write_efa_hdr {
        out.push_str(&format!("<{}.{}\n", treeperm_to_str(tp), perturb_seed));
    }
    out.push_str(&msa_to_fasta_file_l112(msa));
    (hp, out)
}

/// `align` subcommand: orchestrates MUSCLE alignment including replicate, stratified and Super5 modes.
#[track_caller]
pub fn cmd_align<FCmdLineUpdate, FRunMpc, FRunSuper5>(
    input_file_name: &str,
    output_pattern: &str,
    minsuper: Option<uint>,
    consistency_iter_count: Option<uint>,
    refine_iter_count: Option<uint>,
    stratified: bool,
    diversified: bool,
    replicates: Option<uint>,
    perturb_seed: Option<uint>,
    tree_perm: Option<TREEPERM>,
    mut cmd_line_update: FCmdLineUpdate,
    mut run_mpc: FRunMpc,
    mut run_super5: FRunSuper5,
) -> (MPCFlat, Vec<String>, String)
where
    FCmdLineUpdate: FnMut(&mut HMMParams),
    FRunMpc: FnMut(&mut MPCFlat, &MultiSequence) -> MultiSequence,
    FRunSuper5: FnMut(uint) -> (Vec<String>, String),
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);
    set_global_input_ms(&input_seqs);
    let input_seq_count = input_seqs.seqs.len() as uint;

    // Match C++ align.cpp: an empty input is logged and the command exits
    // without producing an output file. Rust used to panic in mpc_flat_run.
    if input_seq_count == 0 {
        return (MPCFlat::default(), Vec::new(), String::new());
    }

    if let Some(minsuper) = minsuper {
        if input_seq_count >= minsuper {
            multi_sequence_clear(&mut input_seqs);
            let mut out = format!("{input_seq_count} seqs, running Super5 algorithm\n");
            let (files, super5_log) = run_super5(input_seq_count);
            out.push_str(&super5_log);
            return (MPCFlat::default(), files, out);
        }
    }

    let mut log = String::new();
    if input_seq_count > 1000 {
        log.push_str(&warning(
            ">1k sequences, may be slow or use excessive memory, consider using -super5",
        ));
    }

    if output_pattern.is_empty() {
        die("Must set -output");
    }

    let output_wildcard = output_pattern.contains('@');
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let mut m = MPCFlat::default();
    if let Some(consistency_iter_count) = consistency_iter_count {
        m.consistency_iter_count = consistency_iter_count;
    }
    if let Some(refine_iter_count) = refine_iter_count {
        m.refine_iter_count = refine_iter_count;
    }

    if stratified && diversified {
        die("Cannot set both -stratified and -diversified");
    }
    if stratified || diversified {
        if tree_perm.is_some() || perturb_seed.is_some() {
            die("Cannot set -perm or -perturb with -stratified or -diversified");
        }
    }

    let mut rep_count = 1;
    if stratified {
        rep_count = 4;
    } else if diversified {
        rep_count = 100;
    }
    if let Some(replicates) = replicates {
        rep_count = replicates;
    }

    let mut output_file_names = Vec::new();
    if rep_count == 1 {
        let perturb_seed = perturb_seed.unwrap_or(0);
        let tp = tree_perm.unwrap_or(TREEPERM::TP_None);
        if tp == TREEPERM::TP_All {
            die("-perm all not supported, use -stratified");
        }
        let output_file_name = if output_wildcard {
            make_replicate_file_name(output_pattern, tp, perturb_seed)
        } else {
            output_pattern.to_string()
        };
        let (_hp, out) = align(
            &mut m,
            &input_seqs,
            perturb_seed,
            tp,
            false,
            true,
            |hp| cmd_line_update(hp),
            |mpc, input_seqs| run_mpc(mpc, input_seqs),
        );
        std::fs::write(&output_file_name, out).expect("failed to write align output");
        output_file_names.push(output_file_name);
        return (m, output_file_names, log);
    }

    let mut stratified_reps = false;
    if stratified {
        stratified_reps = true;
        rep_count *= 4;
        if tree_perm.is_some() {
            die("Cannot set both -perm and -stratified");
        }
        assert!(rep_count > 0);
    }

    let mut combined_out = String::new();
    for rep_index in 0..rep_count {
        let perturb_seed = if stratified_reps {
            rep_index / 4
        } else {
            rep_index
        };
        let tp = tree_perm.unwrap_or(match rep_index % 4 {
            0 => TREEPERM::TP_None,
            1 => TREEPERM::TP_ABC,
            2 => TREEPERM::TP_ACB,
            3 => TREEPERM::TP_BCA,
            _ => unreachable!(),
        });
        log.push_str(&format!(
            "Replicate {}/{}, {}.{}\n",
            rep_index + 1,
            rep_count,
            treeperm_to_str(tp),
            perturb_seed
        ));
        let write_efa_header = !output_wildcard;
        let (_hp, out) = align(
            &mut m,
            &input_seqs,
            perturb_seed,
            tp,
            write_efa_header,
            true,
            |hp| cmd_line_update(hp),
            |mpc, input_seqs| run_mpc(mpc, input_seqs),
        );
        if output_wildcard {
            let output_file_name = make_replicate_file_name(output_pattern, tp, perturb_seed);
            std::fs::write(&output_file_name, out).expect("failed to write align output");
            output_file_names.push(output_file_name);
        } else {
            combined_out.push_str(&out);
        }
    }

    if !output_wildcard {
        std::fs::write(output_pattern, combined_out).expect("failed to write align output");
        output_file_names.push(output_pattern.to_string());
    }
    (m, output_file_names, log)
}
