// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Linearly interpolates `y` for `x` within the breakpoint arrays (`xs`, `ys`).
#[track_caller]
pub fn hscore(xs: &[f64], ys: &[f64], x: f64) -> f64 {
    let n = xs.len();
    assert_eq!(ys.len(), n);
    assert!(n > 0);
    if x < xs[0] {
        return ys[0];
    }
    for i in 1..n {
        if x >= xs[i - 1] && x < xs[i] {
            let f = (x - xs[i - 1]) / (xs[i] - xs[i - 1]);
            return f * ys[i] + (1.0 - f) * ys[i - 1];
        }
    }
    ys[n - 1]
}

/// Small self-test driver for `hscore` (unused-code block in the C++ source).
#[track_caller]
pub fn test_l21() -> String {
    let xs = [0.1_f64, 0.5, 0.9];
    let ys = [1.0_f64, 2.0, 3.0];
    let n = xs.len();
    assert_eq!(ys.len(), n);
    let x = 0.0_f64;
    let y = hscore(&xs, &ys, x);
    format!("x={x:.3} y={y:.3}\n")
        .replace(".000", "")
        .replace(".00", "")
}

/// `cmd_test` entry point: enumerates path triples and renders their on-path strings.
#[track_caller]
pub fn cmd_test_l27() -> String {
    let mut out = String::new();
    for (pos_a, pos_b, path) in enum_paths_local_l63(3, 3) {
        let line = on_path_local(pos_a, pos_b, &path);
        out.push_str(&line);
    }
    out
}
