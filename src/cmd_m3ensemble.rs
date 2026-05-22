// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Builds an ensemble of MUSCLE3 alignments by varying BLOSUM, parameter group,
/// perturbation seed, and delta; concatenates labeled FASTA replicates to disk.
#[track_caller]
pub fn cmd_m3ensemble(
    input_file_name: &str,
    output_file_name: &str,
    replicates: Option<uint>,
) -> String {
    assert!(!output_file_name.is_empty());
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);

    let replicates = replicates.unwrap_or(16);
    assert!(replicates > 0);
    let delta = 0.1_f32;
    let mut out = String::new();
    let format_g3 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d64:.2e}");
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
            format!("{d64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    for ii in 0..replicates {
        let perturb_seed = ii / 4;
        let param_group = if replicates == 4 { 0 } else { (ii * 7) % 4 };
        let pct_id = match ii % 4 {
            0 => 90,
            1 => 80,
            2 => 70,
            3 => 62,
            _ => unreachable!(),
        };

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
            delta,
            delta,
            delta,
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
        out.push_str(&format!(
            "<blosum{pct_id}:{param_group}.perturb{perturb_seed}.delta{}\n",
            format_g3(delta)
        ));
        out.push_str(&msa_to_fasta_file_l112(&final_msa));
    }
    std::fs::write(output_file_name, &out).expect("failed to write M3 ensemble output");
    out
}
