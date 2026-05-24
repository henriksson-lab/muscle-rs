// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub struct Sweeper {
    pub param_names: Vec<String>,
    pub scores: Vec<f32>,
    pub qs: Vec<f32>,
    pub tcs: Vec<f32>,
    pub param_values_vec: Vec<Vec<f32>>,
    pub best_score: f32,
    pub best_indexes: Vec<uint>,
    pub grid_noise_fract: f32,
    pub fev_file_name: String,
    pub param_count: uint,
    pub grid_sizes: Vec<uint>,
    pub grid_coords: Vec<uint>,
    pub grid_counter: uint,
    pub grid_count: uint,
    pub spatter_tries_per_iter: uint,
    pub spatter_try: uint,
    pub spatter_iter: uint,
    pub spatter_shrink: f32,
    pub min_delta: f32,
    pub spatter_failed_iter_count: uint,
    pub spatter_seed_indexes: Vec<uint>,
    pub spatter_deltas: Vec<f32>,
    pub start_max_distinct_score_drop: f32,
    pub end_max_distinct_score_drop: f32,
    pub start_min_distinct_param_dist: f32,
    pub end_min_distinct_param_dist: f32,
    pub max_distinct_score_drop: f32,
    pub min_distinct_param_dist: f32,
} // original: Sweeper (muscle/src/sweeper.h)

impl Default for Sweeper {
    fn default() -> Self {
        Self {
            param_names: Vec::new(),
            scores: Vec::new(),
            qs: Vec::new(),
            tcs: Vec::new(),
            param_values_vec: Vec::new(),
            best_score: 0.0,
            best_indexes: Vec::new(),
            grid_noise_fract: 0.0,
            fev_file_name: String::new(),
            param_count: 0,
            grid_sizes: Vec::new(),
            grid_coords: Vec::new(),
            grid_counter: uint::MAX,
            grid_count: uint::MAX,
            spatter_tries_per_iter: uint::MAX,
            spatter_try: uint::MAX,
            spatter_iter: uint::MAX,
            spatter_shrink: f32::MAX,
            min_delta: 0.05,
            spatter_failed_iter_count: 0,
            spatter_seed_indexes: Vec::new(),
            spatter_deltas: Vec::new(),
            start_max_distinct_score_drop: 0.04,
            end_max_distinct_score_drop: 0.01,
            start_min_distinct_param_dist: 1.0,
            end_min_distinct_param_dist: 0.2,
            max_distinct_score_drop: f32::MAX,
            min_distinct_param_dist: f32::MAX,
        }
    }
}

fn sweeper_format_g4(d: f64) -> String {
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
}

/// Draw a pseudo-random value in `[lo, hi]` using a small-prime modulus.
#[track_caller]
pub fn sweeper_get_random_value(_s: &Sweeper, lo: f32, hi: f32) -> f32 {
    assert!(hi > lo);
    const SMALL_PRIME: uint = 997;
    let r = randu32() % SMALL_PRIME;
    let f = r as f32 / SMALL_PRIME as f32;
    assert!((0.0..=1.0).contains(&f));
    let value = lo + f * (hi - lo);
    assert!(value >= lo && value <= hi);
    value
}

/// Draw one random value per parameter from the per-parameter bounds.
#[track_caller]
pub fn sweeper_get_random_values(s: &Sweeper, los: &[f32], his: &[f32]) -> Vec<f32> {
    let mut values = Vec::new();
    for i in 0..s.param_count as usize {
        let value = sweeper_get_random_value(s, los[i], his[i]);
        values.push(value);
    }
    values
}

/// Interpolate the `step`th of `n` grid points between `lo` and `hi`,
/// optionally jittered by `grid_noise_fract`.
#[track_caller]
pub fn sweeper_get_grid_value(s: &Sweeper, lo: f32, hi: f32, step: uint, n: uint) -> f32 {
    assert!(lo < hi);
    assert!(n > 0);
    let mut d_step = step as f32;
    if s.grid_noise_fract != 0.0 {
        assert!(s.grid_noise_fract > 0.0 && s.grid_noise_fract < 1.0);
        let d = sweeper_get_random_value(s, -s.grid_noise_fract, s.grid_noise_fract);
        d_step += d;
    }

    if n == 1 {
        lo
    } else {
        lo + (hi - lo) * d_step / (n - 1) as f32
    }
}

/// Evaluate every point on the Cartesian-product grid defined by
/// `los`, `his`, `sizes` using `get_score`.
#[track_caller]
pub fn sweeper_explore_grid<F>(
    s: &mut Sweeper,
    los: &[f32],
    his: &[f32],
    sizes: &[uint],
    mut get_score: F,
) -> String
where
    F: FnMut(&Sweeper, &[f32]) -> (f64, f64),
{
    assert_eq!(los.len() as uint, s.param_count);
    assert_eq!(his.len() as uint, s.param_count);
    assert_eq!(sizes.len() as uint, s.param_count);

    s.grid_sizes = sizes.to_vec();
    s.grid_count = sizes[0];
    for size in sizes.iter().take(s.param_count as usize).skip(1) {
        s.grid_count *= *size;
    }

    s.grid_coords.clear();
    s.grid_coords.resize(s.param_count as usize, 0);
    let mut fev = String::new();
    s.grid_counter = 0;
    while s.grid_counter < s.grid_count {
        let mut values = Vec::new();
        for param_index in 0..s.param_count as usize {
            let lo = los[param_index];
            let hi = his[param_index];
            let n = sizes[param_index];
            let step = s.grid_coords[param_index];
            let value = sweeper_get_grid_value(s, lo, hi, step, n);
            values.push(value);
        }
        let (q, tc) = get_score(s, &values);
        fev.push_str(&sweeper_run1(s, &values, q, tc));
        let ok = get_next_enum_grid(sizes, &mut s.grid_coords);
        if s.grid_counter + 1 == s.grid_count {
            assert!(!ok);
        } else {
            assert!(ok);
        }
        s.grid_counter += 1;
    }
    s.grid_counter = uint::MAX;
    s.grid_count = uint::MAX;
    fev
}

/// Evaluate `n` randomly sampled parameter vectors via `get_score`.
#[track_caller]
pub fn sweeper_explore_random<F>(
    s: &mut Sweeper,
    los: &[f32],
    his: &[f32],
    n: uint,
    mut get_score: F,
) -> String
where
    F: FnMut(&Sweeper, &[f32]) -> (f64, f64),
{
    let mut fev = String::new();
    for _ in 0..n {
        let values = sweeper_get_random_values(s, los, his);
        let (q, tc) = get_score(s, &values);
        fev.push_str(&sweeper_run1(s, &values, q, tc));
    }
    fev
}

/// Bind the sweeper to an output FEV file (created/truncated here).
#[track_caller]
pub fn sweeper_set_fev(s: &mut Sweeper, file_name: &str) {
    assert!(s.fev_file_name.is_empty());
    std::fs::File::create(file_name).expect("failed to create Sweeper FEV file");
    s.fev_file_name = file_name.to_string();
}

/// Configure the parameter names (and implicitly the parameter count).
#[track_caller]
pub fn sweeper_set_param_names(s: &mut Sweeper, param_names: &[String]) {
    s.param_names = param_names.to_vec();
    s.param_count = s.param_names.len() as uint;
}

/// Record a single benchmark result, update best-score tracking, and
/// append a FEV line (also written to the FEV file if configured).
#[track_caller]
pub fn sweeper_run1(s: &mut Sweeper, param_values: &[f32], q: f64, tc: f64) -> String {
    s.param_values_vec.push(param_values.to_vec());
    let score = tc as f32;
    s.scores.push(score);
    s.qs.push(q as f32);
    s.tcs.push(tc as f32);
    let index = s.scores.len() as uint;
    assert_eq!(s.param_values_vec.len() as uint, index);
    let mut new_best = false;
    if score > s.best_score {
        s.best_score = score;
        s.best_indexes.clear();
        s.best_indexes.push(index);
        new_best = true;
    } else if score == s.best_score {
        s.best_indexes.push(index);
        new_best = true;
    }

    let mut fev = String::new();
    let format_g8 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 8 {
            let raw = format!("{d:.7e}");
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
            let decimals = (7 - exp).max(0) as usize;
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
    fev.push_str(&format!("{index}"));
    fev.push_str(&format!("\tscore={}", format_g8(f64::from(score))));
    fev.push_str(&format!("\tQ={}", format_g8(q)));
    fev.push_str(&format!("\tTC={}", format_g8(tc)));
    for i in 0..s.param_count as usize {
        fev.push_str(&format!(
            "\t{}={}",
            s.param_names[i],
            format_g8(f64::from(param_values[i]))
        ));
    }
    if !s.grid_coords.is_empty() {
        assert_eq!(s.grid_coords.len(), s.param_count as usize);
        assert_eq!(s.grid_sizes.len(), s.param_count as usize);
        for param_index in 0..s.param_count as usize {
            let coord = s.grid_coords[param_index];
            let size = s.grid_sizes[param_index];
            fev.push_str(&format!("\tgridcoord{param_index}={coord}/{size}"));
        }
    }
    if new_best {
        fev.push_str("\tnewbest=yes");
    }
    fev.push('\n');
    if !s.fev_file_name.is_empty() {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&s.fev_file_name)
            .expect("failed to open Sweeper FEV file");
        file.write_all(fev.as_bytes())
            .expect("failed to write Sweeper FEV file");
        file.flush().expect("failed to flush Sweeper FEV file");
    }
    fev
}

/// Indices into `scores` sorted by descending score.
/// C++ `Sweeper::GetSortOrder` uses unstable `QuickSortOrderDesc`; we match
/// that here so tie-broken parameter listings agree across implementations.
#[track_caller]
pub fn sweeper_get_sort_order(s: &Sweeper) -> Vec<uint> {
    let n = s.scores.len();
    assert_eq!(s.param_values_vec.len(), n);
    quick_sort_order_desc_by(n, |a, b| {
        s.scores[a]
            .partial_cmp(&s.scores[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// Print the top-N parameter vectors, including the surrounding "cloud"
/// of similar-score neighbours.
#[track_caller]
pub fn sweeper_log_distinct_top(
    s: &Sweeper,
    _deltas: &[f32],
    max_cloud_size: uint,
    max_cloud_score_dist: f32,
    max_cloud_param_dist: f32,
    n: uint,
) -> String {
    let format_g8 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 8 {
            let raw = format!("{d64:.7e}");
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
            let decimals = (7 - exp).max(0) as usize;
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
    let indexes = sweeper_get_distinct_top_indexes(
        s,
        n,
        s.max_distinct_score_drop,
        s.min_distinct_param_dist,
    );
    let nix = indexes.len() as uint;
    let m = std::cmp::min(nix, n);
    let mut out = String::new();
    out.push_str(&format!("Distinct top ({m} of {nix})\n"));

    out.push(' ');
    out.push('n');
    out.push_str(&format!("  {:>5.5}", "Cloud"));
    out.push_str(&format!("  {:>8.8}", "Q"));
    out.push_str(&format!("  {:>8.8}", "TC"));
    out.push_str(&format!("  {:>8.8}", "ParamDst"));
    out.push_str(&format!("  {:>8.8}", "ScoreDff"));
    for param_index in 0..s.param_count as usize {
        out.push_str(&format!("  {:>12.12}", s.param_names[param_index]));
    }
    out.push('\n');

    for rank in 0..m {
        let index = indexes[rank as usize];
        let params = sweeper_get_params(s, index);
        out.push_str(&format!("{rank:2}"));
        out.push_str(&format!("  {:>5.5}", ""));
        out.push_str(&format!("  {:8.5}", sweeper_get_q(s, index)));
        out.push_str(&format!("  {:8.5}", sweeper_get_tc(s, index)));
        out.push_str(&format!("  {:>8.8}", ""));
        out.push_str(&format!("  {:>8.8}", ""));
        for param_index in 0..s.param_count as usize {
            out.push_str(&format!("  {:>12}", format_g8(params[param_index])));
        }
        out.push('\n');

        let cloud_indexes = sweeper_get_cloud(
            s,
            index,
            max_cloud_size,
            max_cloud_score_dist,
            max_cloud_param_dist,
        );
        let cn = cloud_indexes.len();
        for (ci, index2) in cloud_indexes.iter().copied().enumerate().take(cn) {
            let params2 = sweeper_get_params(s, index2);
            let param_dist = sweeper_get_param_dist(s, index, index2);
            let score_diff = sweeper_get_score_diff(s, index, index2);
            out.push_str(&format!("{rank:2}"));
            out.push_str(&format!("  {ci:5}"));
            out.push_str(&format!("  {:8.5}", sweeper_get_q(s, index2)));
            out.push_str(&format!("  {:8.5}", sweeper_get_tc(s, index2)));
            out.push_str(&format!("  {param_dist:8.5}"));
            out.push_str(&format!("  {score_diff:8.5}"));
            for param_index in 0..s.param_count as usize {
                out.push_str(&format!("  {:>12}", format_g8(params2[param_index])));
            }
            out.push('\n');
        }
        if cn > 0 {
            out.push('\n');
        }
    }
    log(&out);
    out
}

/// Tabulate the score/Q/TC/params for the given parameter-vector indexes.
#[track_caller]
pub fn sweeper_log_indexes(s: &Sweeper, indexes: &[uint]) -> String {
    let mut out = String::new();
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
    let n = indexes.len();
    out.push_str(" Score");
    out.push_str("       Q");
    out.push_str("      TC");
    for i in 0..s.param_count as usize {
        out.push_str(&format!("  {:>12.12}", s.param_names[i]));
    }
    out.push('\n');
    for index in indexes.iter().take(n) {
        let i = *index as usize;
        let score = s.scores[i];
        let q = s.qs[i];
        let tc = s.tcs[i];

        out.push_str(&format!("{score:6.4}"));
        out.push_str(&format!("  {q:6.4}"));
        out.push_str(&format!("  {tc:6.4}"));
        for param_index in 0..s.param_count as usize {
            let value = s.param_values_vec[i][param_index];
            out.push_str(&format!("{:>10}", format_g4(value)));
        }
        out.push('\n');
    }
    log(&out);
    out
}

/// Format the top-`n` highest-scoring parameter vectors as a table.
#[track_caller]
pub fn sweeper_log_top(s: &Sweeper, n: uint) -> String {
    let format_g8 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 8 {
            let raw = format!("{d64:.7e}");
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
            let decimals = (7 - exp).max(0) as usize;
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
    let total = s.scores.len() as uint;
    let order = sweeper_get_sort_order(s);
    let m = std::cmp::min(total, n);
    let mut out = String::new();
    out.push_str("Top params:\n");
    out.push_str(" Score");
    out.push_str("       Q");
    out.push_str("      TC");
    for i in 0..s.param_count as usize {
        out.push_str(&format!("  {:>8.8}", s.param_names[i]));
    }
    out.push('\n');
    for k in 0..m {
        let i = order[k as usize] as usize;
        let score = s.scores[i];
        let q = s.qs[i];
        let tc = s.tcs[i];

        out.push_str(&format!("{score:6.4}"));
        out.push_str(&format!("  {q:6.4}"));
        out.push_str(&format!("  {tc:6.4}"));
        for param_index in 0..s.param_count as usize {
            let value = s.param_values_vec[i][param_index];
            out.push_str(&format!("{:>12}", format_g8(value)));
        }
        out.push('\n');
    }
    log(&out);
    out
}

/// Perturb `center_values` by random fractions of `max_deltas` (with
/// random sign per parameter).
#[track_caller]
pub fn sweeper_get_spattered_values(
    s: &Sweeper,
    center_values: &[f32],
    max_deltas: &[f32],
) -> Vec<f32> {
    assert_eq!(center_values.len(), s.param_count as usize);
    assert_eq!(max_deltas.len(), s.param_count as usize);
    let mut values = Vec::new();
    for param_index in 0..s.param_count as usize {
        let center_value = center_values[param_index];
        let max_delta = max_deltas[param_index];
        const M: uint = 1_000_003;
        let r = randu32() % M;
        let f = r as f32 / M as f32;
        assert!((0.0..=1.0).contains(&f));
        let g = 0.2 + f * 0.8;
        let mut delta = max_delta * g;
        if r % 2 == 0 {
            delta = -delta;
        }
        values.push(center_value + delta);
    }
    values
}

/// Pick up to `max_count` top-scoring indexes that are pairwise separated
/// in parameter space by at least `min_param_dist`.
#[track_caller]
pub fn sweeper_get_distinct_top_indexes(
    s: &Sweeper,
    max_count: uint,
    max_score_drop: f32,
    min_param_dist: f32,
) -> Vec<uint> {
    let mut indexes = Vec::new();
    let n = s.scores.len();
    if n == 0 {
        return indexes;
    }
    let mut order = sweeper_get_sort_order(s);
    assert_eq!(order.len(), n);
    let top_index = order[0];
    indexes.push(top_index);
    for k in 0..n {
        let index = order[k];
        let score_diff = sweeper_get_score_diff(s, top_index, index);
        assert!(score_diff >= 0.0);
        if score_diff > max_score_drop {
            order.truncate(k + 1);
            break;
        }
    }

    let m = order.len();
    for &index in order.iter().take(m) {
        let mut reject_params_too_similar = false;
        for &accepted in &indexes {
            let dist = sweeper_get_param_dist(s, index, accepted);
            if dist < min_param_dist {
                reject_params_too_similar = true;
                break;
            }
        }
        if reject_params_too_similar {
            continue;
        }
        indexes.push(index);
        if indexes.len() as uint >= max_count {
            return indexes;
        }
    }
    indexes
}

/// Spatter (random-walk) search: repeatedly perturb the best parameter
/// vectors, shrinking deltas, until improvement stalls.
#[track_caller]
pub fn sweeper_explore_spatter<F>(
    s: &mut Sweeper,
    start_value_vec: &[Vec<f32>],
    start_deltas: &[f32],
    tries_per_iter: uint,
    max_iters: uint,
    max_fail_iters: uint,
    shrink: f32,
    mut get_score: F,
) -> String
where
    F: FnMut(&Sweeper, &[f32]) -> (f64, f64),
{
    let start_count = start_value_vec.len();
    assert!(start_count > 0);
    let mut fev = String::new();
    for start_values in start_value_vec.iter().take(start_count) {
        assert_eq!(start_values.len(), s.param_count as usize);
        assert_eq!(start_deltas.len(), s.param_count as usize);
        let (q, tc) = get_score(s, start_values);
        fev.push_str(&sweeper_run1(s, start_values, q, tc));
    }
    assert_eq!(s.scores.len(), start_count);
    assert_eq!(s.param_values_vec.len(), start_count);

    s.spatter_seed_indexes.clear();
    s.spatter_seed_indexes.push(0);
    s.spatter_tries_per_iter = tries_per_iter;
    assert!(shrink > 0.0 && shrink < 1.0);
    s.spatter_shrink = shrink;
    s.spatter_deltas = start_deltas.to_vec();
    s.max_distinct_score_drop = s.start_max_distinct_score_drop;
    s.min_distinct_param_dist = s.start_min_distinct_param_dist;

    s.spatter_iter = 0;
    while s.spatter_iter < max_iters {
        if s.spatter_iter == 0 {
            s.spatter_tries_per_iter = 4 * tries_per_iter;
        } else {
            s.spatter_tries_per_iter = tries_per_iter;
        }

        log("\n");
        log(&format!(
            ">>>>> m_SpatterIter={} (max {}, failed {} / {}) <<<<<\n",
            s.spatter_iter, max_iters, s.spatter_failed_iter_count, max_fail_iters
        ));
        log(&format!(
            "m_MaxDistinctScoreDrop = {}, m_MinDistinctParamDist = {}\n",
            sweeper_format_g4(f64::from(s.max_distinct_score_drop)),
            sweeper_format_g4(f64::from(s.min_distinct_param_dist))
        ));
        let _ = sweeper_log_spatter_deltas(s);

        let (improved, iter_fev) = sweeper_spatter_iter(s, |s, values| get_score(s, values));
        fev.push_str(&iter_fev);
        s.spatter_tries_per_iter = tries_per_iter;

        log(&format!(
            ">>>>> Improved = {}\n",
            if improved { 'Y' } else { 'N' }
        ));
        let _ = sweeper_log_top(s, 10);
        let _ = sweeper_log_distinct_top(
            s,
            &s.spatter_deltas,
            8,
            s.max_distinct_score_drop,
            s.min_distinct_param_dist,
            10,
        );

        if improved {
            s.spatter_failed_iter_count = 0;
        } else {
            s.spatter_failed_iter_count += 1;
            if s.spatter_failed_iter_count >= max_fail_iters {
                log("\nConverged, max failed iters\n");
                break;
            }
        }
        sweeper_spatter_update_deltas_shrink(s, s.spatter_shrink);

        if s.max_distinct_score_drop * 0.9 >= s.end_max_distinct_score_drop {
            s.max_distinct_score_drop *= 0.9;
        }
        if s.min_distinct_param_dist * 0.9 >= s.end_min_distinct_param_dist {
            s.min_distinct_param_dist *= 0.8;
        }
        s.spatter_iter += 1;
    }
    fev
}

/// Multiply each spatter delta by `shrink`, clipped at `min_delta`.
#[track_caller]
pub fn sweeper_spatter_update_deltas_shrink(s: &mut Sweeper, shrink: f32) {
    for param_index in 0..s.param_count as usize {
        if s.spatter_deltas[param_index] > s.min_delta {
            s.spatter_deltas[param_index] *= shrink;
        }
    }
}

/// Format spatter deltas in `name+value` form on a single line.
#[track_caller]
pub fn sweeper_printf_spatter_deltas(s: &Sweeper) -> String {
    let mut out = String::new();
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
    for param_index in 0..s.param_count as usize {
        out.push_str(&format!("  {}", s.param_names[param_index]));
        out.push_str(&format!("+{}", format_g4(s.spatter_deltas[param_index])));
    }
    out.push('\n');
    out
}

/// Format spatter deltas with a `Deltas:` prefix for logging.
#[track_caller]
pub fn sweeper_log_spatter_deltas(s: &Sweeper) -> String {
    let mut out = String::from("Deltas:");
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
    for param_index in 0..s.param_count as usize {
        out.push_str(&format!("  {}", s.param_names[param_index]));
        out.push_str(&format!("={}", format_g4(s.spatter_deltas[param_index])));
    }
    out.push('\n');
    log(&out);
    out
}

/// Run one spatter iteration: refresh seed indexes, perturb and score
/// `spatter_tries_per_iter` candidates, return whether the best improved.
#[track_caller]
pub fn sweeper_spatter_iter<F>(s: &mut Sweeper, mut get_score: F) -> (bool, String)
where
    F: FnMut(&Sweeper, &[f32]) -> (f64, f64),
{
    let new_seed_indexes = sweeper_get_distinct_top_indexes(
        s,
        s.spatter_tries_per_iter,
        s.max_distinct_score_drop,
        s.min_distinct_param_dist,
    );
    assert!(!new_seed_indexes.is_empty());
    s.spatter_seed_indexes = new_seed_indexes;

    log(&format!(
        "\nIter {} seeds ({}):\n",
        s.spatter_iter,
        s.spatter_seed_indexes.len()
    ));
    let _ = sweeper_log_indexes(s, &s.spatter_seed_indexes);

    let best_score_start = s.best_score;
    let k_count = s.spatter_seed_indexes.len();
    assert!(k_count > 0);
    let mut k = 0usize;
    let mut fev = String::new();
    s.spatter_try = 0;
    while s.spatter_try < s.spatter_tries_per_iter {
        assert!(k < k_count);
        let index = s.spatter_seed_indexes[k];
        let values = s.param_values_vec[index as usize].clone();
        k %= k_count;

        let spattered_values = sweeper_get_spattered_values(s, &values, &s.spatter_deltas);
        let (q, tc) = get_score(s, &spattered_values);
        fev.push_str(&sweeper_run1(s, &spattered_values, q, tc));
        s.spatter_try += 1;
    }

    let best_score_end = s.best_score;
    let improved = best_score_end - best_score_start > 0.001;
    (improved, fev)
}

/// Return the recorded parameter vector at the given trial index.
pub fn sweeper_get_params(s: &Sweeper, index: uint) -> &[f32] {
    assert!((index as usize) < s.param_values_vec.len());
    &s.param_values_vec[index as usize]
}

/// Euclidean distance between two parameter vectors in the trial set.
pub fn sweeper_get_param_dist(s: &Sweeper, index1: uint, index2: uint) -> f32 {
    let params1 = sweeper_get_params(s, index1);
    let params2 = sweeper_get_params(s, index2);
    let mut sum2 = 0.0;
    for param_index in 0..s.param_count as usize {
        let param1 = params1[param_index];
        let param2 = params2[param_index];
        let d = param1 - param2;
        sum2 += d * d;
    }
    sum2.sqrt()
}

/// Return the recorded score at the given trial index.
pub fn sweeper_get_score_l552(s: &Sweeper, index: uint) -> f32 {
    assert!((index as usize) < s.scores.len());
    s.scores[index as usize]
}

/// Return the recorded Q metric at the given trial index.
pub fn sweeper_get_q(s: &Sweeper, index: uint) -> f32 {
    assert!((index as usize) < s.qs.len());
    s.qs[index as usize]
}

/// Return the recorded TC metric at the given trial index.
pub fn sweeper_get_tc(s: &Sweeper, index: uint) -> f32 {
    assert!((index as usize) < s.tcs.len());
    s.tcs[index as usize]
}

/// Signed score difference `score(index1) - score(index2)`.
pub fn sweeper_get_score_diff(s: &Sweeper, index1: uint, index2: uint) -> f32 {
    let score1 = sweeper_get_score_l552(s, index1);
    let score2 = sweeper_get_score_l552(s, index2);
    score1 - score2
}

/// Absolute score distance between two trials.
pub fn sweeper_get_score_dist(s: &Sweeper, index1: uint, index2: uint) -> f32 {
    let diff = sweeper_get_score_diff(s, index1, index2);
    diff.abs()
}

/// Collect up to `max_size` trial indexes close to `index` in both score
/// and parameter space, sorted by score difference.
#[track_caller]
pub fn sweeper_get_cloud(
    s: &Sweeper,
    index: uint,
    max_size: uint,
    max_score_dist: f32,
    max_param_dist: f32,
) -> Vec<uint> {
    let mut indexes = Vec::new();
    let n = s.param_values_vec.len();
    let mut tmp_indexes = Vec::new();
    let mut score_diffs = Vec::new();
    for index2 in 0..n as uint {
        if index2 == index {
            continue;
        }
        let score_diff = sweeper_get_score_diff(s, index, index2);
        if score_diff.abs() > max_score_dist {
            continue;
        }
        let param_dist = sweeper_get_param_dist(s, index, index2);
        if param_dist > max_param_dist {
            continue;
        }
        tmp_indexes.push(index2);
        score_diffs.push(score_diff);
    }
    let m = tmp_indexes.len();
    if m == 0 {
        return indexes;
    }
    // C++ uses unstable QuickSortOrder on ScoreDiffs (sweeper.cpp:611);
    // match it so the chosen neighbours agree on ties.
    let order = quick_sort_order_by(m, |a, b| {
        score_diffs[a]
            .partial_cmp(&score_diffs[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for i in 0..std::cmp::min(m, max_size as usize) {
        indexes.push(tmp_indexes[order[i] as usize]);
    }
    indexes
}
