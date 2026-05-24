// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const DEFAULT_PERTURB_VAR: f32 = 0.25;

#[derive(Clone, Debug, PartialEq)]
pub struct M3AlnParams {
    pub subst_mx_letter: [[f32; 20]; 20],
    pub gap_open: f32,
    pub center_added: bool,
    pub perturb_seed: uint,
    pub perturb_subst_mx_delta: f32,
    pub perturb_gap_params_delta: f32,
    pub perturb_dist_mx_delta: f32,
    pub perturb_subst_mx_done: bool,
    pub perturb_gap_params_done: bool,
    pub nuc_match_score: f32,
    pub nuc_mismatch_score: f32,
    pub term_gap_open: f32,
    pub term_gap_ext: f32,
    pub ready: bool,
    pub linkage: String,
    pub tree_iters: uint,
    pub kmer_dist: String,
    pub center: f32,
    pub min_std_rand: uint,
} // original: M3AlnParams (muscle/src/m3alnparams.h)

impl Default for M3AlnParams {
    fn default() -> Self {
        Self {
            subst_mx_letter: [[0.0; 20]; 20],
            gap_open: f32::MAX,
            center_added: false,
            perturb_seed: 0,
            perturb_subst_mx_delta: 0.0,
            perturb_gap_params_delta: 0.0,
            perturb_dist_mx_delta: 0.0,
            perturb_subst_mx_done: false,
            perturb_gap_params_done: false,
            nuc_match_score: f32::MAX,
            nuc_mismatch_score: f32::MAX,
            term_gap_open: f32::MAX,
            term_gap_ext: f32::MAX,
            ready: false,
            linkage: "min".to_string(),
            tree_iters: 1,
            kmer_dist: "66".to_string(),
            center: f32::MAX,
            min_std_rand: 1,
        }
    }
}

/// Perturbs `param` by a random signed delta bounded by `max_delta` (const-overload variant).
#[track_caller]
pub fn m3_aln_params_perturb1_l6(ap: &mut M3AlnParams, param: &mut f32, max_delta: f32) {
    let sign = if m3_aln_params_get_rand(ap) % 2 == 0 {
        -1.0
    } else {
        1.0
    };
    let small_prime = 997;
    let f = (m3_aln_params_get_rand(ap) % small_prime) as f32 / small_prime as f32;
    assert!((0.0..=1.0).contains(&f));
    let delta = sign * max_delta * f;
    *param += delta;
}

/// Perturbs `param` by a random signed delta bounded by `max_delta` (mutating-overload variant).
#[track_caller]
pub fn m3_aln_params_perturb1_l17(ap: &mut M3AlnParams, param: &mut f32, max_delta: f32) {
    let sign = if m3_aln_params_get_rand(ap) % 2 == 0 {
        -1.0
    } else {
        1.0
    };
    let small_prime = 997;
    let f = (m3_aln_params_get_rand(ap) % small_prime) as f32 / small_prime as f32;
    assert!((0.0..=1.0).contains(&f));
    let delta = sign * max_delta * f;
    *param += delta;
}

/// Renders a human-readable dump of the M3 alignment parameters.
#[track_caller]
pub fn m3_aln_params_print(ap: &M3AlnParams) -> String {
    let state = ALPHA_STATE.lock().unwrap();
    let mut out = String::new();
    let format_g = |d: f32, precision: usize| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= precision as i32 {
            let raw = format!("{d64:.p$e}", p = precision - 1);
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
            let decimals = (precision as i32 - 1 - exp).max(0) as usize;
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
    out.push_str(&format!(
        "m_GapOpen={} m_Center={}",
        format_g(ap.gap_open, 6),
        format_g(ap.center, 6)
    ));
    out.push_str(&format!(" linkage={}", ap.linkage));
    out.push_str(&format!(" treeiters={}", ap.tree_iters));
    out.push_str(&format!(" kmerdist={}\n", ap.kmer_dist));
    out.push_str(&format!(" perturb({})", ap.perturb_seed));
    if ap.perturb_seed != 0 {
        out.push_str(&format!(
            " substmx={}, gapparams={}, distmx={}",
            format_g(ap.perturb_subst_mx_delta, 3),
            format_g(ap.perturb_gap_params_delta, 3),
            format_g(ap.perturb_dist_mx_delta, 3)
        ));
    }
    out.push('\n');
    for i in 0..3 {
        out.push_str(&format!("  SubstMx[{}]: ", state.letter_to_char[i]));
        for j in 0..8 {
            out.push_str(&format!(
                " {}={:8.4}",
                state.letter_to_char[j], ap.subst_mx_letter[i][j]
            ));
        }
        out.push('\n');
    }
    out.push('\n');
    for i in 0..3 {
        out.push_str(&format!("SubstMx-C[{}]: ", state.letter_to_char[i]));
        for j in 0..8 {
            out.push_str(&format!(
                " {}={:8.4}",
                state.letter_to_char[j],
                ap.subst_mx_letter[i][j] - ap.center
            ));
        }
        out.push('\n');
    }
    log(&out);
    out
}

/// Adds a constant `x` to every entry of the substitution matrix (used to center scores).
#[track_caller]
pub fn m3_aln_params_add_center(ap: &mut M3AlnParams, x: f32) {
    if x == 0.0 {
        return;
    }
    assert!(!ap.center_added);
    for i in 0..20 {
        for j in 0..20 {
            ap.subst_mx_letter[i][j] += x;
        }
    }
    ap.center_added = true;
}

/// Initializes the parameters from a BLOSUM matrix and applies any requested perturbations.
#[track_caller]
pub fn m3_aln_params_set_blosum(
    ap: &mut M3AlnParams,
    pct_id: uint,
    n: uint,
    gap_open: f32,
    center: f32,
    perturb_seed: uint,
    perturb_subst_mx_delta: f32,
    perturb_gap_params_delta: f32,
    perturb_dist_mx_delta: f32,
) {
    m3_aln_params_set_blosum_with_log(
        ap,
        pct_id,
        n,
        gap_open,
        center,
        perturb_seed,
        perturb_subst_mx_delta,
        perturb_gap_params_delta,
        perturb_dist_mx_delta,
        false,
    );
}

/// Initializes the parameters from a BLOSUM matrix and optionally logs like C++ `DoLog`.
#[track_caller]
pub fn m3_aln_params_set_blosum_with_log(
    ap: &mut M3AlnParams,
    pct_id: uint,
    n: uint,
    gap_open: f32,
    center: f32,
    perturb_seed: uint,
    perturb_subst_mx_delta: f32,
    perturb_gap_params_delta: f32,
    perturb_dist_mx_delta: f32,
    do_log: bool,
) -> Option<String> {
    set_alpha_lc(false);
    ap.perturb_seed = perturb_seed;
    ap.perturb_subst_mx_delta = perturb_subst_mx_delta;
    ap.perturb_gap_params_delta = perturb_gap_params_delta;
    ap.perturb_dist_mx_delta = perturb_dist_mx_delta;
    ap.center_added = false;
    ap.perturb_gap_params_done = false;
    ap.perturb_subst_mx_done = false;
    ap.subst_mx_letter = get_subst_mx_letter_blosum(pct_id);
    let (default_gap_open, default_center) = get_gap_params_blosum(pct_id, n);
    ap.gap_open = if gap_open != f32::MAX {
        gap_open
    } else {
        default_gap_open
    };
    ap.center = if center != f32::MAX {
        center
    } else {
        default_center
    };
    m3_aln_params_add_center(ap, ap.center);
    m3_aln_params_perturb_my_params(ap);
    ap.ready = true;
    if do_log {
        Some(m3_aln_params_print(ap))
    } else {
        None
    }
}

/// Overwrites the substitution matrix and gap parameters with externally supplied values.
#[track_caller]
pub fn m3_aln_params_update_mx(
    ap: &mut M3AlnParams,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
    center: f32,
) {
    let _ = m3_aln_params_update_mx_with_log(ap, subst_mx_letter, gap_open, center, false);
}

/// Overwrites the substitution matrix and optionally logs like C++ `DoLog`.
#[track_caller]
pub fn m3_aln_params_update_mx_with_log(
    ap: &mut M3AlnParams,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
    center: f32,
    do_log: bool,
) -> Option<String> {
    set_alpha_lc(false);
    for i in 0..20 {
        for j in 0..20 {
            ap.subst_mx_letter[i][j] = subst_mx_letter[i][j];
        }
    }
    ap.perturb_subst_mx_done = false;
    ap.perturb_gap_params_done = false;
    ap.center_added = false;
    ap.gap_open = gap_open;
    ap.center = center;
    m3_aln_params_add_center(ap, ap.center);
    ap.ready = true;
    if do_log {
        Some(m3_aln_params_print(ap))
    } else {
        None
    }
}

/// Builds the M3 parameter block from command-line options (subst-mx file or BLOSUM).
#[track_caller]
pub fn m3_aln_params_set_from_cmd_line(
    ap: &mut M3AlnParams,
    is_nucleo: bool,
    do_log: bool,
    subst_mx_file_name: Option<&str>,
    gap_open: Option<f32>,
    center: Option<f32>,
    blosum_pct: Option<uint>,
    blosum_param_set: Option<uint>,
    perturb_seed: Option<uint>,
    linkage: Option<&str>,
    kmer_dist: Option<&str>,
    tree_iters: Option<uint>,
) -> Option<String> {
    assert!(!is_nucleo);
    set_alpha_lc(is_nucleo);

    if let Some(file_name) = subst_mx_file_name {
        assert!(gap_open.is_some());
        assert!(center.is_some());
        ap.subst_mx_letter = read_subst_mx_letter_from_file(file_name);
        ap.gap_open = gap_open.unwrap();
        ap.center = center.unwrap();
        ap.perturb_subst_mx_done = false;
        ap.perturb_gap_params_done = false;
        ap.center_added = false;
    } else {
        ap.perturb_subst_mx_done = false;
        ap.perturb_gap_params_done = false;
        ap.center_added = false;

        let blosum_pct = blosum_pct.unwrap_or(62);
        let blosum_param_set = blosum_param_set.unwrap_or(0);
        ap.subst_mx_letter = get_subst_mx_letter_blosum(blosum_pct);
        let (default_gap_open, default_center) =
            get_gap_params_blosum(blosum_pct, blosum_param_set);
        ap.gap_open = gap_open.unwrap_or(default_gap_open);
        ap.center = center.unwrap_or(default_center);
    }

    m3_aln_params_add_center(ap, ap.center);

    if let Some(seed) = perturb_seed {
        ap.perturb_seed = seed;
        if ap.perturb_seed != 0 {
            ap.perturb_subst_mx_delta = 0.1;
            ap.perturb_gap_params_delta = 0.1;
            ap.perturb_dist_mx_delta = 0.1;
        }
        m3_aln_params_perturb_my_params(ap);
    }

    ap.linkage = "biased".to_string();
    if let Some(linkage) = linkage {
        ap.linkage = linkage.to_string();
    }

    ap.kmer_dist = "66".to_string();
    if let Some(kmer_dist) = kmer_dist {
        ap.kmer_dist = kmer_dist.to_string();
    }

    ap.tree_iters = 1;
    if let Some(tree_iters) = tree_iters {
        ap.tree_iters = tree_iters;
    }

    ap.ready = true;

    if do_log {
        Some(m3_aln_params_print(ap))
    } else {
        None
    }
}

/// Adds random noise to each substitution-matrix entry using the configured seed.
#[track_caller]
pub fn m3_aln_params_perturb_subst_mx(ap: &mut M3AlnParams) {
    if ap.perturb_seed == 0 || ap.perturb_subst_mx_delta == 0.0 {
        return;
    }
    assert!(!ap.perturb_subst_mx_done);
    for i in 0..20 {
        for j in 0..20 {
            let mut score = ap.subst_mx_letter[i][j];
            m3_aln_params_perturb1_l17(ap, &mut score, ap.perturb_subst_mx_delta);
            ap.subst_mx_letter[i][j] = score;
        }
    }
    ap.perturb_subst_mx_done = true;
}

/// Adds random noise to the gap-open and center parameters.
#[track_caller]
pub fn m3_aln_params_perturb_gap_params(ap: &mut M3AlnParams) {
    if ap.perturb_gap_params_delta == 0.0 {
        return;
    }
    assert!(!ap.perturb_gap_params_done);
    let mut gap_open = ap.gap_open;
    m3_aln_params_perturb1_l17(ap, &mut gap_open, ap.perturb_gap_params_delta);
    ap.gap_open = gap_open;
    let mut center = ap.center;
    m3_aln_params_perturb1_l17(ap, &mut center, ap.perturb_gap_params_delta);
    ap.center = center;
    ap.perturb_gap_params_done = true;
}

/// Reseeds and perturbs both the substitution matrix and the gap parameters.
#[track_caller]
pub fn m3_aln_params_perturb_my_params(ap: &mut M3AlnParams) {
    if ap.perturb_seed == 0 {
        return;
    }
    m3_aln_params_init_perturb(ap, ap.perturb_seed);
    m3_aln_params_perturb_gap_params(ap);
    m3_aln_params_perturb_subst_mx(ap);
}

/// Seeds the embedded minstd random generator (non-zero modulo 2^31-1).
#[track_caller]
pub fn m3_aln_params_init_perturb(ap: &mut M3AlnParams, seed: uint) {
    let mut s = seed % 2_147_483_647;
    if s == 0 {
        s = 1;
    }
    ap.min_std_rand = s;
}

/// Returns the next value from the embedded minstd random generator.
#[track_caller]
pub fn m3_aln_params_get_rand(ap: &mut M3AlnParams) -> uint {
    let product = (ap.min_std_rand as u64) * 48271_u64;
    ap.min_std_rand = (product % 2_147_483_647_u64) as uint;
    ap.min_std_rand
}

/// Perturbs each off-diagonal entry of a symmetric distance matrix in place.
#[track_caller]
pub fn m3_aln_params_perturb_dist_mx(ap: &mut M3AlnParams, dist_mx: &mut [Vec<f32>]) {
    if ap.perturb_seed == 0 || ap.perturb_dist_mx_delta == 0.0 {
        return;
    }
    let n = dist_mx.len();
    for i in 0..n {
        assert_eq!(dist_mx[i].len(), n);
        for j in 0..i {
            let mut d = dist_mx[i][j];
            m3_aln_params_perturb1_l6(ap, &mut d, ap.perturb_dist_mx_delta);
            dist_mx[i][j] = d;
            dist_mx[j][i] = d;
        }
    }
}
