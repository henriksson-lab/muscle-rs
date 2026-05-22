// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct TestSwMmBruteState {
    pub best_score: f32,
    pub best_path: String,
    pub best_pos_a: uint,
    pub best_pos_b: uint,
    pub log_all_paths: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TestSwStats {
    pub n: uint,
    pub n_all_ok: uint,
    pub n_path_diff: uint,
    pub n_score_diff: uint,
}

/// Resets the brute-force best-score tracker before a new search.
#[track_caller]
pub fn clear_brute_l36(state: &mut TestSwMmBruteState) {
    state.best_score = 0.0;
    state.best_path.clear();
    state.best_pos_a = uint::MAX;
    state.best_pos_b = uint::MAX;
}

/// Scores one enumerated path during brute-force search and updates the
/// best-so-far state.
#[track_caller]
pub fn on_path_l44<F>(
    state: &mut TestSwMmBruteState,
    pos_a: uint,
    pos_b: uint,
    path: &str,
    mut get_local_score: F,
) -> Option<String>
where
    F: FnMut(uint, uint, &str) -> f32,
{
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
    let score = get_local_score(pos_a, pos_b, path);
    let log = if state.log_all_paths {
        let score_s = format_g3(score);
        Some(format!("{score_s:>10}  {pos_a:5}  {pos_b:5}  {path}\n"))
    } else {
        None
    };
    if score > state.best_score {
        state.best_score = score;
        state.best_path = path.to_string();
        state.best_pos_a = pos_a;
        state.best_pos_b = pos_b;
    }
    log
}

/// Builds an MASM profile from a set of aligned amino-acid rows.
#[track_caller]
pub fn make_masm_a_as(rows: &[String]) -> MASM {
    let seq_count = rows.len();
    let mut col_count = uint::MAX;
    for i in 0..seq_count {
        let row = &rows[i];
        if i == 0 {
            col_count = row.len() as uint;
        } else {
            assert_eq!(row.len() as uint, col_count);
        }
    }
    let labels = vec!["Row".to_string(); seq_count];
    let mut aln = MultiSequence::default();
    multi_sequence_from_strings(&mut aln, &labels, rows);
    let mut m = MASM::default();
    mega_from_msa_aa_only(&aln, -3.0, -1.0);
    masm_from_msa(&mut m, &aln, "MSA", 3.0, 1.0);
    m
}

/// Compares brute-force and MASM/Mega aligners on rows_A vs sequence B
/// and returns the log plus per-test counters.
#[track_caller]
pub fn test_l96<FLocal, FMm>(
    rows_a: &[String],
    b: &str,
    _always_log: bool,
    mut get_local_score: FLocal,
    sw_fast_masm_mega_prof: FMm,
) -> (String, uint, uint, uint, uint)
where
    FLocal: FnMut(uint, uint, &str) -> f32,
    FMm: Fn(&MASM, &[Vec<byte>], f32, f32) -> (f32, uint, uint, uint, uint, String),
{
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
    let seq_count_a = rows_a.len() as uint;
    let mut la = uint::MAX;
    for (i, row) in rows_a.iter().enumerate() {
        if i == 0 {
            la = row.len() as uint;
        } else {
            assert_eq!(row.len() as uint, la);
        }
    }

    let mut brute = TestSwMmBruteState::default();
    clear_brute_l36(&mut brute);
    let ma = make_masm_a_as(rows_a);
    let pb = make_mega_profile_aa(b);
    let (mm_score, mm_loi, mm_loj, _mm_leni, _mm_lenj, mm_path) =
        sw_fast_masm_mega_prof(&ma, &pb, -3.0, -1.0);

    let lb = b.len() as uint;
    for (pos_a, pos_b, path) in enum_paths_local_l63(la, lb) {
        on_path_l44(&mut brute, pos_a, pos_b, &path, |_pos_a, _pos_b, path| {
            get_local_score(pos_a, pos_b, path)
        });
    }

    let n = 1;
    let mut n_all_ok = 0;
    let mut n_path_diff = 0;
    let mut n_score_diff = 0;
    let mut all_ok = true;
    let mut out = String::new();
    if brute.best_path != mm_path {
        out.push_str("@PATHDIFF\n");
        n_path_diff += 1;
        all_ok = false;
    }
    if (brute.best_score - mm_score).abs() > 1e-6 {
        out.push_str("@SCOREDIFF\n");
        n_score_diff += 1;
        all_ok = false;
    }
    if all_ok {
        n_all_ok += 1;
    }

    if !all_ok {
        out.push('\n');
        for (i, row) in rows_a.iter().enumerate().take(seq_count_a as usize) {
            out.push_str(&format!("{row}  >A{i}\n"));
        }
        out.push_str(&format!("{b}  >B\n"));

        let best_score_s = format_g3(brute.best_score);
        let mm_score_s = format_g3(mm_score);
        out.push_str(&format!(
            "  {best_score_s:>7}  {}  ({}, {})  Brute\n",
            brute.best_path, brute.best_pos_a, brute.best_pos_b
        ));
        out.push_str(&format!(
            "  {mm_score_s:>7}  {mm_path}  ({mm_loi}, {mm_loj})  MASM_Mega \n"
        ));
    }
    (out, n, n_all_ok, n_path_diff, n_score_diff)
}

/// Splits a `|`-separated MSA string and dispatches to `test_l96`.
#[track_caller]
pub fn test_l157<FLocal, FMm>(
    s_a: &str,
    b: &str,
    get_local_score: FLocal,
    sw_fast_masm_mega_prof: FMm,
) -> (String, uint, uint, uint, uint)
where
    FLocal: FnMut(uint, uint, &str) -> f32,
    FMm: Fn(&MASM, &[Vec<byte>], f32, f32) -> (f32, uint, uint, uint, uint, String),
{
    let rows_a = split(s_a, '|');
    test_l96(&rows_a, b, true, get_local_score, sw_fast_masm_mega_prof)
}

/// Builds a detailed trace log for a single MSA-vs-sequence alignment
/// path.
#[track_caller]
pub fn log_path<FTrace>(
    s_a: &str,
    b: &str,
    pos_a: uint,
    pos_b: uint,
    path: &str,
    trace_local_score: FTrace,
) -> String
where
    FTrace: Fn(&MASM, &[Vec<byte>], uint, uint, uint, uint, &str) -> String,
{
    let mut out = String::new();
    out.push_str("___________________________________\n");

    let rows_a = split(s_a, '|');
    let seq_count_a = rows_a.len() as uint;
    let mut la = uint::MAX;
    for (i, row) in rows_a.iter().enumerate() {
        if i == 0 {
            la = row.len() as uint;
        } else {
            assert_eq!(row.len() as uint, la);
        }
    }

    let ma = make_masm_a_as(&rows_a);
    out.push_str(&masm_log_me(&ma));
    let pb = make_mega_profile_aa(b);
    let lb = b.len() as uint;

    out.push('\n');
    for (i, row) in rows_a.iter().enumerate().take(seq_count_a as usize) {
        out.push_str(&format!("{row}  >A{i}\n"));
    }
    out.push_str(&format!("{b}  >B\n"));
    out.push_str(&format!("Path {path}\n"));
    out.push_str(&trace_local_score(&ma, &pb, la, lb, pos_a, pos_b, path));
    out
}

/// Returns a random short amino acid sequence within the test length
/// range.
#[track_caller]
pub fn get_rand_seq_l208() -> String {
    let min_rand_l: uint = 3;
    let max_rand_l: uint = 7;
    let l = min_rand_l + randu32() % (max_rand_l - min_rand_l + 1);
    assert!(l >= min_rand_l && l <= max_rand_l);
    let mut s = String::new();
    for _ in 0..l {
        s.push(AMINO_ALPHA.as_bytes()[(randu32() % 20) as usize] as char);
    }
    s
}

/// Returns a random MSA with gaps; guarantees no column is all-gaps.
#[track_caller]
pub fn get_rand_rows() -> Vec<String> {
    let min_rand_l: uint = 3;
    let max_rand_l: uint = 7;
    let min_rand_n: uint = 1;
    let max_rand_n: uint = 5;
    let rand_gap_pct: uint = 25;
    let mut rows = Vec::new();
    let l = min_rand_l + randu32() % (max_rand_l - min_rand_l + 1);
    let n = min_rand_n + randu32() % (max_rand_n - min_rand_n + 1);
    assert!(l >= min_rand_l && l <= max_rand_l);
    assert!(n >= min_rand_n && n <= max_rand_n);

    for _i in 0..n {
        let mut row = String::new();
        for _j in 0..l {
            if randu32() % 100 < rand_gap_pct {
                row.push('-');
            } else {
                row.push(AMINO_ALPHA.as_bytes()[(randu32() % 20) as usize] as char);
            }
        }
        rows.push(row);
    }
    for col_idx in 0..l as usize {
        let mut all_gaps = true;
        for seq_idx in 0..n as usize {
            if !matches!(rows[seq_idx].as_bytes()[col_idx] as char, '-' | '.') {
                all_gaps = false;
                break;
            }
        }
        if all_gaps {
            let row_idx = (randu32() % n) as usize;
            let c = AMINO_ALPHA.as_bytes()[(randu32() % 20) as usize] as char;
            rows[row_idx].replace_range(col_idx..col_idx + 1, &c.to_string());
        }
    }
    rows
}

/// Runs one random MSA-vs-sequence comparison and accumulates statistics.
#[track_caller]
pub fn test_random_l253<FLocal, FMm>(
    stats: &mut TestSwStats,
    get_local_score: FLocal,
    sw_fast_masm_mega_prof: FMm,
) -> String
where
    FLocal: FnMut(uint, uint, &str) -> f32,
    FMm: Fn(&MASM, &[Vec<byte>], f32, f32) -> (f32, uint, uint, uint, uint, String),
{
    let rows_a = get_rand_rows();
    let b = get_rand_seq_l208();
    let (out, n, n_all_ok, n_path_diff, n_score_diff) =
        test_l96(&rows_a, &b, false, get_local_score, sw_fast_masm_mega_prof);
    stats.n += n;
    stats.n_all_ok += n_all_ok;
    stats.n_path_diff += n_path_diff;
    stats.n_score_diff += n_score_diff;
    out
}

/// Entry point for the `test_sw_mm` command: runs a fixed regression case
/// and prints the alignment trace.
#[track_caller]
pub fn cmd_test_sw_mm<FLocal, FMm, FTrace>(
    get_local_score: FLocal,
    sw_fast_masm_mega_prof: FMm,
    trace_local_score: FTrace,
) -> String
where
    FLocal: FnMut(uint, uint, &str) -> f32,
    FMm: Fn(&MASM, &[Vec<byte>], f32, f32) -> (f32, uint, uint, uint, uint, String) + Copy,
    FTrace: Fn(&MASM, &[Vec<byte>], uint, uint, uint, uint, &str) -> String,
{
    let mut out = String::new();
    let (test_out, _n, _ok, _pd, _sd) =
        test_l157("WAQHEAW", "CTFWH", get_local_score, sw_fast_masm_mega_prof);
    out.push_str(&test_out);
    out.push_str(&log_path(
        "WAQHEAW",
        "CTFWH",
        0,
        3,
        "MDDM",
        trace_local_score,
    ));
    out
}
