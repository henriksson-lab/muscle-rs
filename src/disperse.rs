// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Compute and report the LP and column dispersion metrics of an ensemble FASTA.
#[track_caller]
pub fn cmd_disperse(file_name: &str, max_gap_fract: f64) -> String {
    let format_g4 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d:.3e}");
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
            let decimals = (3 - exp).max(0) as usize;
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
    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, file_name);

    let (d_lp, d_cols) = ensemble_get_dispersion(&e, max_gap_fract);
    let d_lp_s = format_g4(d_lp);
    let d_cols_s = format_g4(d_cols);
    format!("@disperse file={file_name} D_LP={d_lp_s} D_Cols={d_cols_s}\n")
}
