// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Viterbi alignment of `a` and `b` using bit-traceback in the supplied scratch buffer.
#[track_caller]
pub fn viterbi_fast_mem(
    mem: &mut XDPMem,
    a: &[byte],
    la: uint,
    b: &[byte],
    lb: uint,
    pi: &mut PathInfo,
) -> f32 {
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;

    if la * lb > 100 * 1000 * 1000 {
        die(&format!("ViterbiFastMem, seqs too long LA={la}, LB={lb}"));
    }
    assert!(a.len() >= la as usize);
    assert!(b.len() >= lb as usize);

    mem.max_la = la;
    mem.max_lb = lb;
    mem.buffer1 = vec![MINUS_INFINITY; lb as usize + 1];
    mem.buffer2 = vec![MINUS_INFINITY; lb as usize + 1];
    mem.tb_bit = vec![vec![0; lb as usize + 1]; la as usize + 1];
    mem.tb_bit_row_count = la + 1;
    mem.tb_bit_col_count = lb + 1;
    mem.tb_bit_allocated_row_count = la + 1;
    mem.tb_bit_allocated_col_count = lb + 1;
    path_info_alloc2(pi, la, lb);

    let mut open_a = -3.0_f32;
    let mut ext_a = -0.5_f32;
    let mut m0 = 0.0_f32;
    for i in 0..la {
        let aa = a[i as usize];
        let mut open_b = -3.0_f32;
        let mut ext_b = -0.5_f32;
        let mut i0 = MINUS_INFINITY;

        for j in 0..lb {
            let bb = b[j as usize];
            let mut trace_bits = 0;
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
            m0 = mem.buffer1[j as usize];
            mem.buffer1[j as usize] = xm + get_blosum_score_chars(aa, bb);

            let md = saved_m0 + open_b;
            mem.buffer2[j as usize] += ext_b;
            if md >= mem.buffer2[j as usize] {
                mem.buffer2[j as usize] = md;
                trace_bits |= TRACEBITS_MD;
            }

            let mi = saved_m0 + open_a;
            i0 += ext_a;
            if mi >= i0 {
                i0 = mi;
                trace_bits |= TRACEBITS_MI;
            }

            open_b = -3.0;
            ext_b = -0.5;
            mem.tb_bit[i as usize][j as usize] = trace_bits;
        }

        mem.tb_bit[i as usize][lb as usize] = 0;
        let md = m0 - 3.0;
        mem.buffer2[lb as usize] += -0.5;
        if md >= mem.buffer2[lb as usize] {
            mem.buffer2[lb as usize] = md;
            mem.tb_bit[i as usize][lb as usize] = TRACEBITS_MD;
        }

        m0 = MINUS_INFINITY;
        open_a = -3.0;
        ext_a = -0.5;
    }

    let mut i1 = MINUS_INFINITY;
    for j in 1..lb {
        mem.tb_bit[la as usize][j as usize] = 0;
        let mi = mem.buffer1[j as usize - 1] - 3.0;
        i1 += -0.5;
        if mi > i1 {
            i1 = mi;
            mem.tb_bit[la as usize][j as usize] = TRACEBITS_MI;
        }
    }

    let final_m = mem.buffer1[lb as usize - 1];
    let final_d = mem.buffer2[lb as usize];
    let final_i = i1;
    let mut score = final_m;
    let mut state = b'M';
    if final_d > score {
        score = final_d;
        state = b'D';
    }
    if final_i > score {
        score = final_i;
        state = b'I';
    }

    trace_back_bit_mem(mem, la, lb, state, pi);
    score
}
