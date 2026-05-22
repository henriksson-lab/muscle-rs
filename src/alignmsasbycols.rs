// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Aligns MSAs `x` and `y` by forcing matched anchor columns (`cols_x[i]`/`cols_y[i]`) to align.
/// Produces an `m/M/x/y` path and the gap-padded MSAs `x2`/`y2`; `merge_map[i]` is the column in
/// the merged alignment where the i'th anchor lands.
#[track_caller]
pub fn align_ms_as_by_cols(
    x: &MultiSequence,
    y: &MultiSequence,
    cols_x: &[uint],
    cols_y: &[uint],
    path: &mut String,
    merge_map: &mut Vec<uint>,
    x2: &mut MultiSequence,
    y2: &mut MultiSequence,
) {
    multi_sequence_clear(x2);
    multi_sequence_clear(y2);
    merge_map.clear();
    let nc = cols_x.len();
    assert_eq!(cols_y.len(), nc);
    assert!(nc > 0);
    for i in 1..nc {
        assert!(cols_x[i] > cols_x[i - 1]);
        assert!(cols_y[i] > cols_y[i - 1]);
    }

    let col_count_x = multi_sequence_get_col_count(x);
    let col_count_y = multi_sequence_get_col_count(y);
    let mut col_x = 0_u32;
    let mut col_y = 0_u32;
    let first_col_x = cols_x[0];
    let first_col_y = cols_y[0];
    if first_col_x < first_col_y {
        for _ in 0..first_col_y - first_col_x {
            path.push('y');
            col_y += 1;
        }
    } else if first_col_y < first_col_x {
        for _ in 0..first_col_x - first_col_y {
            path.push('x');
            col_x += 1;
        }
    }
    while col_x < first_col_x {
        path.push('m');
        col_x += 1;
        col_y += 1;
    }
    assert_eq!(col_x, first_col_x);
    assert_eq!(col_y, first_col_y);
    merge_map.push(path.len() as uint);
    path.push('M');

    for i in 1..nc {
        let next_col_x = cols_x[i];
        let next_col_y = cols_y[i];
        assert!(col_x < next_col_x);
        assert!(col_y < next_col_y);
        let nx = next_col_x - col_x;
        let ny = next_col_y - col_y;
        if nx < ny {
            for _ in 0..ny - nx {
                path.push('y');
                col_y += 1;
            }
        } else if ny < nx {
            for _ in 0..nx - ny {
                path.push('x');
                col_x += 1;
            }
        }
        while col_x + 1 < next_col_x {
            path.push('m');
            col_x += 1;
            col_y += 1;
        }
        assert_eq!(col_x + 1, next_col_x);
        assert_eq!(col_y + 1, next_col_y);
        merge_map.push(path.len() as uint);
        path.push('M');
        col_x = next_col_x;
        col_y = next_col_y;
    }

    col_x += 1;
    col_y += 1;
    while col_x < col_count_x && col_y < col_count_y {
        path.push('m');
        col_x += 1;
        col_y += 1;
    }
    while col_x < col_count_x {
        path.push('x');
        col_x += 1;
    }
    while col_y < col_count_y {
        path.push('y');
        col_y += 1;
    }

    let mut nx = 0_u32;
    let mut ny = 0_u32;
    for c in path.chars() {
        match c {
            'm' | 'M' => {
                nx += 1;
                ny += 1;
            }
            'x' => nx += 1,
            'y' => ny += 1,
            _ => panic!("Invalid AlignMSAsByCols path char '{c}'"),
        }
    }
    assert_eq!(nx, col_count_x);
    assert_eq!(ny, col_count_y);

    let pl = path.len();
    for seq in &x.seqs {
        let mut col = 0usize;
        let mut out_seq = Sequence {
            label: seq.label.clone(),
            char_vec: Vec::with_capacity(pl),
        };
        for c in path.chars() {
            match c {
                'm' | 'M' | 'x' => {
                    out_seq.char_vec.push(seq.char_vec[col]);
                    col += 1;
                }
                'y' => out_seq.char_vec.push('.'),
                _ => panic!("Invalid AlignMSAsByCols path char '{c}'"),
            }
        }
        x2.seqs.push(out_seq);
        x2.owners.push(true);
    }

    for seq in &y.seqs {
        let mut col = 0usize;
        let mut out_seq = Sequence {
            label: seq.label.clone(),
            char_vec: Vec::with_capacity(pl),
        };
        for c in path.chars() {
            match c {
                'm' | 'M' | 'y' => {
                    out_seq.char_vec.push(seq.char_vec[col]);
                    col += 1;
                }
                'x' => out_seq.char_vec.push('.'),
                _ => panic!("Invalid AlignMSAsByCols path char '{c}'"),
            }
        }
        y2.seqs.push(out_seq);
        y2.owners.push(true);
    }
}
