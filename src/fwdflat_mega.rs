// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Mega-profile variant of the pair-HMM forward DP; writes the flat forward matrix.
#[track_caller]
pub fn mega_calc_fwd_flat_mega(profile_x: &[Vec<byte>], profile_y: &[Vec<byte>], flat: &mut [f32]) {
    let lx = profile_x.len() as uint;
    let ly = profile_y.len() as uint;
    if f64::from(lx) * f64::from(ly) * 5.0 + 100.0 > f64::from(i32::MAX) {
        panic!("HMM overflow, sequence lengths {lx}, {ly} (max ~21k)");
    }
    assert!(lx > 0 && ly > 0);
    assert!(flat.len() >= get_fb_size(lx, ly) as usize);

    let start_score = *PAIR_HMM_START_SCORE.lock().unwrap();
    let trans_score = *PAIR_HMM_TRANS_SCORE.lock().unwrap();
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
    let scoring = mega_get_scoring_snapshot();

    let ins_x0 = mega_scoring_get_ins_score(&scoring, profile_x, 0);
    let ins_y0 = mega_scoring_get_ins_score(&scoring, profile_y, 0);
    let emit_x0_y0 = mega_scoring_get_match_score(&scoring, profile_x, 0, profile_y, 0);

    let ly1 = (ly + 1) as usize;
    let base_0_0 = 0usize;
    let base_1_1 = HMMSTATE_COUNT as usize * (ly1 + 1);
    let base_1_0 = HMMSTATE_COUNT as usize * ly1;
    let base_0_1 = HMMSTATE_COUNT as usize;
    let base_inc_i = HMMSTATE_COUNT as usize * ly1;
    let base_inc_j = HMMSTATE_COUNT as usize;

    flat[base_0_0 + HMMSTATE_M as usize] = LOG_ZERO;
    flat[base_0_0 + HMMSTATE_IX as usize] = LOG_ZERO;
    flat[base_0_0 + HMMSTATE_JX as usize] = LOG_ZERO;
    flat[base_0_0 + HMMSTATE_IY as usize] = LOG_ZERO;
    flat[base_0_0 + HMMSTATE_JY as usize] = LOG_ZERO;

    flat[base_1_1 + HMMSTATE_M as usize] = t_sm + emit_x0_y0;
    flat[base_1_0 + HMMSTATE_IX as usize] = t_si + ins_x0;
    flat[base_1_0 + HMMSTATE_JX as usize] = t_sj + ins_x0;
    flat[base_0_1 + HMMSTATE_IY as usize] = t_si + ins_y0;
    flat[base_0_1 + HMMSTATE_JY as usize] = t_sj + ins_y0;

    let mut base = base_1_0;
    for _i in 1..=lx {
        flat[base + HMMSTATE_M as usize] = LOG_ZERO;
        flat[base + HMMSTATE_IY as usize] = LOG_ZERO;
        flat[base + HMMSTATE_JY as usize] = LOG_ZERO;
        base += base_inc_i;
    }

    base = base_0_1;
    for _j in 1..=ly {
        flat[base + HMMSTATE_M as usize] = LOG_ZERO;
        flat[base + HMMSTATE_IX as usize] = LOG_ZERO;
        flat[base + HMMSTATE_JX as usize] = LOG_ZERO;
        base += base_inc_j;
    }

    base = base_1_0;
    let mut next_base = base + base_inc_i;
    for i in 1..lx {
        let emit_x = mega_scoring_get_ins_score(&scoring, profile_x, i);
        flat[next_base + HMMSTATE_IX as usize] = flat[base + HMMSTATE_IX as usize] + t_ii + emit_x;
        flat[next_base + HMMSTATE_JX as usize] = flat[base + HMMSTATE_JX as usize] + t_jj + emit_x;
        base = next_base;
        next_base += base_inc_i;
    }

    base = base_0_1;
    next_base = base + base_inc_j;
    for j in 1..ly {
        let emit_y = mega_scoring_get_ins_score(&scoring, profile_y, j);
        flat[next_base + HMMSTATE_IY as usize] = flat[base + HMMSTATE_IY as usize] + t_ii + emit_y;
        flat[next_base + HMMSTATE_JY as usize] = flat[base + HMMSTATE_JY as usize] + t_jj + emit_y;
        base = next_base;
        next_base += base_inc_j;
    }

    let mut base_i_j = base_1_1;
    let mut base_i1_j = base_0_1;
    let mut base_i_j1 = base_1_0;
    let mut base_i1_j1 = base_0_0;

    for i in 1..=lx {
        let emit_x = mega_scoring_get_ins_score(&scoring, profile_x, i - 1);
        for j in 1..=ly {
            let emit_y = mega_scoring_get_ins_score(&scoring, profile_y, j - 1);
            let emit_pair =
                mega_scoring_get_match_score(&scoring, profile_x, i - 1, profile_y, j - 1);
            if i == 1 && j == 1 {
                flat[base_1_1 + HMMSTATE_M as usize] = t_sm + emit_x0_y0;
            } else {
                // Right-associative LOG_ADD grouping (scoretype.h:138).
                let m_m = flat[base_i1_j1 + HMMSTATE_M as usize] + t_mm;
                let ix_m = flat[base_i1_j1 + HMMSTATE_IX as usize] + t_im;
                let jx_m = flat[base_i1_j1 + HMMSTATE_JX as usize] + t_jm;
                let iy_m = flat[base_i1_j1 + HMMSTATE_IY as usize] + t_im;
                let jy_m = flat[base_i1_j1 + HMMSTATE_JY as usize] + t_jm;
                let sum_prev = log_add_hack(
                    m_m,
                    log_add_hack(ix_m, log_add_hack(jx_m, log_add_hack(iy_m, jy_m))),
                );
                flat[base_i_j + HMMSTATE_M as usize] = sum_prev + emit_pair;
            }

            let prev_m_i1_j = flat[base_i1_j + HMMSTATE_M as usize];
            let prev_m_i_j1 = flat[base_i_j1 + HMMSTATE_M as usize];
            flat[base_i_j + HMMSTATE_IX as usize] = log_add_hack(
                flat[base_i1_j + HMMSTATE_IX as usize] + t_ii,
                prev_m_i1_j + t_mi,
            ) + emit_x;
            flat[base_i_j + HMMSTATE_JX as usize] = log_add_hack(
                flat[base_i1_j + HMMSTATE_JX as usize] + t_jj,
                prev_m_i1_j + t_mj,
            ) + emit_x;
            flat[base_i_j + HMMSTATE_IY as usize] = log_add_hack(
                flat[base_i_j1 + HMMSTATE_IY as usize] + t_ii,
                prev_m_i_j1 + t_mi,
            ) + emit_y;
            flat[base_i_j + HMMSTATE_JY as usize] = log_add_hack(
                flat[base_i_j1 + HMMSTATE_JY as usize] + t_jj,
                prev_m_i_j1 + t_mj,
            ) + emit_y;

            base_i_j += base_inc_j;
            base_i1_j += base_inc_j;
            base_i_j1 += base_inc_j;
            base_i1_j1 += base_inc_j;
        }

        base_i_j += base_inc_j;
        base_i1_j += base_inc_j;
        base_i_j1 += base_inc_j;
        base_i1_j1 += base_inc_j;
    }
}
