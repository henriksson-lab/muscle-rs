// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Sanity-checks log-domain addition and multiplication against linear
/// arithmetic for two probability values.
#[track_caller]
pub fn test2_l4(p1d: f64, p2d: f64) -> String {
    let p1 = p1d as f32;
    let p2 = p2d as f32;

    let prod = p1 * p2;
    let sum = p1 + p2;

    let log_p1 = p1.ln();
    let log_p2 = p2.ln();
    let log_prod = prod.ln();
    let log_sum = sum.ln();

    let sum_p12 = log_p1 + log_p2;
    assert!(myfeq(log_prod as f64, sum_p12 as f64));

    let add = sum_log_prob(log_p1, log_p2);
    assert!(myfeq(add as f64, log_sum as f64));

    let mut pe = log_p1;
    pe = sum_log_prob(pe, log_p2);
    assert!(myfeq(pe as f64, log_sum as f64));

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
    format!("P1={} P2={} ok\n", format_g3(p1), format_g3(p2))
}

/// Benchmark scaffold summing many `exp(x)` values; currently a no-op
/// stub.
#[track_caller]
pub fn test_exp() -> String {
    let mut xs = Vec::<f32>::new();
    for _ in 0..0 {
        let r = randu32() % 16;
        let x = -(r as f32) / 16.0;
        xs.push(x);
    }
    let mut sum = 0.0_f32;
    for x in &xs {
        sum += x.exp();
    }
    for x in &xs {
        sum += x.exp();
    }
    assert_eq!(sum, 0.0);
    String::new()
}
