// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Sums forward times backward terminal probabilities to get the
/// log-domain total alignment probability.
#[track_caller]
pub fn calc_total_prob_flat(flat_fwd: &[f32], flat_bwd: &[f32], lx: uint, ly: uint) -> f32 {
    let state_count = HMMSTATE_COUNT as usize;
    let base = state_count * (lx as usize * (ly as usize + 1) + ly as usize);
    assert!(flat_fwd.len() >= base + state_count);
    assert!(flat_bwd.len() >= base + state_count);

    let mut sum = LOG_ZERO;
    for state in 0..state_count {
        sum = log_add_hack(sum, flat_fwd[base + state] + flat_bwd[base + state]);
    }
    sum
}
