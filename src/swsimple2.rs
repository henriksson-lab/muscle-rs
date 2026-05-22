// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Loose equality for SW scores: tolerates 1% relative error and treats
/// any pair of very-negative values as equal.
#[track_caller]
pub fn myfeq(x: f64, y: f64) -> bool {
    if x == y {
        return true;
    }
    if x < -999.0 && y < -999.0 {
        return true;
    }
    let x_abs = x.abs();
    let y_abs = y.abs();
    let max = x_abs.max(y_abs);
    let diff = (x_abs - y_abs).abs();
    diff < max * 0.01
}

/// Render a single matrix cell, mapping sentinel values to `*`, `&`, ` `.
#[track_caller]
pub fn logx(x: f32) -> String {
    if x == -9e9_f32 {
        format!("  {:>7.7}", "*")
    } else if x == f32::MAX {
        format!("  {:>7.7}", "&")
    } else if x == -8e8_f32 {
        format!("  {:>7.7}", " ")
    } else {
        format!("  {x:7.2}")
    }
}

/// Format a float DP matrix with a header row for debug output.
#[track_caller]
pub fn log_mx(name: &str, mx: &[Vec<f32>]) -> String {
    let la = mx.len();
    let lb = mx[0].len();
    let mut s = String::new();
    s.push_str(&format!("\nLogMx({name})\n"));
    s.push_str("      ");
    for j in 0..lb {
        s.push_str(&format!("  {j:7}"));
    }
    s.push('\n');
    for i in 0..la {
        s.push_str(&format!("{i:3} | "));
        for j in 0..lb {
            s.push_str(&logx(mx[i][j]));
        }
        s.push('\n');
    }
    s.push('\n');
    s
}

/// Format a traceback-character matrix for debug output.
#[track_caller]
pub fn log_tb_mx(name: &str, mx: &[Vec<char>]) -> String {
    let la = mx.len();
    let lb = mx[0].len();
    let mut s = String::new();
    s.push_str(&format!("\nLogMx({name})\n"));
    s.push_str("      ");
    for j in 0..lb {
        s.push_str(&format!("  {}", j % 10));
    }
    s.push('\n');
    for i in 0..la {
        s.push_str(&format!("{i:3} | "));
        for j in 0..lb {
            s.push(mx[i][j]);
        }
        s.push('\n');
    }
    s.push('\n');
    s
}

/// Compare two DP matrices cell-by-cell with `myfeq`; panics on mismatch.
#[track_caller]
pub fn cmp_mx(c: char, sm: &[Vec<f32>], m: &[Vec<f32>]) {
    let mut la = sm.len();
    assert!(m.len() == la);
    let mut lb = sm[0].len();
    assert!(m[0].len() == lb);

    if c != 'M' {
        la -= 1;
        lb -= 1;
    }
    for i in 1..la {
        for j in 1..lb {
            let sx = sm[i][j];
            let x = m[i][j];
            if !myfeq(sx as f64, x as f64) {
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
                panic!("{c} i={i} j={j} {} {}", format_g3(sx), format_g3(x));
            }
        }
    }
}

/// Run the reference SW (`sw_simple_fwd_mdi`) and the fast SW
/// (`sw_fast_s_mx`) in parallel and assert their scores agree.
#[track_caller]
pub fn sw_simple2<FMatch, FMM, FMD, FMI, FDM, FDD, FIM, FII>(
    mem: &mut XDPMem,
    la: uint,
    lb: uint,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
    get_match_score: FMatch,
    get_score_mm: FMM,
    get_score_md: FMD,
    get_score_mi: FMI,
    get_score_dm: FDM,
    get_score_dd: FDD,
    get_score_im: FIM,
    get_score_ii: FII,
) -> f32
where
    FMatch: Fn(uint, uint) -> f32 + Copy,
    FMM: Fn(uint, uint) -> f32 + Copy,
    FMD: Fn(uint, uint) -> f32 + Copy,
    FMI: Fn(uint, uint) -> f32 + Copy,
    FDM: Fn(uint, uint) -> f32 + Copy,
    FDD: Fn(uint, uint) -> f32 + Copy,
    FIM: Fn(uint, uint) -> f32 + Copy,
    FII: Fn(uint, uint) -> f32 + Copy,
{
    let mut simple_fwd_m = Vec::new();
    let mut simple_fwd_d = Vec::new();
    let mut simple_fwd_i = Vec::new();
    let mut simple_tbm = Vec::new();
    let mut simple_tbd = Vec::new();
    let mut simple_tbi = Vec::new();
    let mut simple_path = String::new();
    let mut simple_lo_a = uint::MAX;
    let mut simple_lo_b = uint::MAX;
    let simple_score = sw_simple_fwd_mdi(
        la,
        lb,
        &mut simple_lo_a,
        &mut simple_lo_b,
        &mut simple_path,
        &mut simple_fwd_m,
        &mut simple_fwd_d,
        &mut simple_fwd_i,
        &mut simple_tbm,
        &mut simple_tbd,
        &mut simple_tbi,
        get_match_score,
        get_score_mm,
        get_score_md,
        get_score_mi,
        get_score_dm,
        get_score_dd,
        get_score_im,
        get_score_ii,
    );

    let mut smx = Mx {
        row_count: la,
        col_count: lb,
        data: vec![vec![0.0; lb as usize]; la as usize],
        ..Mx::default()
    };
    for i in 0..la {
        for j in 0..lb {
            smx.data[i as usize][j as usize] = get_match_score(i, j);
        }
    }
    let open = get_score_md(0, 0);
    let ext = get_score_dd(0, 0);
    let (smx_score, _smx_lo_i, _smx_lo_j, _smx_len_i, _smx_len_j, _smx_path) =
        sw_fast_s_mx(mem, &smx, open, ext);

    assert!((simple_score - smx_score).abs() < 1e-3);
    *lo_a = simple_lo_a;
    *lo_b = simple_lo_b;
    *path = simple_path;
    simple_score
}

/// CLI smoke test that runs `sw_simple2` on a hard-coded pair with the
/// BLOSUM62 path scorer.
#[track_caller]
pub fn cmd_swsimple2() -> String {
    let a = "SEQVENCE".to_string();
    let b = "QVEN".to_string();
    let ps = PathScorerAABLOSUM62 {
        gap_open: -2.2,
        gap_ext: -0.5,
        seq_a: a.clone(),
        seq_b: b.clone(),
        base: PathScorer {
            la: a.len() as uint,
            lb: b.len() as uint,
            ..PathScorer::default()
        },
    };
    let mut mem = XDPMem::default();
    let mut lo_a = uint::MAX;
    let mut lo_b = uint::MAX;
    let mut path = String::new();
    let score = sw_simple2(
        &mut mem,
        ps.base.la,
        ps.base.lb,
        &mut lo_a,
        &mut lo_b,
        &mut path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps, pa, pb),
    );
    let score_s = if score == 0.0 {
        "0".to_string()
    } else if !score.is_finite() {
        score.to_string()
    } else {
        let score64 = f64::from(score);
        let exp = score64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{score64:.2e}");
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
            format!("{score64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    format!("Score {score_s} path={path}\n")
}
