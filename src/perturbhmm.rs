// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Multiplies `*p` by a random factor in `[1-var, 1+var]`.
#[track_caller]
pub fn perturb(p: &mut f32, var: f32) {
    assert!((0.0..1.0).contains(&var));
    let pct = randu32() % 100;
    let fract = pct as f32 / 100.0;
    assert!((0.0..=1.0).contains(&fract));
    let lo = 1.0 - var;
    let hi = 1.0 + var;
    let d = lo + (hi - lo) * fract;
    *p *= d;
}

/// Perturbs each transition/emission probability by `params.var` and renormalises.
#[track_caller]
pub fn hmm_params_perturb_probs(params: &mut HMMParams, seed: uint) {
    if seed == 0 {
        return;
    }

    reset_rand(seed);
    assert!(params.var > 0.0 && params.var < 1.0);

    for i in 0..params.trans.len() {
        let pct = randu32() % 100;
        let fract = pct as f32 / 100.0;
        assert!((0.0..=1.0).contains(&fract));
        let lo = 1.0 - params.var;
        let hi = 1.0 + params.var;
        let d = lo + (hi - lo) * fract;
        params.trans[i] *= d;
    }

    let alpha_size = params.alpha.len();
    for i in 0..alpha_size {
        for j in 0..=i {
            let pct = randu32() % 100;
            let fract = pct as f32 / 100.0;
            assert!((0.0..=1.0).contains(&fract));
            let lo = 1.0 - params.var;
            let hi = 1.0 + params.var;
            let d = lo + (hi - lo) * fract;
            let p = params.emits[i][j] * d;
            params.emits[i][j] = p;
            params.emits[j][i] = p;
        }
    }

    hmm_params_normalize(params);
}

/// Returns the mean absolute difference between two HMMParams (transitions, emissions).
#[track_caller]
pub fn hmm_params_compare(hp1: &HMMParams, hp2: &HMMParams) -> (f32, f32) {
    let nt = hp1.trans.len();
    let alpha_size = hp1.alpha.len();
    assert_eq!(hp2.trans.len(), nt);
    assert_eq!(hp1.emits.len(), alpha_size);
    assert_eq!(hp2.emits.len(), alpha_size);

    let mut sum_t = 0.0;
    for i in 0..nt {
        let p1 = hp1.trans[i];
        let p2 = hp2.trans[i];
        sum_t += (p1 - p2).abs();
    }
    let mean_trans_delta = sum_t / nt as f32;

    let mut sum_e = 0.0;
    for i in 0..alpha_size {
        for j in 0..alpha_size {
            let p1 = hp1.emits[i][j];
            let p2 = hp2.emits[i][j];
            sum_e += (p1 - p2).abs();
        }
    }
    let mean_emit_delta = sum_e / (alpha_size * alpha_size) as f32;
    (mean_trans_delta, mean_emit_delta)
}

/// Runs one perturb iteration and returns a one-line progress summary of the deltas.
#[track_caller]
pub fn run1(iter: uint, nucleo: bool) -> String {
    let hp_def = hmm_params_from_defaults(nucleo);

    let mut hp = hmm_params_from_defaults(nucleo);
    hmm_params_perturb_probs(&mut hp, iter);

    let (mean_trans_delta, mean_emit_delta) = hmm_params_compare(&hp_def, &hp);

    progress_log(&format!(
        "Iter {iter}, trans {mean_trans_delta:8.6}, emit {mean_emit_delta:8.6}\n"
    ))
}

/// Implements the `perturbhmm` subcommand: runs `iters` perturbations and logs each.
#[track_caller]
pub fn cmd_perturbhmm(iters: uint, nucleo: bool) -> String {
    let mut out = String::new();
    for iter in 0..iters {
        out.push_str(&run1(iter, nucleo));
    }
    out
}
