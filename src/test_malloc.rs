// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Attempts to reserve `n` bytes and reports whether the allocation
/// succeeded.
#[track_caller]
pub fn test_l3(n: usize) -> String {
    let mut v = Vec::<u8>::new();
    let ok = v.try_reserve_exact(n).is_ok();
    format!(
        "{:>10.10}  {}\n",
        mem_bytes_to_str(n as f64),
        if ok { "ok" } else { "failed" }
    )
}

/// Entry point for the `test_malloc` command: probes allocator behaviour at
/// progressively larger sizes.
#[track_caller]
pub fn cmd_test_malloc() -> String {
    let mut out = String::new();
    out.push_str(&test_l3(18_398_178_000));
    let mut n = (uint64::from(uint::MAX) + 1) / 2;
    for _ in 0..8 {
        out.push_str(&test_l3(n as usize));
        n *= 2;
    }
    out
}
