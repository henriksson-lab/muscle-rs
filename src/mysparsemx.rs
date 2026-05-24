// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const MIN_SPARSE_PROB: f32 = 0.01_f32;

pub const MIN_SPARSE_SCORE: f32 = -4.6051702_f32;

pub const POSTERIOR_CUTOFF: f32 = 0.01_f32;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MySparseMx {
    pub lx: uint,
    pub ly: uint,
    pub vec_size: uint,
    pub max_vec_size: uint,
    pub value_vec: Vec<(f32, uint)>,
    pub max_lx: uint,
    pub offsets: Vec<uint>,
    pub x: Option<Vec<byte>>,
    pub y: Option<Vec<byte>>,
} // original: MySparseMx (muscle/src/mysparsemx.h)

/// Grows the value vector capacity to at least `size`, with 256-entry slack.
#[track_caller]
pub fn my_sparse_mx_alloc_vec(mx: &mut MySparseMx, size: uint) {
    if size <= mx.max_vec_size {
        return;
    }
    mx.max_vec_size = size + 256;
    mx.value_vec.resize(mx.max_vec_size as usize, (0.0, 0));
}

/// Returns the maximum stored probability across row `i`.
#[track_caller]
pub fn my_sparse_mx_get_max_prob_row(mx: &MySparseMx, i: uint) -> f32 {
    let offset = my_sparse_mx_get_offset(mx, i);
    let size = my_sparse_mx_get_size(mx, i);
    let mut max = 0.0_f32;
    for k in 0..size {
        let p = mx.value_vec[(offset + k) as usize].0;
        max = max.max(p);
    }
    max
}

/// Returns the index into the value vector where row `i` begins.
#[track_caller]
pub fn my_sparse_mx_get_offset(mx: &MySparseMx, i: uint) -> uint {
    assert!(i < mx.lx);
    mx.offsets[i as usize]
}

/// Returns the number of stored entries in row `i`.
#[track_caller]
pub fn my_sparse_mx_get_size(mx: &MySparseMx, i: uint) -> uint {
    assert!(i < mx.lx);
    let offset = my_sparse_mx_get_offset(mx, i);
    mx.offsets[i as usize + 1] - offset
}

/// Looks up `P(i, j)` in the sparse matrix; returns 0.0 if the entry is below the threshold.
#[track_caller]
pub fn my_sparse_mx_get_prob(mx: &MySparseMx, i: uint, j: uint) -> f32 {
    let mut offset = my_sparse_mx_get_offset(mx, i);
    let size = my_sparse_mx_get_size(mx, i);
    for _k in 0..size {
        let j2 = mx.value_vec[offset as usize].1;
        if j2 == j {
            return mx.value_vec[offset as usize].0;
        } else if j2 > j {
            return 0.0;
        }
        offset += 1;
    }
    0.0
}

/// Grows the row-offsets table to hold at least `lx` rows, with 128-row slack.
#[track_caller]
pub fn my_sparse_mx_alloc_lx(mx: &mut MySparseMx, lx: uint) {
    if lx <= mx.max_lx {
        return;
    }
    mx.max_lx = lx + 128;
    mx.offsets.resize(mx.max_lx as usize + 1, uint::MAX);
}

/// Reuses `old_mx`'s sparsity pattern but recomputes probabilities from `post`, normalized by `seq_count`.
#[inline(always)]
pub fn my_sparse_mx_update_from_post(
    mx: &mut MySparseMx,
    old_mx: &MySparseMx,
    post: &[f32],
    seq_count: uint,
) {
    let vec_size = old_mx.vec_size;
    let lx = old_mx.lx;
    let ly = old_mx.ly;
    my_sparse_mx_alloc_lx(mx, lx);
    my_sparse_mx_alloc_vec(mx, vec_size);
    mx.lx = lx;
    mx.ly = ly;
    for i in 0..lx {
        mx.offsets[i as usize] = old_mx.offsets[i as usize];
    }
    mx.offsets[lx as usize] = old_mx.offsets[lx as usize];
    mx.vec_size = vec_size;

    for i in 0..mx.lx as usize {
        let offset = mx.offsets[i] as usize;
        let size = mx.offsets[i + 1] as usize - offset;
        let row_base = i * ly as usize;
        for k in 0..size {
            let value_index = offset + k;
            let col = old_mx.value_vec[value_index].1;
            let p = post[row_base + col as usize] / seq_count as f32;
            mx.value_vec[value_index] = (p, col);
        }
    }
}

/// Builds a sparse matrix from the dense posterior `post`, keeping only entries above `MIN_SPARSE_PROB`.
#[track_caller]
pub fn my_sparse_mx_from_post(mx: &mut MySparseMx, post: &[f32], lx: uint, ly: uint) {
    mx.lx = lx;
    mx.ly = ly;

    my_sparse_mx_alloc_lx(mx, lx);
    let mut entries = Vec::<(f32, uint)>::new();
    let mut offset = 0_u32;
    for i in 0..lx {
        mx.offsets[i as usize] = offset;
        for j in 0..ly {
            let p = post[(i * ly + j) as usize];
            if p >= MIN_SPARSE_PROB {
                entries.push((p, j));
                offset += 1;
            }
        }
    }
    mx.offsets[lx as usize] = offset;

    mx.vec_size = offset;
    my_sparse_mx_alloc_vec(mx, mx.vec_size);
    mx.value_vec[..mx.vec_size as usize].copy_from_slice(&entries);
}

/// Returns a one-line summary of the sparse matrix dimensions and nnz count.
#[track_caller]
pub fn my_sparse_mx_log_stats(mx: &MySparseMx, msg: &str) -> String {
    format!(
        "MySparseMx({msg}) LX={}, LY={} VecSize={}\n",
        mx.lx, mx.ly, mx.vec_size
    )
}

/// Pretty-prints the sparse matrix as a dense table for diagnostics.
#[track_caller]
pub fn my_sparse_mx_log_me(mx: &MySparseMx) -> String {
    let mut out = String::new();
    let format_g3 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d64:.2e}");
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
            let decimals = (2 - exp).max(0) as usize;
            format!("{d64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    out.push('\n');
    out.push_str(&format!(
        "MySparseMx({:p}) LX={}, LY={}\n",
        mx, mx.lx, mx.ly
    ));
    out.push('\n');
    out.push_str("  Row   Size");
    if mx.x.is_some() {
        out.push_str("  x  ");
    }
    for j in 0..mx.ly {
        out.push_str(&format!("  {j:8}"));
    }
    out.push('\n');
    if let Some(y) = &mx.y {
        out.push('\n');
        out.push_str("                 ");
        for &c in y.iter().take(mx.ly as usize) {
            out.push_str(&format!("  {:8}", c as char));
        }
        out.push('\n');
    }

    for i in 0..mx.lx {
        let size = my_sparse_mx_get_size(mx, i);
        out.push_str(&format!("{i:5}"));
        out.push_str(&format!("  {size:5}"));
        if let Some(x) = &mx.x {
            out.push_str(&format!("  {}  ", x[i as usize] as char));
        }
        for j in 0..mx.ly {
            let p = my_sparse_mx_get_prob(mx, i, j);
            if p == 0.0 {
                out.push_str(&format!("  {:8.8}", "."));
            } else {
                out.push_str(&format!("  {:>8}", format_g3(p)));
            }
        }
        out.push('\n');
    }
    out
}

/// Materializes the sparse matrix as a dense `lx * ly` posterior vector with zeros for missing entries.
#[inline(always)]
pub fn my_sparse_mx_to_post(mx: &MySparseMx) -> Vec<f32> {
    let mut post = vec![0.0_f32; (mx.lx * mx.ly) as usize];
    let ly = mx.ly as usize;
    for row in 0..mx.lx as usize {
        let offset = mx.offsets[row] as usize;
        let size = mx.offsets[row + 1] as usize - offset;
        let row_base = row * ly;
        for k in 0..size {
            let (p, col) = mx.value_vec[offset + k];
            post[row_base + col as usize] = p;
        }
    }
    post
}

/// For each column, returns the lowest and highest row index that has a non-zero entry there.
#[track_caller]
pub fn my_sparse_mx_get_col_to_row_lo_hi(mx: &MySparseMx) -> (Vec<uint>, Vec<uint>) {
    let mut col_to_row_lo = vec![uint::MAX; mx.ly as usize];
    let mut col_to_row_hi = vec![uint::MAX; mx.ly as usize];

    for row in 0..mx.lx {
        let offset = my_sparse_mx_get_offset(mx, row);
        let size = my_sparse_mx_get_size(mx, row);
        for k in 0..size {
            let col = mx.value_vec[(offset + k) as usize].1;
            assert!(col < mx.ly);
            if col_to_row_lo[col as usize] == uint::MAX {
                col_to_row_lo[col as usize] = row;
                col_to_row_hi[col as usize] = row;
            } else {
                if row < col_to_row_lo[col as usize] {
                    col_to_row_lo[col as usize] = row;
                }
                if row > col_to_row_hi[col as usize] {
                    col_to_row_hi[col as usize] = row;
                }
            }
        }
    }
    (col_to_row_lo, col_to_row_hi)
}
