// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

static LAST_SUM_ALLOC: std::sync::Mutex<f64> = std::sync::Mutex::new(0.0);
static ALLOC_WARNING_DONE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

unsafe extern "C" {
    fn malloc(size: usize) -> *mut std::ffi::c_void;
    fn free(ptr: *mut std::ffi::c_void);
}

/// Owning raw allocation returned by the C++-equivalent `myalloc_` wrapper.
///
/// The pointed-to bytes are intentionally uninitialized, matching `malloc`.
/// Callers must initialize bytes before reading them.
pub struct MyAllocRaw {
    ptr: std::ptr::NonNull<byte>,
    len: usize,
}

impl MyAllocRaw {
    #[track_caller]
    pub fn as_mut_ptr(&mut self) -> *mut byte {
        self.ptr.as_ptr()
    }

    #[track_caller]
    pub fn as_ptr(&self) -> *const byte {
        self.ptr.as_ptr()
    }

    #[track_caller]
    pub fn len(&self) -> usize {
        self.len
    }

    #[track_caller]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl Drop for MyAllocRaw {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                free(self.ptr.as_ptr().cast::<std::ffi::c_void>());
            }
        }
    }
}

fn account_cpp_allocation(bytes: usize) {
    if ALLOC_WARNING_DONE.load(std::sync::atomic::Ordering::Relaxed) {
        return;
    }

    let mut sum_alloc = SUM_ALLOC.lock().unwrap();
    let mut last_sum_alloc = LAST_SUM_ALLOC.lock().unwrap();
    *sum_alloc += bytes as f64;
    if *sum_alloc - *last_sum_alloc > 1e9 {
        let ram = get_phys_mem_bytes_l990();
        let alloced = get_mem_use_bytes_l1008();
        if alloced > ram * 0.9 {
            eprintln!(
                "\n\n=========================================================\n\
                 WARNING: {} Gb memory allocated so far.\n\
                 This process may crash soon, or run slowly due to paging.\n\
                 Typical cause of excessive memory use is large dataset\n\
                 with long sequences (more than around 15k letters).\n\
                 =========================================================\n",
                float_to_str_l1385(alloced / 1e9)
            );
            ALLOC_WARNING_DONE.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    }
    *last_sum_alloc = *sum_alloc;
}

/// C++-equivalent raw `myalloc_`: returns malloc-owned uninitialized bytes.
#[track_caller]
pub fn myalloc_(n: usize, m: usize) -> MyAllocRaw {
    let bytes = n.wrapping_mul(m);
    if n != 0 {
        assert_eq!(bytes / n, m);
    }
    account_cpp_allocation(bytes);

    if bytes == 0 {
        return MyAllocRaw {
            ptr: std::ptr::NonNull::dangling(),
            len: 0,
        };
    }

    let ptr = unsafe { malloc(bytes).cast::<byte>() };
    let Some(ptr) = std::ptr::NonNull::new(ptr) else {
        let alloced = get_mem_use_bytes_l1008();
        die(&format!(
            "Out of memory mymalloc({}), alloced {} bytes",
            float_to_str_l1385(bytes as f64),
            float_to_str_l1385(alloced)
        ));
    };

    MyAllocRaw { ptr, len: bytes }
}

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
