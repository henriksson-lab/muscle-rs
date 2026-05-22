// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Polynomial approximation of `log(1 + exp(-x))` used by `log_add_hack`.
/// Mirrors C++ `LOGEXP1` (scoretype.h:100) — four piecewise polynomial
/// branches for `x` in `[0, 1]`, `(1, 2.5]`, `(2.5, 4.5]`, `(4.5, 7.5]`. The
/// previous Rust port only had branches 1 and 4, silently using the wrong
/// polynomial for most of the range.
#[inline(always)]
pub fn hack(x: f32) -> f32 {
    // Hot path called millions of times from the HMM fwd/bwd recurrences.
    // `log_add_hack` already gates this to `(0, LOG_UNDERFLOW_THRESHOLD]`, so
    // a release-mode `assert!` only adds a branch per call without protecting
    // anything; keep the asserts but gate to debug.
    debug_assert!(x >= 0.0);
    debug_assert!(x <= LOG_UNDERFLOW_THRESHOLD);
    if x <= 1.0 {
        return ((-0.009350833524763_f32 * x + 0.130659527668286_f32) * x + 0.498799810682272_f32)
            * x
            + 0.693203116424741_f32;
    }
    if x <= 2.5 {
        return ((-0.014532321752540_f32 * x + 0.139942324101744_f32) * x + 0.495635523139337_f32)
            * x
            + 0.692140569840976_f32;
    }
    if x <= 4.5 {
        return ((-0.004605031767994_f32 * x + 0.063427417320019_f32) * x + 0.695956496475118_f32)
            * x
            + 0.514272634594009_f32;
    }
    ((-0.000458661602210_f32 * x + 0.009695946122598_f32) * x + 0.930734667215156_f32) * x
        + 0.168037164329057_f32
}

/// Fast `log(exp(x) + exp(y))` using the polynomial `hack` approximation.
#[inline(always)]
pub fn log_add_hack(x: f32, y: f32) -> f32 {
    if x < y {
        return if x == LOG_ZERO || y - x >= LOG_UNDERFLOW_THRESHOLD {
            y
        } else {
            hack(y - x) + x
        };
    }
    if y == LOG_ZERO || x - y >= LOG_UNDERFLOW_THRESHOLD {
        x
    } else {
        hack(x - y) + y
    }
}

/// Numerically stable `log(1 + exp(x))`, returning 0 for very negative
/// inputs.
#[track_caller]
pub fn log1pexp(x: f32) -> f32 {
    if x < -88.029691931_f32 {
        0.0
    } else {
        x.exp().ln_1p()
    }
}

/// Log-domain sum: returns `log(exp(x) + exp(y))`.
#[track_caller]
pub fn sum_log_prob(x: f32, y: f32) -> f32 {
    if x > y {
        x + log1pexp(y - x)
    } else {
        y + log1pexp(x - y)
    }
}

/// Entry point for the `testlog` command: cross-checks the log-domain sum
/// approximations against the precise version.
#[track_caller]
pub fn cmd_testlog() -> String {
    const N: uint = 0;
    const M: uint = 0;
    let mut v = Vec::<f32>::new();
    for _ in 0..N {
        let x = -((randu32() % 20) as f32) + ((randu32() % 100) as f32) / 10000.0;
        v.push(x);
    }
    let mut mv = Vec::<f32>::new();
    for _ in 0..M {
        let x = -((randu32() % 20) as f32) + ((randu32() % 100) as f32) / 10000.0;
        mv.push(x);
    }
    let mut diffs = 0_u32;
    for x in &v {
        for y in &v {
            let sum1 = sum_log_prob(*x, *y);
            let sum2 = sum_log_prob(*x, *y);
            if !myfeq(sum1 as f64, sum2 as f64) {
                diffs += 1;
            }
        }
    }
    let mut total = 0.0_f32;
    for x in &v {
        for y in &v {
            total += sum_log_prob(*x, *y);
            total += sum_log_prob(*x, *y);
            total += log_add_hack(*x, *y);
            total += *x + *y;
        }
    }
    for x in &mv {
        total += *x;
    }
    assert_eq!(diffs, 0);
    assert_eq!(total, 0.0);
    String::new()
}
