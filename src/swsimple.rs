// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Reference Smith-Waterman that fills all three (M, D, I) forward and
/// traceback matrices and returns the best local-alignment score.
#[track_caller]
pub fn sw_simple_fwd_mdi<FMatch, FMM, FMD, FMI, FDM, FDD, FIM, FII>(
    la: uint,
    lb: uint,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
    fwd_m: &mut Vec<Vec<f32>>,
    fwd_d: &mut Vec<Vec<f32>>,
    fwd_i: &mut Vec<Vec<f32>>,
    tbm: &mut Vec<Vec<char>>,
    tbd: &mut Vec<Vec<char>>,
    tbi: &mut Vec<Vec<char>>,
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
    *fwd_m = vec![vec![f32::MAX; lb as usize + 1]; la as usize + 1];
    *fwd_d = vec![vec![f32::MAX; lb as usize + 1]; la as usize + 1];
    *fwd_i = vec![vec![f32::MAX; lb as usize + 1]; la as usize + 1];
    *tbm = vec![vec!['?'; lb as usize + 1]; la as usize + 1];
    *tbd = vec![vec!['?'; lb as usize + 1]; la as usize + 1];
    *tbi = vec![vec!['?'; lb as usize + 1]; la as usize + 1];

    for i in 0..=la as usize {
        fwd_m[i][0] = MINUS_INFINITY;
        fwd_d[i][0] = MINUS_INFINITY;
        fwd_i[i][0] = MINUS_INFINITY;
        tbm[i][0] = 'S';
        tbd[i][0] = '?';
        tbi[i][0] = '?';
    }

    for j in 0..=lb as usize {
        fwd_m[0][j] = MINUS_INFINITY;
        fwd_d[0][j] = MINUS_INFINITY;
        fwd_i[0][j] = MINUS_INFINITY;
        tbm[0][j] = 'S';
        tbd[0][j] = '?';
        tbi[0][j] = '?';
    }

    let mut best_score = 0.0;
    let mut best_i = uint::MAX;
    let mut best_j = uint::MAX;
    for i in 0..la {
        for j in 0..lb {
            let m = get_match_score(i, j);
            let sm = m;
            let mm = fwd_m[i as usize][j as usize] + get_score_mm(i, j) + m;
            let dm = fwd_d[i as usize][j as usize] + get_score_dm(i, j) + m;
            let im = fwd_i[i as usize][j as usize] + get_score_im(i, j) + m;

            let mut s = mm;
            let mut t = 'M';
            if dm > s {
                s = dm;
                t = 'D';
            }
            if im > s {
                s = im;
                t = 'I';
            }
            if sm >= s {
                s = sm;
                t = 'S';
            }

            fwd_m[i as usize + 1][j as usize + 1] = s;
            tbm[i as usize + 1][j as usize + 1] = t;
            if s > best_score {
                best_score = s;
                best_i = i + 1;
                best_j = j + 1;
            }

            let md = fwd_m[i as usize][j as usize + 1] + get_score_md(i, j + 1);
            let dd = fwd_d[i as usize][j as usize + 1] + get_score_dd(i, j + 1);
            if md >= dd {
                fwd_d[i as usize + 1][j as usize + 1] = md;
                tbd[i as usize + 1][j as usize + 1] = 'M';
            } else {
                fwd_d[i as usize + 1][j as usize + 1] = dd;
                tbd[i as usize + 1][j as usize + 1] = 'D';
            }

            let mi = fwd_m[i as usize + 1][j as usize] + get_score_mi(i + 1, j);
            let ii = fwd_i[i as usize + 1][j as usize] + get_score_ii(i + 1, j);
            if mi >= ii {
                fwd_i[i as usize + 1][j as usize + 1] = mi;
                tbi[i as usize + 1][j as usize + 1] = 'M';
            } else {
                fwd_i[i as usize + 1][j as usize + 1] = ii;
                tbi[i as usize + 1][j as usize + 1] = 'I';
            }
        }
    }

    if best_i == uint::MAX {
        return 0.0;
    }

    let mut i = best_i;
    let mut j = best_j;
    path.clear();
    let mut state = 'M';
    loop {
        path.push(state);
        match state {
            'M' => {
                state = tbm[i as usize][j as usize];
                i -= 1;
                j -= 1;
            }
            'D' => {
                state = tbd[i as usize][j as usize];
                i -= 1;
            }
            'I' => {
                state = tbi[i as usize][j as usize];
                j -= 1;
            }
            _ => panic!("invalid traceback state"),
        }

        if i == 0 || j == 0 || state == 'S' {
            break;
        }
    }

    *path = path.chars().rev().collect();
    *lo_a = i;
    *lo_b = j;

    assert!(best_i >= *lo_a);
    assert!(best_j >= *lo_b);
    best_score
}

/// Smith-Waterman that exposes only the match (M) forward matrix to the
/// caller, allocating throwaway D/I/traceback buffers internally.
#[track_caller]
pub fn sw_simple_fwd_m<FMatch, FMM, FMD, FMI, FDM, FDD, FIM, FII>(
    la: uint,
    lb: uint,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
    fwd_m: &mut Vec<Vec<f32>>,
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
    let mut fwd_d = Vec::new();
    let mut fwd_i = Vec::new();
    let mut tbm = Vec::new();
    let mut tbd = Vec::new();
    let mut tbi = Vec::new();
    sw_simple_fwd_mdi(
        la,
        lb,
        lo_a,
        lo_b,
        path,
        fwd_m,
        &mut fwd_d,
        &mut fwd_i,
        &mut tbm,
        &mut tbd,
        &mut tbi,
        get_match_score,
        get_score_mm,
        get_score_md,
        get_score_mi,
        get_score_dm,
        get_score_dd,
        get_score_im,
        get_score_ii,
    )
}

/// Convenience Smith-Waterman wrapper that only returns the score, path
/// and start coordinates.
#[track_caller]
pub fn sw_simple<FMatch, FMM, FMD, FMI, FDM, FDD, FIM, FII>(
    la: uint,
    lb: uint,
    lo_a: &mut uint,
    lo_b: &mut uint,
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
    let mut fwd_m = Vec::new();
    let mut fwd_d = Vec::new();
    let mut fwd_i = Vec::new();
    let mut tbm = Vec::new();
    let mut tbd = Vec::new();
    let mut tbi = Vec::new();
    sw_simple_fwd_mdi(
        la,
        lb,
        lo_a,
        lo_b,
        path,
        &mut fwd_m,
        &mut fwd_d,
        &mut fwd_i,
        &mut tbm,
        &mut tbd,
        &mut tbi,
        get_match_score,
        get_score_mm,
        get_score_md,
        get_score_mi,
        get_score_dm,
        get_score_dd,
        get_score_im,
        get_score_ii,
    )
}
