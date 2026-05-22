// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Format the M/D/I traceback matrices stored in `mem` for debug output.
#[track_caller]
pub fn log_tbsw(msg: &str, mem: &XDPMem, la: uint, lb: uint) -> String {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;
    const TRACEBITS_SM: byte = 0x10;
    const TRACEBITS_UNINIT: byte = !0x1f;

    let mut out = String::new();
    out.push_str(&format!("TBM {msg}\n"));
    for i in 0..la {
        out.push_str(&format!("{i:3} | "));
        for j in 0..lb {
            let bits = mem.tb_bit[i as usize][j as usize];
            let c = if bits == TRACEBITS_UNINIT {
                '*'
            } else if bits & TRACEBITS_DM != 0 {
                'D'
            } else if bits & TRACEBITS_IM != 0 {
                'I'
            } else if bits & TRACEBITS_SM != 0 {
                'S'
            } else {
                'M'
            };
            out.push(c);
        }
        out.push('\n');
    }

    out.push_str(&format!("\nTBD {msg}\n"));
    for i in 0..la {
        out.push_str(&format!("{i:3} | "));
        for j in 0..lb {
            let bits = mem.tb_bit[i as usize][j as usize];
            let c = if bits == TRACEBITS_UNINIT {
                '*'
            } else if bits & TRACEBITS_MD != 0 {
                'M'
            } else {
                'D'
            };
            out.push(c);
        }
        out.push('\n');
    }

    out.push_str("\nTBI\n");
    for i in 0..la {
        out.push_str(&format!("{i:3} | "));
        for j in 0..lb {
            let bits = mem.tb_bit[i as usize][j as usize];
            let c = if bits == TRACEBITS_UNINIT {
                '*'
            } else if bits & TRACEBITS_MI != 0 {
                'M'
            } else {
                'I'
            };
            out.push(c);
        }
        out.push('\n');
    }
    out
}

/// Walk the bit-packed traceback in `mem` from `(besti, bestj)` to the
/// local-alignment start and return the lengths and `MDI` path string.
#[track_caller]
pub fn trace_back_bit_sw(
    mem: &XDPMem,
    la: uint,
    lb: uint,
    besti: uint,
    bestj: uint,
) -> (uint, uint, String) {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;
    const TRACEBITS_SM: byte = 0x10;

    assert!(besti <= la);
    assert!(bestj <= lb);
    let mut path = String::new();
    let mut i = besti;
    let mut j = bestj;
    let mut state = b'M';
    loop {
        path.push(char::from(state));

        match state {
            b'M' => {
                assert!(i > 0 && j > 0);
                let t = mem.tb_bit[(i - 1) as usize][(j - 1) as usize];
                if t & TRACEBITS_DM != 0 {
                    state = b'D';
                } else if t & TRACEBITS_IM != 0 {
                    state = b'I';
                } else if t & TRACEBITS_SM != 0 {
                    let leni = besti - i + 1;
                    let lenj = bestj - j + 1;
                    let path = path.bytes().rev().map(char::from).collect();
                    return (leni, lenj, path);
                } else {
                    state = b'M';
                }
                i -= 1;
                j -= 1;
            }
            b'D' => {
                assert!(i > 0);
                let t = mem.tb_bit[(i - 1) as usize][j as usize];
                if t & TRACEBITS_MD != 0 {
                    state = b'M';
                } else {
                    state = b'D';
                }
                i -= 1;
            }
            b'I' => {
                assert!(j > 0);
                let t = mem.tb_bit[i as usize][(j - 1) as usize];
                if t & TRACEBITS_MI != 0 {
                    state = b'M';
                } else {
                    state = b'I';
                }
                j -= 1;
            }
            _ => panic!("TraceBackBitSW, invalid state {}", state as char),
        }
    }
}

/// Fast Smith-Waterman driven by a pre-built score matrix `smx` with
/// affine gap penalties; returns score, endpoints, lengths and path.
#[track_caller]
pub fn sw_fast_s_mx(
    mem: &mut XDPMem,
    smx: &Mx,
    open: f32,
    ext: f32,
) -> (f32, uint, uint, uint, uint, String) {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;
    const TRACEBITS_SM: byte = 0x10;
    const TRACEBITS_UNINIT: byte = !0x1f;

    let la = smx.row_count;
    let lb = smx.col_count;
    assert!(open <= 0.0);
    assert!(ext <= 0.0);
    assert!(smx.data.len() >= la as usize);
    if la > 0 {
        assert!(smx.data[0].len() >= lb as usize);
    }

    mem.max_la = la + 32;
    mem.max_lb = lb + 32;
    mem.buffer1 = vec![0.0; mem.max_lb as usize + 1];
    mem.buffer2 = vec![0.0; mem.max_lb as usize + 1];
    mem.tb_bit = vec![vec![TRACEBITS_UNINIT; mem.max_lb as usize]; mem.max_la as usize];
    mem.tb_bit_row_count = la;
    mem.tb_bit_col_count = lb;
    mem.tb_bit_allocated_row_count = mem.max_la;
    mem.tb_bit_allocated_col_count = mem.max_lb;

    let mut len_i = 0;
    let mut len_j = 0;
    for j in 0..=lb as usize {
        mem.buffer1[j] = MINUS_INFINITY;
        mem.buffer2[j] = MINUS_INFINITY;
    }

    let mut best_score = 0.0;
    let mut best_i = uint::MAX;
    let mut best_j = uint::MAX;
    let mut m0 = 0.0;
    for i in 0..la {
        let mut i0 = MINUS_INFINITY;
        for j in 0..lb {
            let mut trace_bits: byte = 0;
            let saved_m0 = m0;

            let mut xm = m0;
            if mem.buffer2[j as usize] > xm {
                xm = mem.buffer2[j as usize];
                trace_bits = TRACEBITS_DM;
            }
            if i0 > xm {
                xm = i0;
                trace_bits = TRACEBITS_IM;
            }
            if 0.0 >= xm {
                xm = 0.0;
                trace_bits = TRACEBITS_SM;
            }

            m0 = mem.buffer1[j as usize];
            xm += smx.data[i as usize][j as usize];
            if xm > best_score {
                best_score = xm;
                best_i = i;
                best_j = j;
            }
            mem.buffer1[j as usize] = xm;

            let md = saved_m0 + open;
            mem.buffer2[j as usize] += ext;
            if md >= mem.buffer2[j as usize] {
                mem.buffer2[j as usize] = md;
                trace_bits |= TRACEBITS_MD;
            }

            let mi = saved_m0 + open;
            i0 += ext;
            if mi >= i0 {
                i0 = mi;
                trace_bits |= TRACEBITS_MI;
            }

            mem.tb_bit[i as usize][j as usize] = trace_bits;
        }
        m0 = MINUS_INFINITY;
    }

    if best_score <= 0.0 {
        return (0.0, uint::MAX, uint::MAX, len_i, len_j, String::new());
    }

    let (trace_len_i, trace_len_j, path) = trace_back_bit_sw(mem, la, lb, best_i + 1, best_j + 1);
    len_i = trace_len_i;
    len_j = trace_len_j;
    assert!(best_i + 1 >= len_i);
    assert!(best_j + 1 >= len_j);
    let lo_i = best_i + 1 - len_i;
    let lo_j = best_j + 1 - len_j;
    (best_score, lo_i, lo_j, len_i, len_j, path)
}

/// Smith-Waterman of two raw amino-acid strings under BLOSUM62.
#[track_caller]
pub fn sw_fast_strings_blosum62(
    mem: &mut XDPMem,
    a: &str,
    b: &str,
    open: f32,
    ext: f32,
) -> (f32, uint, uint, uint, uint, String) {
    let smx = make_blosum62_s_mx_l54(a, b);
    sw_fast_s_mx(mem, &smx, open, ext)
}

/// Smith-Waterman of two `Sequence` objects under BLOSUM62.
#[track_caller]
pub fn sw_fast_seqs_blosum62(
    mem: &mut XDPMem,
    a: &Sequence,
    b: &Sequence,
    open: f32,
    ext: f32,
) -> (f32, uint, uint, uint, uint, String) {
    let smx = make_blosum62_s_mx_l30(a, b);
    sw_fast_s_mx(mem, &smx, open, ext)
}

/// Project the 20x20 log-odds matrix onto the residue pairs of `a` and `b`.
#[track_caller]
pub fn make_log_odds_s_mx(a: &Sequence, b: &Sequence, log_odds_mx: &[Vec<f32>]) -> Mx {
    let la = a.char_vec.len();
    let lb = b.char_vec.len();
    let mut mx_s = Mx {
        name: "LOS".to_string(),
        row_count: la as uint,
        col_count: lb as uint,
        data: vec![vec![0.0; lb]; la],
    };
    let state = ALPHA_STATE.lock().unwrap();
    for i in 0..la {
        let ai = state.char_to_letter[a.char_vec[i] as usize];
        for j in 0..lb {
            let bi = state.char_to_letter[b.char_vec[j] as usize];
            if ai < 20 && bi < 20 {
                mx_s.data[i][j] = log_odds_mx[ai as usize][bi as usize];
            } else {
                mx_s.data[i][j] = 0.0;
            }
        }
    }
    mx_s
}

/// Smith-Waterman of two sequences scored by a caller-supplied log-odds matrix.
#[track_caller]
pub fn sw_fast_seqs_lo(
    mem: &mut XDPMem,
    log_odds_mx: &[Vec<f32>],
    a: &Sequence,
    b: &Sequence,
    open: f32,
    ext: f32,
) -> (f32, uint, uint, uint, uint, String) {
    let smx = make_log_odds_s_mx(a, b, log_odds_mx);
    sw_fast_s_mx(mem, &smx, open, ext)
}

/// CLI entry point: run all-pairs Smith-Waterman BLOSUM62 over the input
/// MFA and emit per-pair scores plus local alignments.
#[track_caller]
pub fn cmd_sw(input_file_name: &str) -> String {
    let mut input = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input, input_file_name, true);

    let seq_count = input.seqs.len() as uint;
    if seq_count < 2 {
        die(&format!("{seq_count} seqs"));
    }

    let open = -5.0_f32;
    let ext = -1.0_f32;
    let pair_count = (seq_count * (seq_count - 1)) / 2;
    let mut pair_idx = 0;
    let mut mem = XDPMem::default();
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
    for i in 0..seq_count {
        let seq_i = &input.seqs[i as usize];
        let label_i = seq_i.label.clone();
        for j in (i + 1)..seq_count {
            assert!(pair_idx < pair_count);
            pair_idx += 1;

            let seq_j = &input.seqs[j as usize];
            let label_j = seq_j.label.clone();
            let (score, loi, loj, _leni, _lenj, path) =
                sw_fast_seqs_blosum62(&mut mem, seq_i, seq_j, open, ext);

            out.push('\n');
            if !path.is_empty() {
                let ai = seq_i.char_vec.iter().collect::<String>();
                let bj = seq_j.char_vec.iter().collect::<String>();
                out.push_str(&write_local_aln(
                    &label_i,
                    ai.as_bytes(),
                    &label_j,
                    bj.as_bytes(),
                    loi,
                    loj,
                    &path,
                ));
            }
            out.push_str(&format!(
                "{} {} {}\n",
                seq_i.label,
                seq_j.label,
                format_g3(score)
            ));
        }
    }
    out
}
