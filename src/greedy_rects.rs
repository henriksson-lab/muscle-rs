// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns true if `(r, c)` falls inside the `r_count` x `c_count` grid.
pub fn in_bounds(r: i32, c: i32, r_count: i32, c_count: i32) -> bool {
    r >= 0 && r < r_count && c >= 0 && c < c_count
}

/// Returns maximal (greedy) contiguous, non-overlapping all-true rectangles
/// with width >= `min_width` and height >= `min_height`.
pub fn greedy_rects(mat: &[Vec<bool>], min_width: i32, min_height: i32) -> Vec<Rect> {
    let mut rects = Vec::new();
    if mat.is_empty() {
        return rects;
    }

    let row_count = mat.len();
    let col_count = mat[0].len();
    if col_count == 0 {
        return rects;
    }

    for row in mat {
        if row.len() != col_count {
            return rects;
        }
    }

    let mut consumed = vec![vec![false; col_count]; row_count];
    for r in 0..row_count {
        for c in 0..col_count {
            if consumed[r][c] || !mat[r][c] {
                continue;
            }

            let mut width = 0usize;
            while c + width < col_count && mat[r][c + width] && !consumed[r][c + width] {
                width += 1;
            }

            let mut height = 1usize;
            'height_loop: while r + height < row_count {
                for cc in c..c + width {
                    if !mat[r + height][cc] || consumed[r + height][cc] {
                        break 'height_loop;
                    }
                }
                height += 1;
            }

            for rr in r..r + height {
                for cc in c..c + width {
                    consumed[rr][cc] = true;
                }
            }

            if width as i32 >= min_width && height as i32 >= min_height {
                rects.push(Rect {
                    top: r as i32,
                    left: c as i32,
                    width: width as i32,
                    height: height as i32,
                });
            }
        }
    }

    rects
}
