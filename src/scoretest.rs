// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Count occurrences of each amino-acid letter in a column, plus gap count.
#[track_caller]
pub fn get_letter_counts(seq: &str) -> (uint, Vec<uint>) {
    let mut letter_counts = vec![0_u32; 20];
    let mut gap_count: uint = 0;
    for c in seq.bytes() {
        if c == b'-' || c == b'.' {
            gap_count += 1;
            continue;
        }
        let letter = match (c as char).to_ascii_uppercase() {
            'A' => 0,
            'C' => 1,
            'D' => 2,
            'E' => 3,
            'F' => 4,
            'G' => 5,
            'H' => 6,
            'I' => 7,
            'K' => 8,
            'L' => 9,
            'M' => 10,
            'N' => 11,
            'P' => 12,
            'Q' => 13,
            'R' => 14,
            'S' => 15,
            'T' => 16,
            'V' => 17,
            'W' => 18,
            'Y' => 19,
            _ => uint::MAX,
        };
        assert!(letter < 20);
        letter_counts[letter as usize] += 1;
    }
    (gap_count, letter_counts)
}

/// Score two profile columns with given parameters and return the score and a verbose log.
#[track_caller]
pub fn inner_test(
    col_a: &str,
    col_b: &str,
    norm_aa_freqs: uint,
    mul_occs: uint,
    center: f32,
    add_center: f32,
    na: uint,
    lla: uint,
    lga: uint,
    gla: uint,
    gga: uint,
    nb: uint,
    llb: uint,
    lgb: uint,
    glb: uint,
    ggb: uint,
    lca: &[uint],
    lcb: &[uint],
) -> (f32, String) {
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
    let format_g4 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d64:.3e}");
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

    let mut subst_mx = get_subst_mx_letter_blosum(62);
    let subst_center = (if center != f32::MAX { center } else { 0.0 })
        + if add_center != f32::MAX {
            add_center
        } else {
            0.0
        };
    if subst_center != 0.0 {
        for row in &mut subst_mx {
            for score in row {
                *score += subst_center;
            }
        }
    }

    let mut ppa = ProfPos3::default();
    let mut ppb = ProfPos3::default();
    prof_pos3_set_freqs2(&mut ppa, na, lla, lga, gla, gga, lca);
    prof_pos3_set_aa_scores(&mut ppa, &subst_mx);
    prof_pos3_set_freqs2(&mut ppb, nb, llb, lgb, glb, ggb, lcb);
    prof_pos3_set_aa_scores(&mut ppb, &subst_mx);
    let score = score_prof_pos2(&ppa, &ppb);

    let mut out = String::new();
    out.push_str("\n_________________________________\n");
    out.push_str(&format!("Norm={norm_aa_freqs}, MulOccs={mul_occs}"));
    if center != f32::MAX {
        out.push_str(&format!(" Center={}", format_g3(center)));
    }
    if add_center != f32::MAX {
        out.push_str(&format!(" AddCenter={}", format_g3(add_center)));
    }
    out.push('\n');
    out.push_str("PPA:\n");
    out.push_str(&prof_pos3_log_me(&ppa));
    out.push_str("PPB:\n");
    out.push_str(&prof_pos3_log_me(&ppb));
    out.push_str(&format!("Score = {}\n", format_g4(score)));
    out.push_str(&format!(
        "&& {col_a:>7} {col_b:>7} Norm={norm_aa_freqs} MulOccs={mul_occs}"
    ));
    if center != f32::MAX {
        out.push_str(&format!("    Center={center:6.2}"));
    }
    if add_center != f32::MAX {
        out.push_str(&format!(" AddCenter={add_center:6.2}"));
    }
    out.push_str(&format!(" Score={score:8.3}\n"));
    (score, out)
}

/// Run a single Center vs AddCenter equivalence test on a pair of columns.
#[track_caller]
pub fn test1_l87(col_a: &str, col_b: &str, c: f32) -> String {
    let format_g5 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 5 {
            let raw = format!("{d64:.4e}");
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
            let decimals = (4 - exp).max(0) as usize;
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

    let na = col_a.len() as uint;
    let nb = col_b.len() as uint;
    let (lga, lca) = get_letter_counts(col_a);
    let (lgb, lcb) = get_letter_counts(col_b);
    let lla = na - lga;
    let llb = nb - lgb;
    let gla = 0;
    let glb = 0;
    let gga = 0;
    let ggb = 0;

    let mut out = String::new();
    out.push_str("\n==========================\n");
    out.push_str(&format!("ColA = {col_a}\n"));
    out.push_str(&format!("ColB = {col_b}\n"));

    let (score1, log1) = inner_test(
        col_a, col_b, 1, 1, c, 0.0, na, lla, lga, gla, gga, nb, llb, lgb, glb, ggb, &lca, &lcb,
    );
    out.push_str(&log1);
    let (score2, log2) = inner_test(
        col_a, col_b, 0, 0, 0.0, c, na, lla, lga, gla, gga, nb, llb, lgb, glb, ggb, &lca, &lcb,
    );
    out.push_str(&log2);
    if myfeq(score1 as f64, score2 as f64) {
        out.push_str(" == Equivalent OK ==\n");
    } else {
        panic!(
            "Score1 {}, Score2 {}\n",
            format_g5(score1),
            format_g5(score2)
        );
    }
    out
}

/// `scoretest` command: run fixed pair-column scoring tests to verify Center/AddCenter equivalence.
#[track_caller]
pub fn cmd_scoretest() -> String {
    set_alpha_l209(ALPHA::ALPHA_Amino);
    let mut out = String::new();
    out.push_str(&test1_l87("C-", "CC", 0.5));
    out.push_str(&test1_l87("SEQ", "SEQ", 1.0));
    out.push_str(&test1_l87("AAC-", "MCC", 2.0));
    out.push_str(&test1_l87("AAC-", "SEQ---", 2.5));
    out
}
