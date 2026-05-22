// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Expands `path_xy` to produce aligned row strings (`-` for gaps) from raw sequences `x`,`y`.
#[track_caller]
pub fn make_aln_rows_l4(x: &str, y: &str, path_xy: &str) -> (String, String) {
    let x_seq = x.as_bytes();
    let y_seq = y.as_bytes();
    let lx = x_seq.len();
    let ly = y_seq.len();
    let mut row_x = String::new();
    let mut row_y = String::new();
    let mut x_pos = 0usize;
    let mut y_pos = 0usize;
    for c in path_xy.bytes() {
        if c == b'B' || c == b'M' {
            row_x.push(x_seq[x_pos] as char);
            row_y.push(y_seq[y_pos] as char);
            y_pos += 1;
            x_pos += 1;
        } else if c == b'X' || c == b'D' {
            row_x.push(x_seq[x_pos] as char);
            row_y.push('-');
            x_pos += 1;
        } else if c == b'Y' || c == b'I' {
            row_y.push(y_seq[y_pos] as char);
            row_x.push('-');
            y_pos += 1;
        } else {
            panic!("invalid path char");
        }
    }
    assert!(x_pos == lx && y_pos == ly);
    (row_x, row_y)
}

/// Byte-slice overload of `make_aln_rows` taking a `PathInfo` for the path string.
#[track_caller]
pub fn make_aln_rows_l45(
    x_seq: &[byte],
    lx: uint,
    y_seq: &[byte],
    ly: uint,
    pi: &PathInfo,
) -> (String, String) {
    assert!(x_seq.len() >= lx as usize);
    assert!(y_seq.len() >= ly as usize);
    let path_xy = pi.path.clone();
    let mut row_x = String::new();
    let mut row_y = String::new();
    let mut x_pos = 0usize;
    let mut y_pos = 0usize;
    for c in path_xy.bytes() {
        if c == b'B' || c == b'M' {
            row_x.push(x_seq[x_pos] as char);
            row_y.push(y_seq[y_pos] as char);
            y_pos += 1;
            x_pos += 1;
        } else if c == b'X' || c == b'D' {
            row_x.push(x_seq[x_pos] as char);
            row_y.push('-');
            x_pos += 1;
        } else if c == b'Y' || c == b'I' {
            row_y.push(y_seq[y_pos] as char);
            row_x.push('-');
            y_pos += 1;
        } else {
            panic!("invalid path char");
        }
    }
    assert!(x_pos == lx as usize && y_pos == ly as usize);
    (row_x, row_y)
}

/// `Sequence` overload of `make_aln_rows` using only the `B`/`X`/`Y` path alphabet.
#[track_caller]
pub fn make_aln_rows_l85(x: &Sequence, y: &Sequence, path_xy: &str) -> (String, String) {
    let x_seq = sequence_get_seq_as_string(x).into_bytes();
    let y_seq = sequence_get_seq_as_string(y).into_bytes();
    let lx = x_seq.len();
    let ly = y_seq.len();
    let mut row_x = String::new();
    let mut row_y = String::new();
    let mut x_pos = 0usize;
    let mut y_pos = 0usize;
    for c in path_xy.bytes() {
        if c == b'B' {
            row_x.push(x_seq[x_pos] as char);
            row_y.push(y_seq[y_pos] as char);
            y_pos += 1;
            x_pos += 1;
        } else if c == b'X' {
            row_x.push(x_seq[x_pos] as char);
            row_y.push('-');
            x_pos += 1;
        } else if c == b'Y' {
            row_y.push(y_seq[y_pos] as char);
            row_x.push('-');
            y_pos += 1;
        } else {
            panic!("invalid path char");
        }
    }
    assert!(x_pos == lx && y_pos == ly);
    (row_x, row_y)
}

/// Logs two strings as an aligned pair following `path_xy` and returns the log text.
#[track_caller]
pub fn log_aln_l126(x: &str, y: &str, path_xy: &str) -> String {
    let (row_x, row_y) = make_aln_rows_l4(x, y, path_xy);
    let out = format!("\n{row_x}\n{row_y}\n");
    log(&out);
    out
}

/// Byte-slice overload of `log_aln` accepting a `PathInfo`.
#[track_caller]
pub fn log_aln_l136(x: &[byte], lx: uint, y: &[byte], ly: uint, pi: &PathInfo) -> String {
    assert!(x.len() >= lx as usize);
    assert!(y.len() >= ly as usize);
    let sx = String::from_utf8_lossy(&x[..lx as usize]).to_string();
    let sy = String::from_utf8_lossy(&y[..ly as usize]).to_string();
    log_aln_l126(&sx, &sy, &pi.path)
}

/// `Sequence` overload of `log_aln` that prefixes each row with its label.
#[track_caller]
pub fn log_aln_l149(x: &Sequence, y: &Sequence, path_xy: &str) -> String {
    let (row_x, row_y) = make_aln_rows_l85(x, y, path_xy);
    let out = format!("\n{:10.10}  {row_x}\n{:10.10}  {row_y}\n", x.label, y.label);
    log(&out);
    out
}

/// Builds position-to-column and column-to-position lookup vectors from a path string.
#[track_caller]
pub fn path_to_col_vecs(path_xy: &str) -> (Vec<uint>, Vec<uint>, Vec<uint>, Vec<uint>) {
    let mut pos_to_col_x = Vec::new();
    let mut pos_to_col_y = Vec::new();
    let mut col_to_pos_x = Vec::new();
    let mut col_to_pos_y = Vec::new();

    let col_count = path_xy.len();
    for (col, c) in path_xy.chars().enumerate() {
        let col = col as uint;
        if c == 'B' {
            let pos_x = pos_to_col_x.len() as uint;
            let pos_y = pos_to_col_y.len() as uint;
            col_to_pos_x.push(pos_x);
            col_to_pos_y.push(pos_y);
            pos_to_col_x.push(col);
            pos_to_col_y.push(col);
        } else if c == 'X' {
            let pos_x = pos_to_col_x.len() as uint;
            col_to_pos_x.push(pos_x);
            col_to_pos_y.push(uint::MAX);
            pos_to_col_x.push(col);
        } else if c == 'Y' {
            let pos_y = pos_to_col_y.len() as uint;
            col_to_pos_y.push(pos_y);
            col_to_pos_x.push(uint::MAX);
            pos_to_col_y.push(col);
        } else {
            panic!("invalid path char");
        }
    }

    assert_eq!(col_to_pos_x.len(), col_count);
    assert_eq!(col_to_pos_y.len(), col_count);
    for col in 0..col_count {
        let pos_x = col_to_pos_x[col];
        let pos_y = col_to_pos_y[col];
        if pos_x != uint::MAX {
            assert!((pos_x as usize) < pos_to_col_x.len());
            assert_eq!(pos_to_col_x[pos_x as usize], col as uint);
        }
        if pos_y != uint::MAX {
            assert!((pos_y as usize) < pos_to_col_y.len());
            assert_eq!(pos_to_col_y[pos_y as usize], col as uint);
        }
    }

    (pos_to_col_x, pos_to_col_y, col_to_pos_x, col_to_pos_y)
}

/// Renders the `|`/space annotation row marking matched positions in a pretty-aligned block.
#[track_caller]
pub fn write_annot_row(
    a: &[byte],
    b: &[byte],
    path: &str,
    mut i: uint,
    mut j: uint,
    col_lo: uint,
    col_hi: uint,
) -> String {
    let mut out = format!("{:5.5} ", "");
    let path_bytes = path.as_bytes();
    for k in col_lo..=col_hi {
        let c = path_bytes[k as usize] as char;
        if c == 'M' {
            let aa = a[i as usize];
            let bb = b[j as usize];
            i += 1;
            j += 1;
            if aa.to_ascii_uppercase() == bb.to_ascii_uppercase() {
                out.push('|');
            } else {
                out.push(' ');
            }
        } else {
            if c == 'D' {
                i += 1;
            } else if c == 'I' {
                j += 1;
            } else {
                panic!("invalid path char");
            }
            out.push(' ');
        }
    }
    out.push('\n');
    out
}

/// Renders the `B`-side row of a pretty-printed alignment block and returns the new column index.
#[track_caller]
pub fn write_b_row(
    b: &[byte],
    path: &str,
    mut j: uint,
    col_lo: uint,
    col_hi: uint,
    label_b: &str,
) -> (String, uint) {
    let mut out = format!("{:5} ", j + 1);
    let path_bytes = path.as_bytes();
    for k in col_lo..=col_hi {
        let c = path_bytes[k as usize] as char;
        if c == 'M' || c == 'I' {
            out.push(b[j as usize] as char);
            j += 1;
        } else {
            out.push('-');
        }
    }
    out.push_str(&format!(" {j}  {label_b}\n"));
    (out, j)
}

/// Renders the `A`-side row of a pretty-printed alignment block and returns the new column index.
#[track_caller]
pub fn write_a_row(
    a: &[byte],
    path: &str,
    mut i: uint,
    col_lo: uint,
    col_hi: uint,
    label_a: &str,
) -> (String, uint) {
    let mut out = format!("{:5} ", i + 1);
    let path_bytes = path.as_bytes();
    for k in col_lo..=col_hi {
        let c = path_bytes[k as usize] as char;
        if c == 'M' || c == 'D' {
            out.push(a[i as usize] as char);
            i += 1;
        } else {
            out.push('-');
        }
    }
    out.push_str(&format!(" {i}  {label_a}\n"));
    (out, i)
}

/// Pretty-prints a pairwise alignment in 80-column blocks with index annotations.
#[track_caller]
pub fn write_aln_pretty(a: &[byte], b: &[byte], path: &str) -> String {
    const BLOCK_SIZE: uint = 80;
    let col_lo = 0;
    let col_hi = path.len() as uint - 1;
    assert!(col_hi >= col_lo);

    let mut out = String::new();
    let mut i = 0;
    let mut j = 0;
    let mut col_from = col_lo;
    loop {
        if col_from > col_hi {
            break;
        }
        let mut col_to = col_from + BLOCK_SIZE - 1;
        if col_to > col_hi {
            col_to = col_hi;
        }

        let i0 = i;
        let j0 = j;
        let (a_row, new_i) = write_a_row(a, path, i, col_from, col_to, "");
        i = new_i;
        out.push_str(&a_row);
        out.push_str(&write_annot_row(a, b, path, i0, j0, col_from, col_to));
        let (b_row, new_j) = write_b_row(b, path, j, col_from, col_to, "");
        j = new_j;
        out.push_str(&b_row);
        out.push('\n');

        col_from = col_to + 1;
    }
    out
}
