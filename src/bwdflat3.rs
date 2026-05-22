// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Backward HMM pass: `Bwd[s][i][j]` is the probability of starting in state `s`
/// and aligning the last `LX-i` letters of `X` to the last `LY-j` letters of `Y`.
#[track_caller]
pub fn calc_bwd_flat_l10(x: &[byte], lx: uint, y: &[byte], ly: uint, flat: &mut [f32]) {
    if f64::from(lx) * f64::from(ly) * 5.0 + 100.0 > f64::from(i32::MAX) {
        panic!("HMM overflow, sequence lengths {lx}, {ly} (max ~21k)");
    }
    assert!(x.len() >= lx as usize);
    assert!(y.len() >= ly as usize);
    assert!(flat.len() >= get_fb_size(lx, ly) as usize);

    let start_score = *PAIR_HMM_START_SCORE.lock().unwrap();
    let trans_score = *PAIR_HMM_TRANS_SCORE.lock().unwrap();
    let ins_score = *PAIR_HMM_INS_SCORE.lock().unwrap();
    let match_score = PAIR_HMM_MATCH_SCORE.read().unwrap();

    let t_sm = start_score[HMMSTATE_M as usize];
    let t_si = start_score[HMMSTATE_IX as usize];
    let t_sj = start_score[HMMSTATE_JX as usize];
    let t_mm = trans_score[HMMSTATE_M as usize][HMMSTATE_M as usize];
    let t_mi = trans_score[HMMSTATE_M as usize][HMMSTATE_IX as usize];
    let t_mj = trans_score[HMMSTATE_M as usize][HMMSTATE_JX as usize];
    let t_ii = trans_score[HMMSTATE_IX as usize][HMMSTATE_IX as usize];
    let t_im = trans_score[HMMSTATE_IX as usize][HMMSTATE_M as usize];
    let t_jj = trans_score[HMMSTATE_JX as usize][HMMSTATE_JX as usize];
    let t_jm = trans_score[HMMSTATE_JX as usize][HMMSTATE_M as usize];

    let i_lx = lx as isize;
    let i_ly = ly as isize;
    let ly1 = ly as isize + 1;
    let base_inc_i = HMMSTATE_COUNT as isize * ly1;
    let base_inc_j = HMMSTATE_COUNT as isize;

    let mut base = HMMSTATE_COUNT as usize * ly as usize;
    for _i in 0..i_lx {
        flat[base + HMMSTATE_IY as usize] = LOG_ZERO;
        flat[base + HMMSTATE_JY as usize] = LOG_ZERO;
        base = (base as isize + base_inc_i) as usize;
    }

    base = HMMSTATE_COUNT as usize * (lx as usize * (ly as usize + 1));
    for _j in 0..i_ly {
        flat[base + HMMSTATE_IX as usize] = LOG_ZERO;
        flat[base + HMMSTATE_JX as usize] = LOG_ZERO;
        base = (base as isize + base_inc_j) as usize;
    }

    let mut base_i_j = HMMSTATE_COUNT as isize * (lx as isize * ly1 + ly as isize);
    let mut base_i1_j = base_i_j + base_inc_i;
    let mut base_i_j1 = base_i_j + base_inc_j;
    let mut base_i1_j1 = base_i_j + base_inc_i + base_inc_j;

    for i in (0..=i_lx).rev() {
        let x_char = if i == i_lx { 0 } else { x[i as usize] };
        let emit_x = ins_score[x_char as usize];
        let match_row = &match_score[x_char as usize];
        for j in (0..=i_ly).rev() {
            if i == i_lx && j == i_ly {
                let b = base_i_j as usize;
                flat[b + HMMSTATE_M as usize] = t_sm;
                flat[b + HMMSTATE_IX as usize] = t_si;
                flat[b + HMMSTATE_IY as usize] = t_si;
                flat[b + HMMSTATE_JX as usize] = t_sj;
                flat[b + HMMSTATE_JY as usize] = t_sj;
                base_i_j -= base_inc_j;
                base_i1_j -= base_inc_j;
                base_i_j1 -= base_inc_j;
                base_i1_j1 -= base_inc_j;
                continue;
            }

            let y_char = if j == i_ly { 0 } else { y[j as usize] };
            let emit_y = ins_score[y_char as usize];
            let emit_xy = match_row[y_char as usize];
            let b = base_i_j as usize;

            if i < i_lx && j < i_ly {
                // SAFETY: flat size and sequence lengths were checked above;
                // this branch only indexes valid successor DP cells.
                unsafe {
                    let next_m =
                        *flat.get_unchecked(base_i1_j1 as usize + HMMSTATE_M as usize) + emit_xy;
                    let next_ix =
                        *flat.get_unchecked(base_i1_j as usize + HMMSTATE_IX as usize) + emit_x;
                    let next_jx =
                        *flat.get_unchecked(base_i1_j as usize + HMMSTATE_JX as usize) + emit_x;
                    let next_iy =
                        *flat.get_unchecked(base_i_j1 as usize + HMMSTATE_IY as usize) + emit_y;
                    let next_jy =
                        *flat.get_unchecked(base_i_j1 as usize + HMMSTATE_JY as usize) + emit_y;

                    if i > 0 && j > 0 {
                        // Match the right-associative LOG_ADD grouping from
                        // scoretype.h:138: M_M + (M_IX + (M_JX + (M_IY + M_JY))).
                        let m_m = t_mm + next_m;
                        let m_ix = t_mi + next_ix;
                        let m_jx = t_mj + next_jx;
                        let m_iy = t_mi + next_iy;
                        let m_jy = t_mj + next_jy;
                        *flat.get_unchecked_mut(b + HMMSTATE_M as usize) = log_add_hack(
                            m_m,
                            log_add_hack(m_ix, log_add_hack(m_jx, log_add_hack(m_iy, m_jy))),
                        );
                    } else {
                        *flat.get_unchecked_mut(b + HMMSTATE_M as usize) = LOG_ZERO;
                    }

                    if i > 0 {
                        *flat.get_unchecked_mut(b + HMMSTATE_IX as usize) =
                            log_add_hack(t_ii + next_ix, t_im + next_m);
                        *flat.get_unchecked_mut(b + HMMSTATE_JX as usize) =
                            log_add_hack(t_jj + next_jx, t_jm + next_m);
                    } else {
                        *flat.get_unchecked_mut(b + HMMSTATE_IX as usize) = LOG_ZERO;
                        *flat.get_unchecked_mut(b + HMMSTATE_JX as usize) = LOG_ZERO;
                    }

                    if j > 0 {
                        *flat.get_unchecked_mut(b + HMMSTATE_IY as usize) =
                            log_add_hack(t_ii + next_iy, t_im + next_m);
                        *flat.get_unchecked_mut(b + HMMSTATE_JY as usize) =
                            log_add_hack(t_jj + next_jy, t_jm + next_m);
                    } else {
                        *flat.get_unchecked_mut(b + HMMSTATE_IY as usize) = LOG_ZERO;
                        *flat.get_unchecked_mut(b + HMMSTATE_JY as usize) = LOG_ZERO;
                    }
                }

                base_i_j -= base_inc_j;
                base_i1_j -= base_inc_j;
                base_i_j1 -= base_inc_j;
                base_i1_j1 -= base_inc_j;
                continue;
            }

            if i < i_lx {
                assert_eq!(j, i_ly);
                if i > 0 {
                    let next_ix = flat[base_i1_j as usize + HMMSTATE_IX as usize] + emit_x;
                    let next_jx = flat[base_i1_j as usize + HMMSTATE_JX as usize] + emit_x;
                    flat[b + HMMSTATE_M as usize] = log_add_hack(t_mi + next_ix, t_mj + next_jx);
                    flat[b + HMMSTATE_IX as usize] = t_ii + next_ix;
                    flat[b + HMMSTATE_JX as usize] = t_jj + next_jx;
                } else {
                    flat[b + HMMSTATE_M as usize] = LOG_ZERO;
                    flat[b + HMMSTATE_IX as usize] = LOG_ZERO;
                    flat[b + HMMSTATE_JX as usize] = LOG_ZERO;
                }
            }

            if j < i_ly {
                assert_eq!(i, i_lx);
                let next_iy = flat[base_i_j1 as usize + HMMSTATE_IY as usize] + emit_y;
                let next_jy = flat[base_i_j1 as usize + HMMSTATE_JY as usize] + emit_y;
                if j > 0 {
                    flat[b + HMMSTATE_M as usize] = log_add_hack(t_mi + next_iy, t_mj + next_jy);
                    flat[b + HMMSTATE_IY as usize] = t_ii + next_iy;
                    flat[b + HMMSTATE_JY as usize] = t_jj + next_jy;
                } else {
                    flat[b + HMMSTATE_M as usize] = LOG_ZERO;
                    flat[b + HMMSTATE_IY as usize] = LOG_ZERO;
                    flat[b + HMMSTATE_JY as usize] = LOG_ZERO;
                }
            }

            base_i_j -= base_inc_j;
            base_i1_j -= base_inc_j;
            base_i_j1 -= base_inc_j;
            base_i1_j1 -= base_inc_j;
        }
    }
}
