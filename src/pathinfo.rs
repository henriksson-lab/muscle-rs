// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Frees `pi`'s path buffer if it has grown beyond a small-size threshold.
pub fn path_info_free_if_big(pi: &mut PathInfo) {
    if pi.buffer_bytes < 4 * 4096 {
        return;
    }
    pi.path.clear();
    pi.buffer_bytes = 0;
}

/// Ensures `pi`'s buffer holds at least `bytes` characters (with slack).
pub fn path_info_alloc(pi: &mut PathInfo, bytes: uint) {
    path_info_free_if_big(pi);
    if bytes < pi.buffer_bytes {
        return;
    }
    pi.path.clear();
    pi.buffer_bytes = bytes + 128;
}

/// Reallocates the buffer to exactly `bytes`, asserting growth-only.
pub fn path_info_realloc(pi: &mut PathInfo, bytes: uint) {
    assert!(bytes > pi.buffer_bytes);
    assert!((pi.path.len() as uint) < pi.buffer_bytes || pi.path.is_empty());
    pi.buffer_bytes = bytes;
}

/// Ensures the buffer holds `la + lb + 1` characters for an alignment of those lengths.
pub fn path_info_alloc2(pi: &mut PathInfo, la: uint, lb: uint) {
    path_info_free_if_big(pi);
    let bytes = la + lb + 1;
    if bytes > pi.buffer_bytes {
        path_info_alloc(pi, bytes);
    }
}

/// Resets `pi` to an empty path, ensuring a small buffer is allocated.
pub fn path_info_set_empty(pi: &mut PathInfo) {
    path_info_free_if_big(pi);
    if pi.buffer_bytes == 0 {
        path_info_alloc(pi, 4096);
    }
    pi.path.clear();
}

/// Returns `(total, M, D, I)` counts over the path string.
pub fn path_info_get_counts(pi: &PathInfo) -> (uint, uint, uint, uint) {
    let mut m = 0;
    let mut d = 0;
    let mut i_count = 0;
    for c in pi.path.bytes() {
        if c == b'M' {
            m += 1;
        } else if c == b'D' {
            d += 1;
        } else if c == b'I' {
            i_count += 1;
        } else {
            panic!("invalid path char");
        }
    }
    (m + d + i_count, m, d, i_count)
}

/// Reverses the path string in place.
pub fn path_info_reverse(pi: &mut PathInfo) {
    pi.path = pi.path.bytes().rev().map(char::from).collect();
}

/// Appends another path onto `pi`.
pub fn path_info_append_path(pi: &mut PathInfo, other: &PathInfo) {
    if other.path.is_empty() {
        return;
    }
    pi.path.push_str(&other.path);
}

/// Prepends another path onto `pi`.
pub fn path_info_prepend_path(pi: &mut PathInfo, other: &PathInfo) {
    if other.path.is_empty() {
        return;
    }
    pi.path = format!("{}{}", other.path, pi.path);
}

/// Appends a single operation char (`M`/`D`/`I`) to the path.
pub fn path_info_append_char(pi: &mut PathInfo, c: byte) {
    pi.path.push(char::from(c));
}

/// Appends `count` `M` operations.
pub fn path_info_append_ms(pi: &mut PathInfo, count: uint) {
    for _ in 0..count {
        pi.path.push('M');
    }
}

/// Appends `count` `D` operations.
pub fn path_info_append_ds(pi: &mut PathInfo, count: uint) {
    for _ in 0..count {
        pi.path.push('D');
    }
}

/// Appends `count` `I` operations.
pub fn path_info_append_is(pi: &mut PathInfo, count: uint) {
    for _ in 0..count {
        pi.path.push('I');
    }
}

/// Returns the number of leading `I` operations in the path.
pub fn path_info_get_left_i_count(pi: &PathInfo) -> uint {
    for (idx, c) in pi.path.bytes().enumerate() {
        if c != b'I' {
            return idx as uint;
        }
    }
    panic!("all-I path has no non-I edge");
}

/// Removes leading `I` operations and returns how many were removed.
pub fn path_info_trim_left_is(pi: &mut PathInfo) -> uint {
    let left_i_count = path_info_get_left_i_count(pi);
    pi.path.drain(..left_i_count as usize);
    left_i_count
}

/// Removes trailing `I` operations (leaving at least one character).
pub fn path_info_trim_right_is(pi: &mut PathInfo) {
    while pi.path.len() > 1 && pi.path.as_bytes()[pi.path.len() - 1] == b'I' {
        pi.path.pop();
    }
}

/// Run-length encodes the path; flips `D` and `I` to the column-oriented convention.
pub fn path_info_to_ops(pi: &PathInfo) -> (String, Vec<uint>) {
    let mut ops = String::new();
    let mut lengths = Vec::new();
    let mut iter = pi.path.bytes();
    let mut last_op = iter.next().expect("empty path");
    let mut n = 1;
    for op in iter {
        if op == last_op {
            n += 1;
        } else {
            assert!(n > 0);
            ops.push(char::from(last_op));
            lengths.push(n);
            n = 1;
            last_op = op;
        }
    }
    assert!(n > 0);
    ops.push(char::from(last_op));
    lengths.push(n);

    let mut bytes = ops.into_bytes();
    for c in &mut bytes {
        if *c == b'D' {
            *c = b'I';
        } else if *c == b'I' {
            *c = b'D';
        }
    }
    (
        String::from_utf8(bytes).expect("path ops are ASCII"),
        lengths,
    )
}
