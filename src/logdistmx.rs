// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Formats a distance matrix as a human-readable log table.
#[track_caller]
pub fn log_dist_mx(msg: &str, mx: &[Vec<f32>]) -> String {
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
    out.push('\n');
    out.push_str(&format!("LogDistMx({msg})\n"));
    let row_count = mx.len();
    assert!(row_count > 0);
    for row in 0..row_count {
        out.push_str(&format!("[{row:5}]  "));
        let col_count = mx[row].len();
        for col in 0..col_count {
            let x = mx[row][col];
            if x == f32::MAX {
                out.push_str(&format!("  {:>7.7}", "*"));
            } else if x == LOG_ZERO {
                out.push_str(&format!("  {:>7.7}", "."));
            } else {
                out.push_str(&format!("  {:>7}", format_g3(x)));
            }
        }
        out.push('\n');
    }
    out
}
