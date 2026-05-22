// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Converts an HSV color triplet into 8-bit RGB channels.
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (byte, byte, byte) {
    let h_i = (h * 6.0) as uint;
    let f = h * 6.0 - f64::from(h_i);
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = if h_i == 0 {
        (v, t, p)
    } else if h_i == 1 {
        (q, v, p)
    } else if h_i == 2 {
        (p, v, t)
    } else if h_i == 3 {
        (p, q, v)
    } else if h_i == 4 {
        (t, p, v)
    } else if h_i == 5 {
        (v, p, q)
    } else {
        panic!("invalid hue sector");
    };

    let r = (r * 256.0) as uint;
    let g = (g * 256.0) as uint;
    let b = (b * 256.0) as uint;
    assert!(r < 256);
    assert!(g < 256);
    assert!(b < 256);
    (r as byte, g as byte, b as byte)
}

/// Returns a visually distinct random color, advancing the hue by the golden
/// ratio fraction so successive calls are well-spread.
pub fn get_random_color() -> uint {
    let mut state = CMP_MSA_COLOR_STATE.lock().unwrap();
    if state.h == f64::MAX {
        state.h = f64::from(randu32() % 1000) / 1000.0;
    }

    state.h += 0.618_033_988_749_895;
    state.h %= 1.0;
    let (r, g, b) = hsv_to_rgb(state.h, 0.5, 0.95);
    let color = (uint::from(r) << 16) | (uint::from(g) << 8) | uint::from(b);
    assert!(color <= 0x00ff_ffff);
    color
}

/// Scales each RGB channel of `color` by `factor` (in `[0, 1]`) to produce a
/// darker variant.
pub fn darken(color: uint, factor: f64) -> uint {
    assert!((0.0..=1.0).contains(&factor));
    let mut r = (color >> 16) as byte;
    let mut g = ((color >> 8) % 0xff) as byte;
    let mut b = (color % 0xff) as byte;

    r = (f64::from(r) * factor) as byte;
    g = (f64::from(g) * factor) as byte;
    b = (f64::from(b) * factor) as byte;
    (uint::from(r) << 16) | (uint::from(g) << 8) | uint::from(b)
}

/// Returns a stable `#rrggbb` color string for reference column `ref_col`,
/// generating a new random base color every four columns and darkening within.
pub fn get_color(ref_col: uint) -> String {
    let mut state = CMP_MSA_COLOR_STATE.lock().unwrap();
    if ref_col as usize >= state.colors.len() {
        let n = state.colors.len();
        let new_len = ref_col as usize + 100;
        state.colors.resize(new_len, 0);
        for i in n..new_len {
            if i % 4 == 0 {
                if state.h == f64::MAX {
                    state.h = f64::from(randu32() % 1000) / 1000.0;
                }
                state.h += 0.618_033_988_749_895;
                state.h %= 1.0;
                let (r, g, b) = hsv_to_rgb(state.h, 0.5, 0.95);
                let color = (uint::from(r) << 16) | (uint::from(g) << 8) | uint::from(b);
                assert!(color <= 0x00ff_ffff);
                state.prev_random_color = color;
                state.colors[i] = state.prev_random_color;
            }
            let k = i % 4;
            assert!(k < 4);
            state.colors[i] = darken(state.prev_random_color, (4 - k) as f64 / 4.0);
        }
    }
    let hex_color = state.colors[ref_col as usize];
    format!("#{hex_color:06x}")
}

/// Compares a test MSA against a reference MSA, producing a Q-scorer result and
/// optionally writing a color-coded HTML visualization of the column matches.
#[track_caller]
pub fn cmd_cmp_msa(test_file_name: &str, ref_file_name: &str, output_file_name: &str) -> QScorer {
    let name = get_base_name(ref_file_name);

    let mut test = MultiSequence::default();
    let mut ref_msa = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut test, test_file_name, false);
    multi_sequence_load_mfa_l8(&mut ref_msa, ref_file_name, false);

    let mut qs = QScorer::default();
    q_scorer_run_l337(&mut qs, &name, &test, &ref_msa);

    let test_seq_count = test.seqs.len() as uint;
    let test_col_count = multi_sequence_get_col_count(&test);
    let mut test_seq_index_to_ref_cols = vec![Vec::<uint>::new(); test_seq_count as usize];

    let mut out = String::new();
    out.push_str("<html>\n");
    out.push_str("<body>\n");
    out.push_str("<span style=\"font-size:16px\">");
    out.push_str("<pre>");

    let label_count = qs.labels.len();
    assert_eq!(qs.test_seq_indexes.len(), label_count);
    assert_eq!(qs.ref_seq_indexes.len(), label_count);

    for label_index in 0..label_count {
        let test_seq_index = qs.test_seq_indexes[label_index];
        let ref_seq_index = qs.ref_seq_indexes[label_index];
        if test_seq_index == uint::MAX || ref_seq_index == uint::MAX {
            continue;
        }
        let pos_to_ref_col = &qs.pos_to_ref_col_vec[label_index];

        let ref_cols = &mut test_seq_index_to_ref_cols[test_seq_index as usize];
        ref_cols.resize(test_col_count as usize, uint::MAX);
        let label = &test.seqs[test_seq_index as usize].label;
        let mut pos = 0usize;
        for test_col in 0..test_col_count {
            let tc = msa_get_char(&test, test_seq_index, test_col);
            if !msa_is_gap(&test, test_seq_index, test_col) {
                let ref_col = pos_to_ref_col[pos];
                let rc = msa_get_char(&ref_msa, ref_seq_index, ref_col);
                let ok =
                    rc.is_ascii_alphabetic() && rc.to_ascii_uppercase() == tc.to_ascii_uppercase();
                if !ok {
                    die(&format!(
                        "!Ok\nTest label >{label}\n Ref label >{}\n     Pos {}\nTest col {}\n Ref col {}\nTest seq {}\n Ref seq {}\ntc '{}'\nrc '{}'",
                        ref_msa.seqs[ref_seq_index as usize].label,
                        pos,
                        test_col,
                        ref_col,
                        test_seq_index,
                        ref_seq_index,
                        tc,
                        rc
                    ));
                }
                assert_ne!(ref_col, uint::MAX);
                assert_eq!(ref_cols[test_col as usize], uint::MAX);
                if rc.is_ascii_uppercase() {
                    ref_cols[test_col as usize] = ref_col;
                }
                pos += 1;
            }
        }
    }

    const ROWLEN: uint = 100;
    let block_count = (test_col_count + ROWLEN - 1) / ROWLEN;
    for block_index in 0..block_count {
        let from = block_index * ROWLEN;
        let mut to = from + ROWLEN;
        if to >= test_col_count {
            to = test_col_count;
        }
        for test_seq_index in 0..test_seq_count {
            if test_seq_index_to_ref_cols[test_seq_index as usize].is_empty() {
                continue;
            }
            out.push_str("   ");
            let label = &test.seqs[test_seq_index as usize].label;
            for test_col in from..to {
                let tc = msa_get_char(&test, test_seq_index, test_col);
                let ref_col =
                    test_seq_index_to_ref_cols[test_seq_index as usize][test_col as usize];

                if ref_col == uint::MAX {
                    out.push_str(&format!("<span style=\"color:gray\">{tc}</span>"));
                } else {
                    let color = if tc.is_ascii_lowercase() {
                        "gray".to_string()
                    } else {
                        get_color(ref_col)
                    };
                    out.push_str(&format!(
                        "<span style=\"color:white;background-color:{color}\">{tc}</span>"
                    ));
                }
            }
            for _ in to..ROWLEN {
                out.push(' ');
            }
            out.push_str(&format!(
                "  <span style=\"color:black\">{label}   </span>\n"
            ));
        }
        out.push_str("\n\n");
    }
    out.push_str("</pre>");
    out.push_str("</span>");
    out.push_str("</body>");
    out.push_str("</html>\n");

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write cmp_msa HTML");
    }
    qs
}
