// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Maximum-expected-accuracy DP over a dense posterior. Fills `tb` with the
/// traceback and returns `(score, path)` describing the optimal alignment.
#[track_caller]
pub fn calc_aln_flat(
    post: &[f32],
    lx: uint,
    ly: uint,
    dp_rows: &mut [f32],
    tb: &mut [i8],
) -> (f32, String) {
    let row_len = (ly + 1) as usize;
    assert!(dp_rows.len() >= row_len * 2);
    assert!(tb.len() >= ((lx + 1) * (ly + 1)) as usize);
    assert!(post.len() >= (lx * ly) as usize);

    for j in 0..=ly as usize {
        dp_rows[j] = 0.0;
        tb[j] = b'Y' as i8;
    }

    let mut old_row_offset = 0usize;
    let mut new_row_offset = row_len;
    let mut tb_ptr = row_len;
    let mut post_ix = 0usize;

    for i in 1..=lx as usize {
        tb[tb_ptr] = b'X' as i8;
        tb_ptr += 1;
        dp_rows[new_row_offset] = 0.0;

        for j in 1..=ly as usize {
            let b_score = dp_rows[old_row_offset + j - 1] + post[post_ix];
            post_ix += 1;
            let x_score = dp_rows[old_row_offset + j];
            let y_score = dp_rows[new_row_offset + j - 1];

            let (best, tb_char) = if b_score >= x_score && b_score >= y_score {
                (b_score, b'B' as i8)
            } else if x_score >= y_score {
                (x_score, b'X' as i8)
            } else {
                (y_score, b'Y' as i8)
            };
            dp_rows[new_row_offset + j] = best;
            tb[tb_ptr] = tb_char;
            tb_ptr += 1;
        }

        std::mem::swap(&mut old_row_offset, &mut new_row_offset);
    }

    let score = dp_rows[old_row_offset + ly as usize];
    let path = trace_back_flat(tb, lx, ly);
    (score, path)
}
