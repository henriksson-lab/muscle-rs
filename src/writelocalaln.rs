// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Format a local alignment of `a` and `b` as a multi-block, fixed-width text view.
#[track_caller]
pub fn write_local_aln(
    label_a: &str,
    a: &[byte],
    label_b: &str,
    b: &[byte],
    loi: uint,
    loj: uint,
    path: &str,
) -> String {
    const BLOCK_SIZE: uint = 80;
    let col_lo = 0;
    let col_hi = path.len() as uint - 1;
    assert!(col_hi >= col_lo);

    let mut out = String::new();
    let mut pos_a = loi;
    let mut pos_b = loj;
    let mut col_from = col_lo;
    loop {
        if col_from > col_hi {
            break;
        }
        let mut col_to = col_from + BLOCK_SIZE - 1;
        if col_to > col_hi {
            col_to = col_hi;
        }

        let i0 = pos_a;
        let j0 = pos_b;
        let (a_row, new_pos_a) = write_a_row(a, path, pos_a, col_from, col_to, label_a);
        pos_a = new_pos_a;
        out.push_str(&a_row);
        out.push_str(&write_annot_row(a, b, path, i0, j0, col_from, col_to));
        let (b_row, new_pos_b) = write_b_row(b, path, pos_b, col_from, col_to, label_b);
        pos_b = new_pos_b;
        out.push_str(&b_row);
        out.push('\n');

        col_from += BLOCK_SIZE;
    }
    out
}

// Extra stubs from non-UTF-8 source skipped by CCC UTF-8 reader.
