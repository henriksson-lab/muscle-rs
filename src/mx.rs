// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct SeqData; // original: SeqData (muscle/src/mx.h)

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MxBase {
    pub name: String,
    pub row_count: uint,
    pub col_count: uint,
    pub allocated_row_count: uint,
    pub allocated_col_count: uint,
    pub data: Vec<Vec<f32>>,
} // original: MxBase (muscle/src/mx.h)

#[derive(Clone, Debug, Default)]
pub(crate) struct MxBaseCounts {
    pub(crate) alloc_count: uint,
    pub(crate) zero_alloc_count: uint,
    pub(crate) grow_alloc_count: uint,
    pub(crate) total_bytes: f64,
    pub(crate) max_bytes: f64,
}

pub(crate) static MX_BASE_COUNTS: std::sync::Mutex<MxBaseCounts> =
    std::sync::Mutex::new(MxBaseCounts {
        alloc_count: 0,
        zero_alloc_count: 0,
        grow_alloc_count: 0,
        total_bytes: 0.0,
        max_bytes: 0.0,
    });

fn mx_base_total_bytes(row_count: uint, col_count: uint) -> f64 {
    let row_ptr_bytes = std::mem::size_of::<*const f32>() as f64 * f64::from(row_count);
    let data_bytes =
        std::mem::size_of::<f32>() as f64 * f64::from(row_count) * f64::from(col_count);
    row_ptr_bytes + data_bytes
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mx {
    pub name: String,
    pub row_count: uint,
    pub col_count: uint,
    pub data: Vec<Vec<f32>>,
} // original: Mx (muscle/src/mx.h)

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CppMxBase {
    pub name: String,
    pub row_count: uint,
    pub col_count: uint,
    pub allocated_row_count: uint,
    pub allocated_col_count: uint,
}

pub trait CppMxBaseVirtual {
    fn cpp_base(&self) -> &CppMxBase;
    fn cpp_base_mut(&mut self) -> &mut CppMxBase;
    fn cpp_get_total_bytes(&self) -> uint;
    fn cpp_alloc_data(&mut self, row_count: uint, col_count: uint);
    fn cpp_free_data(&mut self);
}

pub struct CppMxRaw {
    pub base: CppMxBase,
    type_size: uint,
    buffer: Option<MyAllocRaw>,
}

impl CppMxRaw {
    #[track_caller]
    pub fn new(type_size: uint) -> Self {
        assert!(type_size > 0);
        Self {
            base: CppMxBase::default(),
            type_size,
            buffer: None,
        }
    }

    #[track_caller]
    pub fn type_size(&self) -> uint {
        self.type_size
    }

    #[track_caller]
    pub fn raw_buffer_len(&self) -> usize {
        self.buffer.as_ref().map(MyAllocRaw::len).unwrap_or(0)
    }

    #[track_caller]
    pub fn row_ptr_bytes(&self) -> usize {
        std::mem::size_of::<*mut byte>() * self.base.allocated_row_count as usize
    }

    #[track_caller]
    pub fn row_ptr_value(&self, row: uint) -> *mut byte {
        assert!(row < self.base.allocated_row_count);
        let Some(buffer) = &self.buffer else {
            panic!("CppMxRaw row pointer requested before allocation");
        };
        unsafe { *buffer.as_ptr().cast::<*mut byte>().add(row as usize) }
    }

    #[track_caller]
    pub fn cell_ptr(&mut self, row: uint, col: uint) -> *mut byte {
        assert!(row < self.base.row_count);
        assert!(col < self.base.col_count);
        let row_ptr = self.row_ptr_value(row);
        unsafe { row_ptr.add(col as usize * self.type_size as usize) }
    }

    #[track_caller]
    pub fn cell_offset(&self, row: uint, col: uint) -> usize {
        assert!(row < self.base.allocated_row_count);
        assert!(col < self.base.allocated_col_count);
        self.row_ptr_bytes()
            + row as usize * self.base.allocated_col_count as usize * self.type_size as usize
            + col as usize * self.type_size as usize
    }
}

impl Default for CppMxRaw {
    fn default() -> Self {
        Self::new(std::mem::size_of::<f32>() as uint)
    }
}

impl CppMxBaseVirtual for CppMxRaw {
    fn cpp_base(&self) -> &CppMxBase {
        &self.base
    }

    fn cpp_base_mut(&mut self) -> &mut CppMxBase {
        &mut self.base
    }

    fn cpp_get_total_bytes(&self) -> uint {
        self.base
            .allocated_row_count
            .wrapping_mul(self.base.allocated_col_count)
            .wrapping_mul(self.type_size)
            .wrapping_add(
                self.base
                    .allocated_row_count
                    .wrapping_mul(std::mem::size_of::<*mut byte>() as uint),
            )
    }

    fn cpp_alloc_data(&mut self, row_count: uint, col_count: uint) {
        let row_ptr_bytes = std::mem::size_of::<*mut byte>() as f64 * f64::from(row_count);
        if row_ptr_bytes > f64::from(uint::MAX - 16) {
            die(&format!(
                "Mx::AllocData Rows={}, sizeof(T *)={}, row {} bytes too big",
                row_count,
                std::mem::size_of::<*mut byte>(),
                mem_bytes_to_str(row_ptr_bytes)
            ));
        }

        let data_bytes = f64::from(self.type_size) * f64::from(row_count) * f64::from(col_count);
        if data_bytes > f64::from(uint::MAX - 16) {
            die(&format!(
                "Mx::AllocData Rows={}, Cols={}, sizeof(T)={}, data {} bytes too big",
                row_count,
                col_count,
                self.type_size,
                mem_bytes_to_str(data_bytes)
            ));
        }

        let row_bytes = std::mem::size_of::<*mut byte>() * row_count as usize;
        let data_bytes = self.type_size as usize * row_count as usize * col_count as usize;
        let mut buffer = myalloc_(1, row_bytes.wrapping_add(data_bytes));
        let base = unsafe { buffer.as_mut_ptr().add(row_bytes) };
        for i in 0..row_count as usize {
            let row_ptr = unsafe { base.add(i * col_count as usize * self.type_size as usize) };
            unsafe {
                *buffer.as_mut_ptr().cast::<*mut byte>().add(i) = row_ptr;
            }
        }
        self.buffer = Some(buffer);
        self.base.allocated_row_count = row_count;
        self.base.allocated_col_count = col_count;
    }

    fn cpp_free_data(&mut self) {
        self.buffer = None;
        self.base.row_count = 0;
        self.base.col_count = 0;
        self.base.allocated_row_count = 0;
        self.base.allocated_col_count = 0;
    }
}

#[track_caller]
pub fn cpp_mx_base_alloc<M: CppMxBaseVirtual>(
    mx: &mut M,
    row_count: uint,
    col_count: uint,
    name: &str,
) {
    mx.cpp_base_mut().name = name.to_string();
    let mut counts = MX_BASE_COUNTS.lock().unwrap();
    counts.alloc_count += 1;
    if mx.cpp_base().allocated_row_count == 0 {
        counts.zero_alloc_count += 1;
    }

    if row_count > mx.cpp_base().allocated_row_count
        || col_count > mx.cpp_base().allocated_col_count
    {
        if mx.cpp_base().allocated_row_count > 0 {
            counts.grow_alloc_count += 1;
        }
        counts.total_bytes -= f64::from(mx.cpp_get_total_bytes());
        mx.cpp_free_data();

        let n = std::cmp::max(
            row_count.wrapping_add(16),
            mx.cpp_base().allocated_row_count,
        );
        let m = std::cmp::max(
            col_count.wrapping_add(16),
            mx.cpp_base().allocated_col_count,
        );
        mx.cpp_alloc_data(n, m);

        counts.total_bytes += f64::from(mx.cpp_get_total_bytes());
        if counts.total_bytes > counts.max_bytes {
            counts.max_bytes = counts.total_bytes;
        }
    }

    mx.cpp_base_mut().name = name.to_string();
    assert!(row_count <= mx.cpp_base().allocated_row_count);
    assert!(col_count <= mx.cpp_base().allocated_col_count);
    let base = mx.cpp_base_mut();
    base.row_count = row_count;
    base.col_count = col_count;
}

/// Parses `s` as a float, takes its natural log, and formats the result for matrix display.
pub fn logize_str(s: &str) -> String {
    let format_g4 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d:.3e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (3 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let f = (s.parse::<f64>().unwrap_or(0.0).ln()) as f32;
    if f == -8e8_f32 {
        format!("{:>12.12}", "?")
    } else if f < -9e9_f32 / 2.0 {
        format!("{:>12.12}", "*")
    } else if f == 0.0 {
        format!("{:>12.12}", ".")
    } else if (-1e5_f32..=1e5_f32).contains(&f) {
        format!("{:12.5}", f)
    } else {
        format!("{:>12}", format_g4(f64::from(f)))
    }
}

/// Parses `s` as a float, exponentiates it, and formats the result for matrix display.
pub fn expize_str(s: &str) -> String {
    let format_g4 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d:.3e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (3 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let f = (s.parse::<f64>().unwrap_or(0.0).exp()) as f32;
    if f == -8e8_f32 {
        format!("{:>12.12}", "?")
    } else if f < -9e9_f32 / 2.0 {
        format!("{:>12.12}", "*")
    } else if f == 0.0 {
        format!("{:>12.12}", ".")
    } else if (-1e5_f32..=1e5_f32).contains(&f) {
        format!("{:12.5}", f)
    } else {
        format!("{:>12}", format_g4(f64::from(f)))
    }
}

/// Construction hook for an `MxBase` instance; placeholder for accounting/instrumentation.
pub fn mx_base_on_ctor(mx: &mut MxBase) {
    let _name_len = mx.name.len();
    let _shape = (mx.row_count, mx.col_count);
}

/// Destruction hook for an `MxBase` instance; placeholder for accounting/instrumentation.
pub fn mx_base_on_dtor(mx: &mut MxBase) {
    let _allocated_shape = (mx.allocated_row_count, mx.allocated_col_count);
    let _active_shape = (mx.row_count, mx.col_count);
    let _data_rows = mx.data.len();
    let _name_len = mx.name.len();
}

/// Allocates (or grows) the underlying matrix storage with slack padding, updating global counters.
#[track_caller]
pub fn mx_base_alloc(mx: &mut MxBase, row_count: uint, col_count: uint, name: &str) {
    mx.name = name.to_string();
    let mut counts = MX_BASE_COUNTS.lock().unwrap();
    counts.alloc_count += 1;
    if mx.allocated_row_count == 0 {
        counts.zero_alloc_count += 1;
    }

    if row_count > mx.allocated_row_count || col_count > mx.allocated_col_count {
        if mx.allocated_row_count > 0 {
            counts.grow_alloc_count += 1;
        }
        let old_bytes = mx_base_total_bytes(mx.allocated_row_count, mx.allocated_col_count);
        counts.total_bytes -= old_bytes;
        let n = std::cmp::max(row_count + 16, mx.allocated_row_count);
        let m = std::cmp::max(col_count + 16, mx.allocated_col_count);
        mx.data = vec![vec![0.0; m as usize]; n as usize];
        mx.allocated_row_count = n;
        mx.allocated_col_count = m;
        let new_bytes = mx_base_total_bytes(n, m);
        counts.total_bytes += new_bytes;
        if counts.total_bytes > counts.max_bytes {
            counts.max_bytes = counts.total_bytes;
        }
    }

    mx.name = name.to_string();
    assert!(row_count <= mx.allocated_row_count);
    assert!(col_count <= mx.allocated_col_count);
    mx.row_count = row_count;
    mx.col_count = col_count;
}

/// Pretty-prints a matrix with optional log/exp scaling for human-readable diagnostics.
#[track_caller]
pub fn mx_base_log_me(mx: &MxBase, with_data: bool, opts: i32) -> String {
    let get_as_str = |i: uint, j: uint| -> String {
        let format_g4 = |d: f64| -> String {
            if d == 0.0 {
                return "0".to_string();
            }
            if !d.is_finite() {
                return d.to_string();
            }
            let exp = d.abs().log10().floor() as i32;
            let mut s = if exp < -4 || exp >= 4 {
                let raw = format!("{d:.3e}");
                let (mantissa, exponent) = raw.split_once('e').unwrap();
                let mut mantissa = mantissa
                    .trim_end_matches('0')
                    .trim_end_matches('.')
                    .to_string();
                if mantissa == "-0" {
                    mantissa = "0".to_string();
                }
                let exp_value = exponent.parse::<i32>().unwrap();
                let sign = if exp_value >= 0 { '+' } else { '-' };
                format!("{mantissa}e{sign}{:02}", exp_value.abs())
            } else {
                let decimals = (3 - exp).max(0) as usize;
                format!("{d:.decimals$}")
            };
            if !s.contains('e') && !s.contains('E') {
                s = s.trim_end_matches('0').trim_end_matches('.').to_string();
            }
            if s == "-0" {
                s = "0".to_string();
            }
            s
        };
        let f = mx.data[i as usize][j as usize];
        if f == UNINIT {
            format!("{:>12.12}", "?")
        } else if f < MINUS_INFINITY / 2.0 {
            format!("{:>12.12}", "*")
        } else if f == 0.0 {
            format!("{:>12.12}", ".")
        } else if (-1e5_f32..=1e5_f32).contains(&f) {
            format!("{:12.5}", f)
        } else {
            format!("{:>12}", format_g4(f64::from(f)))
        }
    };
    let mut out = String::new();
    out.push('\n');
    if opts & OPT_EXP != 0 {
        out.push_str("Exp ");
    } else if opts & OPT_LOG != 0 {
        out.push_str("Log ");
    }
    out.push_str(&format!(
        "{}({:p}) Rows {}/{}, Cols {}/{}\n",
        mx.name,
        mx as *const MxBase,
        mx.row_count,
        mx.allocated_row_count,
        mx.col_count,
        mx.allocated_col_count
    ));
    if !with_data || mx.row_count == 0 || mx.col_count == 0 {
        log(&out);
        return out;
    }

    let width = get_as_str(0, 0).len();
    let mut modulus = 1_u32;
    for _ in 0..width {
        modulus = modulus.wrapping_mul(10);
    }
    if modulus == 0 {
        modulus = 1;
    }
    out.push_str(&format!("{:5.5}", ""));
    for j in 0..mx.col_count {
        out.push_str(&format!("{:>width$}", j % modulus));
    }
    out.push('\n');
    for i in 0..mx.row_count {
        out.push_str(&format!("{:4} ", i));
        for j in 0..mx.col_count {
            let mut s = get_as_str(i, j);
            if opts & OPT_LOG != 0 {
                s = logize_str(s.trim());
            } else if opts & OPT_EXP != 0 {
                s = expize_str(s.trim());
            }
            out.push_str(&s);
        }
        out.push('\n');
    }
    log(&out);
    out
}

/// Returns a formatted summary of the global `MxBase` allocation counters.
#[track_caller]
pub fn mx_base_log_counts() -> String {
    let counts = MX_BASE_COUNTS.lock().unwrap();
    let mut out = String::new();
    out.push('\n');
    out.push_str("MxBase::LogCounts()\n");
    out.push_str("      What           N\n");
    out.push_str("----------  ----------\n");
    out.push_str(&format!("    Allocs  {:10}\n", counts.alloc_count));
    out.push_str(&format!("ZeroAllocs  {:10}\n", counts.zero_alloc_count));
    out.push_str(&format!("     Grows  {:10}\n", counts.grow_alloc_count));
    out.push_str(&format!(
        "     Bytes  {:>10.10}\n",
        mem_bytes_to_str(counts.total_bytes)
    ));
    out.push_str(&format!(
        " Max bytes  {:>10.10}\n",
        mem_bytes_to_str(counts.max_bytes)
    ));
    log(&out);
    out
}
