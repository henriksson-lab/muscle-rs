// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Resets the brute-force best-score tracker before a new search.
#[track_caller]
pub fn clear_brute_l24(state: &mut TestSwMmBruteState) {
    state.best_score = 0.0;
    state.best_path.clear();
    state.best_pos_a = uint::MAX;
    state.best_pos_b = uint::MAX;
}

/// Scores one enumerated path and updates the brute-force best so far.
#[track_caller]
pub fn on_path_l32<F>(
    state: &mut TestSwMmBruteState,
    pos_a: uint,
    pos_b: uint,
    path: &str,
    mut get_local_score: F,
) -> Option<String>
where
    F: FnMut(uint, uint, &str) -> f32,
{
    let score = get_local_score(pos_a, pos_b, path);
    let log = if state.log_all_paths {
        let mut score_s = format!("{score:.3}");
        while score_s.contains('.') && score_s.ends_with('0') {
            score_s.pop();
        }
        if score_s.ends_with('.') {
            score_s.pop();
        }
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

/// Builds an MASM amino acid profile from a single sequence with default
/// gap penalties.
#[track_caller]
pub fn make_masm_aa(seq: &str) -> MASM {
    make_masm_seq(seq, -3.0, -1.0)
}

/// Builds a Mega-style amino acid profile from a sequence, mapping each
/// residue to its alphabet index.
#[track_caller]
pub fn make_mega_profile_aa(seq: &str) -> Vec<Vec<byte>> {
    let mut prof = Vec::new();
    for c in seq.bytes() {
        let mut letter = match (c as char).to_ascii_uppercase() {
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
        if letter >= 20 {
            letter = 0;
        }
        let mut col = Vec::new();
        col.push(letter as byte);
        prof.push(col);
    }
    prof
}

/// Runs the MASM/Mega Smith-Waterman aligner on `(a, b)` and formats its
/// score and path.
#[track_caller]
pub fn test_masm_mega<F>(a: &str, b: &str, sw_fast_masm_mega_prof: F) -> String
where
    F: Fn(&MASM, &[Vec<byte>], f32, f32) -> (f32, uint, uint, uint, uint, String),
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
    let ma = make_masm_aa(a);
    let pb = make_mega_profile_aa(b);
    let (score, loi, loj, _leni, _lenj, path) = sw_fast_masm_mega_prof(&ma, &pb, -3.0, -1.0);
    let score_s = format_g3(score);
    format!("Test_MASM_Mega {score_s} ({loi}, {loj}) {path}\n")
}

/// Compares brute-force, fast string SW and MASM/Mega aligners on one
/// fixed pair and prints all three results.
#[track_caller]
pub fn test_l89<FLocal, FSw, FMm>(
    a: &str,
    b: &str,
    mut get_local_score: FLocal,
    sw_fast_strings_blosum62: FSw,
    sw_fast_masm_mega_prof: FMm,
) -> String
where
    FLocal: FnMut(uint, uint, &str) -> f32,
    FSw: Fn(&str, &str, f32, f32) -> (f32, uint, uint, uint, uint, String),
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
    let mut brute = TestSwMmBruteState::default();
    clear_brute_l24(&mut brute);

    let la = a.len() as uint;
    let lb = b.len() as uint;
    for (pos_a, pos_b, path) in enum_paths_local_l63(la, lb) {
        on_path_l32(&mut brute, pos_a, pos_b, &path, |_pos_a, _pos_b, path| {
            get_local_score(pos_a, pos_b, path)
        });
    }

    let (sw_score, sw_loi, sw_loj, _sw_leni, _sw_lenj, sw_path) =
        sw_fast_strings_blosum62(a, b, -3.0, -1.0);

    let ma = make_masm_aa(a);
    let pb = make_mega_profile_aa(b);
    let (mm_score, mm_loi, mm_loj, _mm_leni, _mm_lenj, mm_path) =
        sw_fast_masm_mega_prof(&ma, &pb, -3.0, -1.0);

    let best_score_s = format_g3(brute.best_score);
    let sw_score_s = format_g3(sw_score);
    let mm_score_s = format_g3(mm_score);

    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("A={a}({la})"));
    out.push_str(&format!("  B={b}({lb})\n"));
    out.push_str(&format!(
        "  {best_score_s:>7}  {}  ({}, {})  Brute\n",
        brute.best_path, brute.best_pos_a, brute.best_pos_b
    ));
    out.push_str(&format!(
        "  {sw_score_s:>7}  {sw_path}  ({sw_loi}, {sw_loj})  SW\n"
    ));
    out.push_str(&format!(
        "  {mm_score_s:>7}  {mm_path}  ({mm_loi}, {mm_loj})  MASM_Mega \n"
    ));
    out
}

/// Returns a random amino acid sequence with length in `[g_MinRandL,
/// g_MaxRandL]`.
#[track_caller]
pub fn get_rand_seq_l130() -> String {
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

/// Runs one random-sequence comparison between brute force, fast SW and
/// MASM/Mega, accumulating mismatch statistics.
#[track_caller]
pub fn test_random_l144<FLocal, FSw, FMm>(
    stats: &mut TestSwStats,
    mut get_local_score: FLocal,
    sw_fast_strings_blosum62: FSw,
    sw_fast_masm_mega_prof: FMm,
) -> String
where
    FLocal: FnMut(&str, &str, uint, uint, &str) -> f32,
    FSw: Fn(&str, &str, f32, f32) -> (f32, uint, uint, uint, uint, String),
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
    let mut brute = TestSwMmBruteState::default();
    clear_brute_l24(&mut brute);
    stats.n += 1;

    let a = get_rand_seq_l130();
    let b = get_rand_seq_l130();
    let la = a.len() as uint;
    let lb = b.len() as uint;
    for (pos_a, pos_b, path) in enum_paths_local_l63(la, lb) {
        on_path_l32(&mut brute, pos_a, pos_b, &path, |_pos_a, _pos_b, path| {
            get_local_score(&a, &b, pos_a, pos_b, path)
        });
    }

    let (sw_score, sw_loi, sw_loj, _sw_leni, _sw_lenj, sw_path) =
        sw_fast_strings_blosum62(&a, &b, -3.0, -1.0);

    let ma = make_masm_aa(&a);
    let pb = make_mega_profile_aa(&b);
    let (mm_score, mm_loi, mm_loj, _mm_leni, _mm_lenj, mm_path) =
        sw_fast_masm_mega_prof(&ma, &pb, -3.0, -1.0);

    let mut all_ok = true;
    if brute.best_path != sw_path || brute.best_path != mm_path || sw_path != mm_path {
        stats.n_path_diff += 1;
        all_ok = false;
    }
    if (brute.best_score - sw_score).abs() > 1e-6
        || (brute.best_score - mm_score).abs() > 1e-6
        || (sw_score - mm_score).abs() > 1e-6
    {
        stats.n_score_diff += 1;
        all_ok = false;
    }
    if all_ok {
        stats.n_all_ok += 1;
    }

    if all_ok {
        return String::new();
    }

    let best_score_s = format_g3(brute.best_score);
    let sw_score_s = format_g3(sw_score);
    let mm_score_s = format_g3(mm_score);

    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("A={a}({la})"));
    out.push_str(&format!("  B={b}({lb})\n"));
    out.push_str(&format!(
        "  {best_score_s:>7}  {}  ({}, {})  Brute\n",
        brute.best_path, brute.best_pos_a, brute.best_pos_b
    ));
    out.push_str(&format!(
        "  {sw_score_s:>7}  {sw_path}  ({sw_loi}, {sw_loj})  SW\n"
    ));
    out.push_str(&format!(
        "  {mm_score_s:>7}  {mm_path}  ({mm_loi}, {mm_loj})  MASM_Mega \n"
    ));
    out
}

/// Scores an explicit path through the SW matrix and formats the result.
#[track_caller]
pub fn test_path<F>(
    a: &str,
    b: &str,
    pos_a: uint,
    pos_b: uint,
    path: &str,
    get_local_score: F,
) -> String
where
    F: Fn(uint, uint, &str, &str, uint, uint, &str) -> f32,
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
    let la = a.len() as uint;
    let lb = b.len() as uint;
    let score = get_local_score(la, lb, a, b, pos_a, pos_b, path);
    let score_s = format_g3(score);
    format!("TestPath {score_s} ({pos_a}, {pos_b}) {path}\n")
}

/// Entry point for the `test_sw_aa` command: runs `iters` random
/// comparisons and reports overall agreement counts.
#[track_caller]
pub fn cmd_test_sw_aa<FLocal, FSw, FMm>(
    iters: uint,
    mut get_local_score: FLocal,
    sw_fast_strings_blosum62: FSw,
    sw_fast_masm_mega_prof: FMm,
) -> String
where
    FLocal: FnMut(&str, &str, uint, uint, &str) -> f32,
    FSw: Fn(&str, &str, f32, f32) -> (f32, uint, uint, uint, uint, String) + Copy,
    FMm: Fn(&MASM, &[Vec<byte>], f32, f32) -> (f32, uint, uint, uint, uint, String) + Copy,
{
    let mut stats = TestSwStats::default();
    let mut out = String::new();
    for _iter in 0..iters {
        out.push_str(&test_random_l144(
            &mut stats,
            |a, b, pos_a, pos_b, path| get_local_score(a, b, pos_a, pos_b, path),
            sw_fast_strings_blosum62,
            sw_fast_masm_mega_prof,
        ));
    }
    out.push_str(&format!(
        "N {}, allok {}, pathdiff {}, scorediff {}\n",
        stats.n, stats.n_all_ok, stats.n_path_diff, stats.n_score_diff
    ));
    out
}
