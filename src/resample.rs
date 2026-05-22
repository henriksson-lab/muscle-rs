// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `resample` command: generate bootstrap MSA replicates by resampling high-quality columns from an ensemble.
#[track_caller]
pub fn cmd_resample(
    file_name: &str,
    output_pattern: &str,
    max_gap_fract: f64,
    min_conf: f64,
    replicate_count: uint,
) -> Vec<(String, MultiSequence)> {
    if output_pattern.is_empty() {
        panic!("Must set -output");
    }

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, file_name);

    let site_count = ensemble_get_median_hi_qual_col_count(&e, max_gap_fract, min_conf);
    if site_count == 0 {
        let format_g3 = |d: f64| -> String {
            if d == 0.0 {
                return "0".to_string();
            }
            if !d.is_finite() {
                return d.to_string();
            }
            let exp = d.abs().log10().floor() as i32;
            let mut s = if exp < -4 || exp >= 3 {
                let raw = format!("{d:.2e}");
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
                format!("{d:.decimals$}")
            };
            if !s.contains('e') && !s.contains('E') {
                s = s.trim_end_matches('0').trim_end_matches('.').to_string();
            }
            if s == "-0" {
                s = "0".to_string();
            }
            s
        };
        panic!(
            "All columns low qual (max fract {}, min conf {})",
            format_g3(max_gap_fract),
            format_g3(min_conf)
        );
    }

    let non_gappy_unique_ixs = ensemble_get_hi_qual_unique_ixs(&e, max_gap_fract, min_conf);
    let n = non_gappy_unique_ixs.len() as uint;

    let output_wildcard = output_pattern.contains('@');
    let mut combined_out = String::new();
    let mut reps = Vec::<(String, MultiSequence)>::new();
    for rep_index in 0..replicate_count {
        let mut resampled_unique_ixs = Vec::<uint>::new();
        for _ in 0..site_count {
            let r = randu32() % n;
            let unique_ix = non_gappy_unique_ixs[r as usize];
            resampled_unique_ixs.push(unique_ix);
        }

        let rep_aln = ensemble_make_resampled_msa(&e, &resampled_unique_ixs);
        if output_wildcard {
            let output_file_name = make_replicate_file_name_n(output_pattern, rep_index + 1);
            msa_to_fasta_file_l103(&rep_aln, &output_file_name);
            reps.push((output_file_name, rep_aln));
        } else {
            combined_out.push_str(&format!("<resampled.{}\n", rep_index + 1));
            combined_out.push_str(&msa_to_fasta_file_l112(&rep_aln));
            reps.push((format!("resampled.{}", rep_index + 1), rep_aln));
        }
    }

    if !output_wildcard {
        std::fs::write(output_pattern, combined_out).expect("failed to write resample output");
    }
    reps
}
