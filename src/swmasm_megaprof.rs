// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Fast Smith-Waterman between a MASM `ma` and a mega-profile `pb`
/// with affine gaps; returns score, endpoints and traceback string.
#[track_caller]
pub fn sw_fast_masm_mega_prof(
    mem: &mut XDPMem,
    ma: &MASM,
    pb: &[Vec<byte>],
    open: f32,
    ext: f32,
) -> (f32, uint, uint, uint, uint, String) {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;
    const TRACEBITS_SM: byte = 0x10;
    const TRACEBITS_UNINIT: byte = !0x1f;

    let la = ma.col_count;
    let lb = pb.len() as uint;
    assert!(open <= 0.0);
    assert!(ext <= 0.0);
    assert!(ma.cols.len() >= la as usize);

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
        let col_a = &ma.cols[i as usize];
        let mut i0 = MINUS_INFINITY;
        for j in 0..lb {
            let col_b = &pb[j as usize];
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
            xm += masm_col_get_match_score_mega_profile_pos(col_a, col_b);
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
