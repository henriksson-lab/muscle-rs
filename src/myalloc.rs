// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Allocates an `n * m` byte buffer, asserting no overflow, and updates the global allocation total.
#[track_caller]
pub fn myalloc(n: usize, m: usize) -> Vec<byte> {
    let bytes = n.checked_mul(m).expect("allocation size overflow");
    if n != 0 {
        assert_eq!(bytes / n, m);
    }
    *SUM_ALLOC.lock().unwrap() += bytes as f64;
    vec![0; bytes]
}

/// No-op free wrapper; ownership of the `Vec` releases the memory automatically.
#[track_caller]
pub fn myfree(_p: Option<Vec<byte>>) {}

/// Allocates a buffer and records the bytes against a per-file/line tracking key.
#[track_caller]
pub fn myalloc_track(file_name: &str, line_nr: i32, n: usize, m: usize) -> (Vec<byte>, String) {
    let p = myalloc(n, m);
    let bytes = (n * m) as f64;
    let ptr_name = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    let loc_str = format!("{ptr_name}:{line_nr}");
    let mut tracked = TRACKED_ALLOCS.lock().unwrap();
    *tracked.entry(loc_str.clone()).or_insert(0.0) += bytes;
    (p, loc_str)
}

/// Decrements the tracked allocation total for `loc_str` then releases the buffer.
#[track_caller]
pub fn myfree_track(p: Option<Vec<byte>>, loc_str: &str) {
    if let Some(ref bytes) = p {
        let mut tracked = TRACKED_ALLOCS.lock().unwrap();
        if let Some(size) = tracked.get_mut(loc_str) {
            *size -= bytes.len() as f64;
        }
    }
    myfree(p);
}

/// Parses a leading floating-point size from `s` (skipping anything before `=` if present).
#[track_caller]
pub fn get_size_from_str(s: &str) -> f64 {
    let ptr = s.find('=').map(|i| i + 1).unwrap_or(0);
    let bytes = s[ptr..].as_bytes();
    let mut end = 0;
    while end < bytes.len() && (bytes[end] as char).is_ascii_whitespace() {
        end += 1;
    }
    let start = end;
    if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
        end += 1;
    }
    while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
        end += 1;
    }
    if end < bytes.len() && bytes[end] == b'.' {
        end += 1;
        while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
            end += 1;
        }
    }
    if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
        let exp = end;
        end += 1;
        if end < bytes.len() && (bytes[end] == b'+' || bytes[end] == b'-') {
            end += 1;
        }
        let digits = end;
        while end < bytes.len() && (bytes[end] as char).is_ascii_digit() {
            end += 1;
        }
        if digits == end {
            end = exp;
        }
    }
    if start == end {
        return 0.0;
    }
    std::str::from_utf8(&bytes[start..end])
        .ok()
        .and_then(|x| x.parse::<f64>().ok())
        .unwrap_or(0.0)
}

/// Logs the top tracked allocations (per file:line) and the total leaked bytes.
#[track_caller]
pub fn log_allocs_l132() -> String {
    let tracked = TRACKED_ALLOCS.lock().unwrap();
    let mut rows: Vec<(String, f64)> = tracked
        .iter()
        .filter(|(_, bytes)| **bytes != 0.0)
        .map(|(loc, bytes)| (loc.clone(), *bytes))
        .collect();
    rows.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = String::new();
    let mut sum_leak = 0.0;
    for (i, (loc, bytes)) in rows.iter().enumerate() {
        if i < 100 {
            out.push_str(&format!("{:>10.10}  {loc}\n", mem_bytes_to_str(*bytes)));
        }
        sum_leak += *bytes;
    }
    out.push_str(&format!(
        "\n****** TOTAL LEAK {} *******\n\n",
        mem_bytes_to_str(sum_leak)
    ));
    out
}

/// Stub variant of the alloc log dump (returns an empty string).
#[track_caller]
pub fn log_allocs_l171() -> String {
    String::new()
}
