// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Pair-state Smith-Waterman with caller-supplied match and transition
/// scoring closures; writes the local alignment endpoints and path string.
#[track_caller]
pub fn swps<FMatch, FMM, FMD, FMI, FDM, FDD, FIM, FII>(
    mem: &mut XDPMem,
    la: uint,
    lb: uint,
    lo_i: &mut uint,
    lo_j: &mut uint,
    path: &mut String,
    get_match_score: FMatch,
    get_score_mm: FMM,
    get_score_md: FMD,
    get_score_mi: FMI,
    get_score_dm: FDM,
    get_score_dd: FDD,
    get_score_im: FIM,
    get_score_ii: FII,
) -> f32
where
    FMatch: Fn(uint, uint) -> f32,
    FMM: Fn(uint, uint) -> f32,
    FMD: Fn(uint, uint) -> f32,
    FMI: Fn(uint, uint) -> f32,
    FDM: Fn(uint, uint) -> f32,
    FDD: Fn(uint, uint) -> f32,
    FIM: Fn(uint, uint) -> f32,
    FII: Fn(uint, uint) -> f32,
{
    const TRACEBITS_DM: byte = 0x01;
    const TRACEBITS_IM: byte = 0x02;
    const TRACEBITS_MD: byte = 0x04;
    const TRACEBITS_MI: byte = 0x08;
    const TRACEBITS_SM: byte = 0x10;
    const TRACEBITS_UNINIT: byte = !0x1f;

    mem.max_la = la + 32;
    mem.max_lb = lb + 32;
    mem.buffer1 = vec![0.0; mem.max_lb as usize + 1];
    mem.buffer2 = vec![0.0; mem.max_lb as usize + 1];
    mem.tb_bit = vec![vec![TRACEBITS_UNINIT; mem.max_lb as usize]; mem.max_la as usize];
    mem.tb_bit_row_count = la;
    mem.tb_bit_col_count = lb;
    mem.tb_bit_allocated_row_count = mem.max_la;
    mem.tb_bit_allocated_col_count = mem.max_lb;

    for j in 0..=lb as usize {
        mem.buffer1[j] = MINUS_INFINITY;
        mem.buffer2[j] = MINUS_INFINITY;
    }

    let mut best_score = 0.0;
    let mut best_i = uint::MAX;
    let mut best_j = uint::MAX;
    for i in 0..la {
        let mut iscore = MINUS_INFINITY;
        let mut prev_saved_m = MINUS_INFINITY;
        for j in 0..lb {
            let mut trace_bits: byte = 0;
            let saved_m = mem.buffer1[j as usize + 1];

            let mm = prev_saved_m + get_score_mm(i, j);
            let dm = mem.buffer2[j as usize] + get_score_dm(i, j);
            let im = iscore + get_score_im(i, j);
            let mut t = mm;
            if dm > t {
                t = dm;
                trace_bits = TRACEBITS_DM;
            }
            if im > t {
                t = im;
                trace_bits = TRACEBITS_IM;
            }
            if t < 0.0 {
                t = 0.0;
                trace_bits = TRACEBITS_SM;
            }

            let m = t + get_match_score(i, j);
            if m > best_score {
                best_score = m;
                best_i = i;
                best_j = j;
            }
            mem.buffer1[j as usize + 1] = m;

            let md = prev_saved_m + get_score_md(i, j);
            let dd = mem.buffer2[j as usize] + get_score_dd(i, j);
            let mut d = md;
            if dd > d {
                d = md;
                trace_bits |= TRACEBITS_MD;
            }
            mem.buffer2[j as usize] = d;

            let mi = prev_saved_m + get_score_mi(i, j);
            let ii = iscore + get_score_ii(i, j);
            iscore = ii;
            if mi > iscore {
                iscore = mi;
                trace_bits |= TRACEBITS_MI;
            }

            prev_saved_m = saved_m;
            mem.tb_bit[i as usize][j as usize] = trace_bits;
        }
    }

    if best_score <= 0.0 {
        return 0.0;
    }

    let (len_i, len_j, aln_path) = trace_back_bit_sw(mem, la, lb, best_i + 1, best_j + 1);
    assert!(best_i + 1 >= len_i);
    assert!(best_j + 1 >= len_j);
    *lo_i = best_i + 1 - len_i;
    *lo_j = best_j + 1 - len_j;
    *path = aln_path;
    best_score
}
