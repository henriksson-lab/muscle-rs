// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: run multiple perturbed Muscle3 replicates and keep the best by self-score.
#[track_caller]
pub fn cmd_m3select(
    input_file_name: &str,
    output_file_name: &str,
    replicates: Option<uint>,
) -> String {
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);

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

    let mut best_msa: Option<MultiSequence> = None;
    let mut best_self_score = 0.0_f32;
    for ii in 0..replicates {
        let perturb_seed = ii;
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
            &input_seqs,
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
            &master_subst_mx_letter,
            master_gap_open,
            &m3.input_seq_weights,
        );
        let self_score = profile3_get_self_score(&prof);
        if self_score > best_self_score {
            best_msa = Some(final_msa);
            best_self_score = self_score;
        }
    }
    let best_msa = best_msa.expect("BestMSA != 0");
    let out = msa_to_fasta_file_l112(&best_msa);
    if !output_file_name.is_empty() {
        multi_sequence_write_mfa(&best_msa, output_file_name);
    }
    out
}
