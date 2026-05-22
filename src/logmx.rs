// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const OPT_LOG: i32 = 0x01;

pub const OPT_EXP: i32 = 0x02;

pub const OPT_ZERO_BASED: i32 = 0x04;

/// Formats a Tom-style (LX+1)x(LY+1) flat matrix for logging.
#[track_caller]
pub fn log_tom_mx(name: &str, mx: &[f32], lx: uint, ly: uint) -> String {
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
    out.push_str(&format!("Tom {name}: LX={lx} LY={ly}\n"));
    out.push_str("       ");
    for j in 0..=ly {
        out.push_str(&format!("  {j:10}"));
    }
    out.push('\n');

    let mut ix = 0_usize;
    for i in 0..=lx {
        out.push_str(&format!("[{i:3}]  "));
        for _j in 0..=ly {
            let p = mx[ix];
            ix += 1;
            out.push_str(&format!("  {:>10}", format_g3(p)));
        }
        out.push('\n');
    }
    out
}

/// Formats a flat (LX+1) x (LY+1) matrix for logging.
#[track_caller]
pub fn log_flat_mx1(name: &str, my_post: &[f32], lx: uint, ly: uint) -> String {
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
    out.push_str(&format!("Flat1 {name}: LX={lx} LY={ly}\n"));
    out.push_str("       ");
    for j in 0..=ly {
        out.push_str(&format!("  {j:10}"));
    }
    out.push('\n');

    let mut ix = 0_usize;
    for i in 0..=lx {
        out.push_str(&format!("[{i:3}]  "));
        for _j in 0..=ly {
            let p = my_post[ix];
            ix += 1;
            out.push_str(&format!("  {:>10}", format_g3(p)));
        }
        out.push('\n');
    }
    out
}

/// Formats a flat LX x LY matrix for logging.
#[track_caller]
pub fn log_flat_mx(name: &str, my_post: &[f32], lx: uint, ly: uint) -> String {
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
    out.push_str(&format!("Flat {name}: LX={lx} LY={ly}\n"));
    out.push_str("       ");
    for j in 0..ly {
        out.push_str(&format!("  {j:10}"));
    }
    out.push('\n');

    let mut ix = 0_usize;
    for i in 0..lx {
        out.push_str(&format!("[{i:3}]  "));
        for _j in 0..ly {
            let p = my_post[ix];
            ix += 1;
            out.push_str(&format!("  {:>10}", format_g3(p)));
        }
        out.push('\n');
    }
    out
}

/// Formats a 5 x (LX+1) x (LY+1) stack of flat matrices (one per HMM state) for logging.
#[track_caller]
pub fn log_flat_mxs(name: &str, mxs: &[f32], lx: uint, ly: uint) -> String {
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

    for state in 0..HMMSTATE_COUNT as uint {
        out.push_str(&format!("Flat {name}[{state}]: LX={lx} LY={ly}\n"));
        out.push_str("       ");
        for j in 0..=ly {
            out.push_str(&format!("  {j:10}"));
        }
        out.push('\n');

        let mut ix = state as usize;
        for i in 0..=lx {
            out.push_str(&format!("[{i:3}]  "));
            for _j in 0..=ly {
                let x = mxs[ix];
                ix += HMMSTATE_COUNT as usize;
                if x == INVALID_LOG {
                    out.push_str(&format!("  {:>8.8}", "*ERR*"));
                }
                if x == OUT_OF_BAND_LOG {
                    out.push_str(&format!("  {:>8.8}", "#"));
                }
                if x == UNINIT_LOG {
                    out.push_str(&format!("  {:>8.8}", "-"));
                } else if x == LOG_ZERO {
                    out.push_str(&format!("  {:>8.8}", "."));
                } else {
                    out.push_str(&format!("  {:>8}", format_g3(x)));
                }
            }
            out.push('\n');
        }
    }
    out
}
