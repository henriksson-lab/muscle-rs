// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub struct SWTester {
    pub x: Option<SWer>,
    pub y: Option<SWer>,
    pub x_name: String,
    pub y_name: String,
    pub n: uint,
    pub n_agree: uint,
    pub n_score_diff: uint,
    pub n_path_diff: uint,
    pub n_pos_diff: uint,
    pub n_ps_score_ok: uint,
    pub n_ps_score_diff: uint,
    pub a: String,
    pub b: String,
    pub x_score: f32,
    pub y_score: f32,
    pub x_lo_a: uint,
    pub y_lo_a: uint,
    pub x_lo_b: uint,
    pub y_lo_b: uint,
    pub x_path: String,
    pub y_path: String,
} // original: SWTester (muscle/src/swtester.h)

impl Default for SWTester {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            x_name: String::new(),
            y_name: String::new(),
            n: 0,
            n_agree: 0,
            n_score_diff: 0,
            n_path_diff: 0,
            n_pos_diff: 0,
            n_ps_score_ok: 0,
            n_ps_score_diff: 0,
            a: String::new(),
            b: String::new(),
            x_score: f32::MAX,
            y_score: f32::MAX,
            x_lo_a: uint::MAX,
            y_lo_a: uint::MAX,
            x_lo_b: uint::MAX,
            y_lo_b: uint::MAX,
            x_path: String::new(),
            y_path: String::new(),
        }
    }
}

#[track_caller]
pub fn sw_tester_set_x_name(swt: &mut SWTester, name: &str) {
    swt.x_name = name.to_string();
}

#[track_caller]
pub fn sw_tester_set_y_name(swt: &mut SWTester, name: &str) {
    swt.y_name = name.to_string();
}

fn sw_tester_x_name(swt: &SWTester) -> &str {
    if !swt.x_name.is_empty() {
        &swt.x_name
    } else if swt.x.is_some() {
        "SWer"
    } else {
        ""
    }
}

fn sw_tester_y_name(swt: &SWTester) -> &str {
    if !swt.y_name.is_empty() {
        &swt.y_name
    } else if swt.y.is_some() {
        "SWer"
    } else {
        ""
    }
}

/// Clears only cumulative statistics, matching C++ `SWTester::ClearStats`.
#[track_caller]
pub fn sw_tester_clear_stats(swt: &mut SWTester) {
    swt.n = 0;
    swt.n_agree = 0;
    swt.n_score_diff = 0;
    swt.n_path_diff = 0;
    swt.n_pos_diff = 0;
    swt.n_ps_score_ok = 0;
    swt.n_ps_score_diff = 0;
}

/// Runs the X aligner on `(a, b)`, records its result, and tallies the path
/// scorer cross-check.
#[track_caller]
pub fn sw_tester_run_x<FSW, FPS>(
    swt: &mut SWTester,
    a: &str,
    b: &str,
    mut sw: FSW,
    mut get_local_score: FPS,
) where
    FSW: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    swt.a = a.to_string();
    swt.b = b.to_string();
    let x = swt.x.get_or_insert_with(SWer::default);
    swt.x_score = s_wer_run(
        x,
        &swt.a,
        &swt.b,
        &mut swt.x_lo_a,
        &mut swt.x_lo_b,
        &mut swt.x_path,
        |s, lo_a, lo_b, path| sw(s, lo_a, lo_b, path),
    );
    if swt.x_score <= 0.0 {
        return;
    }
    let score_ps = get_local_score(swt.x_lo_a, swt.x_lo_b, &swt.x_path);
    if myfeq(swt.x_score as f64, score_ps as f64) {
        swt.n_ps_score_ok += 1;
    } else {
        swt.n_ps_score_diff += 1;
    }
}

/// Runs the Y aligner on `(a, b)` and stores its score and traceback.
#[track_caller]
pub fn sw_tester_run_y<FSW>(swt: &mut SWTester, a: &str, b: &str, mut sw: FSW)
where
    FSW: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
{
    swt.a = a.to_string();
    swt.b = b.to_string();
    let y = swt.y.get_or_insert_with(SWer::default);
    swt.y_score = s_wer_run(
        y,
        &swt.a,
        &swt.b,
        &mut swt.y_lo_a,
        &mut swt.y_lo_b,
        &mut swt.y_path,
        |s, lo_a, lo_b, path| sw(s, lo_a, lo_b, path),
    );
}

/// Runs both X and Y aligners and returns the comparison log, if any.
#[track_caller]
pub fn sw_tester_run_xy<FX, FY, FPS>(
    swt: &mut SWTester,
    a: &str,
    b: &str,
    mut x_sw: FX,
    mut y_sw: FY,
    mut x_get_local_score: FPS,
) -> Option<String>
where
    FX: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FY: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    swt.x.get_or_insert_with(SWer::default);
    swt.y.get_or_insert_with(SWer::default);
    sw_tester_run_x(
        swt,
        a,
        b,
        |s, lo_a, lo_b, path| x_sw(s, lo_a, lo_b, path),
        |lo_a, lo_b, path| x_get_local_score(lo_a, lo_b, path),
    );
    sw_tester_run_y(swt, a, b, |s, lo_a, lo_b, path| y_sw(s, lo_a, lo_b, path));
    sw_tester_cmp_xy(swt)
}

/// Runs aligner X on `(a, b)` and formats a human-readable report of SW vs
/// PathScorer scores.
#[track_caller]
pub fn sw_tester_run_xab<FSW, FPS>(
    swt: &mut SWTester,
    x_name: &str,
    a: &str,
    b: &str,
    trace: bool,
    mut sw: FSW,
    mut get_local_score: FPS,
) -> Option<String>
where
    FSW: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str, bool) -> f32,
{
    sw_tester_set_x_name(swt, x_name);
    sw_tester_run_x(
        swt,
        a,
        b,
        |s, lo_a, lo_b, path| sw(s, lo_a, lo_b, path),
        |lo_a, lo_b, path| get_local_score(lo_a, lo_b, path, false),
    );
    if swt.x_score <= 0.0 {
        return None;
    }
    let score_ps = get_local_score(swt.x_lo_a, swt.x_lo_b, &swt.x_path, trace);
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
    let mut out = String::new();
    out.push_str(&format!("\nRunXAB({x_name})\n"));
    out.push_str(&format!("A {a}\n"));
    out.push_str(&format!("B {b}\n"));
    out.push_str(&format!(
        "ScoreSW {}, ScorePS {}",
        format_g3(swt.x_score),
        format_g3(score_ps)
    ));
    out.push_str(&format!("   {}\n", swt.x_path));
    Some(out)
}

/// Runs X then Y on `(a, b)` and returns the cross-comparison log.
#[track_caller]
pub fn sw_tester_run_ab<FX, FY, FPS>(
    swt: &mut SWTester,
    a: &str,
    b: &str,
    mut x_sw: FX,
    mut y_sw: FY,
    mut x_get_local_score: FPS,
) -> Option<String>
where
    FX: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FY: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    sw_tester_run_x(
        swt,
        a,
        b,
        |s, lo_a, lo_b, path| x_sw(s, lo_a, lo_b, path),
        |lo_a, lo_b, path| x_get_local_score(lo_a, lo_b, path),
    );
    sw_tester_run_y(swt, a, b, |s, lo_a, lo_b, path| y_sw(s, lo_a, lo_b, path));
    sw_tester_cmp_xy(swt)
}

/// Compares the most recent X and Y results, bumps the disagreement counters
/// and returns a log message if they differ.
#[track_caller]
pub fn sw_tester_cmp_xy(swt: &mut SWTester) -> Option<String> {
    let mut agree = true;
    let mut log = None;
    swt.n += 1;
    if swt.x_score == 0.0 && swt.y_score == 0.0 {
        swt.n_agree += 1;
        return None;
    }

    if !myfeq(swt.x_score as f64, swt.y_score as f64) {
        if agree {
            log = Some(sw_tester_log_result(swt, "@SCOREDIFF"));
        }
        agree = false;
        swt.n_score_diff += 1;
    }
    if swt.x_path != swt.y_path {
        if agree {
            log = Some(sw_tester_log_result(swt, "@PATHDIFF"));
        }
        agree = false;
        swt.n_path_diff += 1;
    }
    if swt.x_lo_a != swt.y_lo_a || swt.x_lo_b != swt.y_lo_b {
        if agree {
            log = Some(sw_tester_log_result(swt, "@POSDIFF"));
        }
        agree = false;
        swt.n_pos_diff += 1;
    }
    if agree {
        swt.n_agree += 1;
    }
    log
}

/// Formats the X/Y mismatch log line tagged with `msg`.
#[track_caller]
pub fn sw_tester_log_result(swt: &SWTester, msg: &str) -> String {
    let x_name = sw_tester_x_name(swt);
    let y_name = sw_tester_y_name(swt);
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
    let mut out = String::new();
    out.push_str(&format!("{msg} {x_name} {y_name}\n"));
    out.push_str(&format!("A: {}\n", swt.a));
    out.push_str(&format!("B: {}\n", swt.b));
    out.push_str(&format!(
        "  {}/{}",
        format_g3(swt.x_score),
        format_g3(swt.y_score)
    ));
    out.push_str(&format!("  loa {},{}", swt.x_lo_a, swt.y_lo_a));
    out.push_str(&format!("  lob {},{}", swt.x_lo_b, swt.y_lo_b));
    out.push_str(&format!("  {},{}", swt.x_path, swt.y_path));
    out.push('\n');
    out
}

/// Generates a random amino acid string of length `l`, optionally including
/// gap characters.
#[track_caller]
pub fn sw_tester_get_random_seq(l: uint, with_gaps: bool) -> String {
    let mut s = String::new();
    for _ in 0..l {
        if with_gaps && randu32() % 3 == 0 {
            s.push('-');
        } else {
            s.push(AMINO_ALPHA.as_bytes()[(randu32() % 20) as usize] as char);
        }
    }
    s
}

/// Runs `iters` random sequence-vs-sequence comparisons, concatenating any
/// disagreement logs.
#[track_caller]
pub fn sw_tester_run_random_seqs_iters<FX, FY, FPS>(
    swt: &mut SWTester,
    min_l: uint,
    max_l: uint,
    iters: uint,
    mut x_sw: FX,
    mut y_sw: FY,
    mut x_get_local_score: FPS,
) -> String
where
    FX: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FY: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    let mut out = String::new();
    for iter in 0..iters {
        let _ = progress_step(iter, iters, "RunRandomSeqsIters");
        if let Some(log) = sw_tester_run_random_seqs(
            swt,
            min_l,
            max_l,
            |s, lo_a, lo_b, path| x_sw(s, lo_a, lo_b, path),
            |s, lo_a, lo_b, path| y_sw(s, lo_a, lo_b, path),
            |lo_a, lo_b, path| x_get_local_score(lo_a, lo_b, path),
        ) {
            out.push_str(&log);
        }
    }
    out
}

/// Runs `iters` random MSA-vs-sequence comparisons, concatenating any
/// disagreement logs.
#[track_caller]
pub fn sw_tester_run_random_msa_seq_iters<FX, FY, FPS>(
    swt: &mut SWTester,
    min_n: uint,
    max_n: uint,
    min_l: uint,
    max_l: uint,
    iters: uint,
    mut x_sw: FX,
    mut y_sw: FY,
    mut x_get_local_score: FPS,
) -> String
where
    FX: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FY: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    let mut out = String::new();
    for iter in 0..iters {
        let _ = progress_step(iter, iters, "RunRandomMSASeqIters");
        if let Some(log) = sw_tester_run_random_msa_seq(
            swt,
            min_n,
            max_n,
            min_l,
            max_l,
            |s, lo_a, lo_b, path| x_sw(s, lo_a, lo_b, path),
            |s, lo_a, lo_b, path| y_sw(s, lo_a, lo_b, path),
            |lo_a, lo_b, path| x_get_local_score(lo_a, lo_b, path),
        ) {
            out.push_str(&log);
        }
    }
    out
}

/// Generates a random A and B sequence pair and runs both aligners on it.
#[track_caller]
pub fn sw_tester_run_random_seqs<FX, FY, FPS>(
    swt: &mut SWTester,
    min_l: uint,
    max_l: uint,
    mut x_sw: FX,
    mut y_sw: FY,
    mut x_get_local_score: FPS,
) -> Option<String>
where
    FX: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FY: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    let la = min_l + randu32() % (max_l - min_l + 1);
    let lb = min_l + randu32() % (max_l - min_l + 1);
    let a = sw_tester_get_random_seq(la, false);
    let b = sw_tester_get_random_seq(lb, false);
    sw_tester_run_ab(
        swt,
        &a,
        &b,
        |s, lo_a, lo_b, path| x_sw(s, lo_a, lo_b, path),
        |s, lo_a, lo_b, path| y_sw(s, lo_a, lo_b, path),
        |lo_a, lo_b, path| x_get_local_score(lo_a, lo_b, path),
    )
}

/// Replaces all-gap columns in a `|`-separated alignment with a random
/// amino acid so no column is entirely gaps.
#[track_caller]
pub fn sw_tester_fix_gaps(aln_bar: &mut String) {
    let mut rows_a = split(aln_bar, '|');
    let seq_count = rows_a.len();
    let col_count = rows_a[0].len();
    for col in 0..col_count {
        let mut all_gaps = true;
        for row in rows_a.iter().take(seq_count) {
            if row.as_bytes()[col] != b'-' {
                all_gaps = false;
                break;
            }
        }
        if all_gaps {
            let seq_index = (randu32() % seq_count as uint) as usize;
            let letter = AMINO_ALPHA.as_bytes()[(randu32() % 20) as usize] as char;
            rows_a[seq_index].replace_range(col..col + 1, &letter.to_string());
        }
    }
    aln_bar.clear();
    for (i, row) in rows_a.iter().enumerate().take(seq_count) {
        if i > 0 {
            aln_bar.push('|');
        }
        aln_bar.push_str(row);
    }

    let rows_a = split(aln_bar, '|');
    let col_count = rows_a[0].len();
    for col in 0..col_count {
        let mut all_gaps = true;
        for row in rows_a.iter().take(seq_count) {
            if row.as_bytes()[col] != b'-' {
                all_gaps = false;
                break;
            }
        }
        assert!(!all_gaps);
    }
}

/// Generates a random MSA and a random sequence and runs both aligners on
/// the pair.
#[track_caller]
pub fn sw_tester_run_random_msa_seq<FX, FY, FPS>(
    swt: &mut SWTester,
    min_n: uint,
    max_n: uint,
    min_l: uint,
    max_l: uint,
    mut x_sw: FX,
    mut y_sw: FY,
    mut x_get_local_score: FPS,
) -> Option<String>
where
    FX: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FY: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
    FPS: FnMut(uint, uint, &str) -> f32,
{
    let n = min_n + randu32() % (max_n - min_n + 1);
    let la = min_l + randu32() % (max_l - min_l + 1);
    let lb = min_l + randu32() % (max_l - min_l + 1);
    let mut a = String::new();
    for i in 0..n {
        if i > 0 {
            a.push('|');
        }
        a.push_str(&sw_tester_get_random_seq(la, true));
    }
    sw_tester_fix_gaps(&mut a);
    let b = sw_tester_get_random_seq(lb, false);
    sw_tester_run_ab(
        swt,
        &a,
        &b,
        |s, lo_a, lo_b, path| x_sw(s, lo_a, lo_b, path),
        |s, lo_a, lo_b, path| y_sw(s, lo_a, lo_b, path),
        |lo_a, lo_b, path| x_get_local_score(lo_a, lo_b, path),
    )
}

/// Formats the cumulative SWTester statistics for reporting.
#[track_caller]
pub fn sw_tester_stats(swt: &SWTester) -> String {
    let mut out = String::new();
    out.push('\n');
    if swt.x.is_some() {
        out.push_str(&format!("{:>10.10}  {}\n", "X", sw_tester_x_name(swt)));
    }
    if swt.y.is_some() {
        out.push_str(&format!("{:>10.10}  {}\n", "Y", sw_tester_y_name(swt)));
    }
    out.push_str(&format!("{:10}  Tests\n", swt.n));
    out.push_str(&format!("{:10}  Agree\n", swt.n_agree));
    out.push_str(&format!("{:10}  Score diff\n", swt.n_score_diff));
    out.push_str(&format!("{:10}  Path diff\n", swt.n_path_diff));
    out.push_str(&format!("{:10}  Pos diff\n", swt.n_pos_diff));
    out.push_str(&format!("{:10}  PS score ok\n", swt.n_ps_score_ok));
    out.push_str(&format!("{:10}  PS score diff\n", swt.n_ps_score_diff));
    out
}
