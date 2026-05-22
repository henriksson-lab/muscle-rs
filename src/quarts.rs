// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Quarts {
    pub min: uint,
    pub lo_q: uint,
    pub med: uint,
    pub hi_q: uint,
    pub max: uint,
    pub total: uint,
    pub avg: f64,
} // original: Quarts (muscle/src/quarts.h)

#[derive(Clone, Debug, Default, PartialEq)]
pub struct QuartsFloat {
    pub min: f32,
    pub lo_q: f32,
    pub med: f32,
    pub hi_q: f32,
    pub max: f32,
    pub total: f32,
    pub avg: f32,
    pub std_dev: f32,
} // original: QuartsFloat (muscle/src/quarts.h)

/// Compute min, quartiles, max, total and average for a vector of unsigned ints.
#[track_caller]
pub fn get_quarts(v: &[uint]) -> Quarts {
    let n = v.len();
    let mut q = Quarts::default();
    if n == 0 {
        return q;
    }
    let mut vs = v.to_vec();
    vs.sort_unstable();
    for x in &vs {
        q.total += *x;
    }
    q.min = vs[0];
    q.lo_q = vs[n / 4];
    q.med = vs[n / 2];
    q.hi_q = vs[(3 * n) / 4];
    q.max = vs[n - 1];
    q.avg = f64::from(q.total) / n as f64;
    q
}

/// Compute min, quartiles, max, total, mean and standard deviation for a
/// vector of floats.
#[track_caller]
pub fn get_quarts_float(v: &[f32]) -> QuartsFloat {
    let n = v.len();
    let mut q = QuartsFloat::default();
    if n == 0 {
        return q;
    }
    let mut vs = v.to_vec();
    vs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for x in &vs {
        q.total += *x;
    }
    let mean = q.total / n as f32;
    let mut sumd = 0.0;
    for x in &vs {
        let d = (*x - mean) * (*x - mean);
        sumd += d;
    }
    q.min = vs[0];
    q.lo_q = vs[n / 4];
    q.med = vs[n / 2];
    q.hi_q = vs[(3 * n) / 4];
    q.max = vs[n - 1];
    q.avg = mean;
    q.std_dev = (sumd / n as f32).sqrt();
    q
}
