// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Counts how many letters of A (`M` or `D` columns) appear along the path.
#[track_caller]
pub fn get_na(path: &str) -> uint {
    let mut n = 0;
    for c in path.bytes() {
        if c == b'M' || c == b'D' {
            n += 1;
        }
    }
    n
}

/// Counts how many letters of B (`M` or `I` columns) appear along the path.
#[track_caller]
pub fn get_nb(path: &str) -> uint {
    let mut n = 0;
    for c in path.bytes() {
        if c == b'M' || c == b'I' {
            n += 1;
        }
    }
    n
}

/// Recursively extends a partial path and emits every local alignment over the given range.
#[track_caller]
pub fn enum_paths_local_recurse(
    lo_a: uint,
    hi_a: uint,
    lo_b: uint,
    hi_b: uint,
    path: String,
    out: &mut Vec<(uint, uint, String)>,
) {
    let col_count = path.len();
    assert!(col_count > 0);
    assert_eq!(path.as_bytes()[0], b'M');
    assert!(lo_a <= hi_a);
    assert!(lo_b <= hi_b);
    let sub_la = hi_a - lo_a + 1;
    let sub_lb = hi_b - lo_b + 1;
    let na = get_na(&path);
    let nb = get_nb(&path);
    assert!(na <= sub_la && nb <= sub_lb);
    if na == sub_la && nb == sub_lb && path.as_bytes()[col_count - 1] == b'M' {
        out.push((lo_a, lo_b, path));
        return;
    }
    let c = path.as_bytes()[col_count - 1];
    if na < sub_la && nb < sub_lb {
        let mut next = path.clone();
        next.push('M');
        enum_paths_local_recurse(lo_a, hi_a, lo_b, hi_b, next, out);
    }
    if na < sub_la && nb <= sub_lb && c != b'I' {
        let mut next = path.clone();
        next.push('D');
        enum_paths_local_recurse(lo_a, hi_a, lo_b, hi_b, next, out);
    }
    if na <= sub_la && nb < sub_lb && c != b'D' {
        let mut next = path;
        next.push('I');
        enum_paths_local_recurse(lo_a, hi_a, lo_b, hi_b, next, out);
    }
}

/// Enumerates local alignment paths bounded by `[lo_a..=hi_a]` and `[lo_b..=hi_b]`.
#[track_caller]
pub fn enum_paths_local_l57(
    lo_a: uint,
    hi_a: uint,
    _la: uint,
    lo_b: uint,
    hi_b: uint,
    _lb: uint,
) -> Vec<(uint, uint, String)> {
    let mut out = Vec::new();
    enum_paths_local_recurse(lo_a, hi_a, lo_b, hi_b, "M".to_string(), &mut out);
    out
}

/// Enumerates every local alignment path for two sequences of lengths `la` and `lb`.
#[track_caller]
pub fn enum_paths_local_l63(la: uint, lb: uint) -> Vec<(uint, uint, String)> {
    let mut out = Vec::new();
    for lo_a in 0..la {
        for hi_a in lo_a..la {
            for lo_b in 0..lb {
                for hi_b in lo_b..lb {
                    out.extend(enum_paths_local_l57(lo_a, hi_a, la, lo_b, hi_b, lb));
                }
            }
        }
    }
    out
}

/// Recursively extends a partial path and emits every global alignment path.
#[track_caller]
pub fn enum_paths_global_recurse(la: uint, lb: uint, path: String, out: &mut Vec<String>) {
    let col_count = path.len();
    assert!(col_count > 0);
    let na = get_na(&path);
    let nb = get_nb(&path);
    assert!(na <= la && nb <= lb);
    if na == la && nb == lb {
        out.push(path);
        return;
    }
    let c = path.as_bytes()[col_count - 1];
    if na < la && nb < lb {
        let mut next = path.clone();
        next.push('M');
        enum_paths_global_recurse(la, lb, next, out);
    }
    if na < la && nb <= lb && c != b'I' {
        let mut next = path.clone();
        next.push('D');
        enum_paths_global_recurse(la, lb, next, out);
    }
    if na <= la && nb < lb && c != b'D' {
        let mut next = path;
        next.push('I');
        enum_paths_global_recurse(la, lb, next, out);
    }
}

/// Enumerates every global alignment path for two sequences of lengths `la` and `lb`.
#[track_caller]
pub fn enum_paths_global(la: uint, lb: uint) -> Vec<String> {
    let mut out = Vec::new();
    enum_paths_global_recurse(la, lb, "M".to_string(), &mut out);
    enum_paths_global_recurse(la, lb, "D".to_string(), &mut out);
    enum_paths_global_recurse(la, lb, "I".to_string(), &mut out);
    out
}

/// Callback used by `enum_paths_global`: formats a path string for logging.
#[track_caller]
pub fn on_path_global(path: &str) -> String {
    format!("{path}\n")
}

/// Callback used by `enum_paths_local`: formats a path with its A/B coordinate ranges.
#[track_caller]
pub fn on_path_local(pos_a: uint, pos_b: uint, path: &str) -> String {
    let na = get_na(path);
    let nb = get_nb(path);
    let hi_a = pos_a + na - 1;
    let hi_b = pos_b + nb - 1;
    format!("{pos_a:3} .. {hi_a:3},  {pos_b:3} .. {hi_b:3}  {path}\n")
}

/// Test command: enumerates and pretty-prints all local 3x3 alignment paths.
#[track_caller]
pub fn cmd_test_l117() -> String {
    let mut out = String::new();
    for (pos_a, pos_b, path) in enum_paths_local_l63(3, 3) {
        out.push_str(&on_path_local(pos_a, pos_b, &path));
    }
    out
}
