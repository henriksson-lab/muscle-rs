// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Score the local alignment ending at `(pos_a, pos_b)` along `path`,
/// updating the forward matrix and best-path bookkeeping in place.
#[track_caller]
pub fn on_path_l18<F>(
    pos_a: uint,
    pos_b: uint,
    path: &str,
    fwd_m: &mut [Vec<f32>],
    best_pos_a: &mut uint,
    best_pos_b: &mut uint,
    best_score: &mut f32,
    best_path: &mut String,
    get_local_score: &F,
) where
    F: Fn(uint, uint, &str) -> f32,
{
    let score = get_local_score(pos_a, pos_b, path);
    if score > *best_score {
        *best_score = score;
        *best_pos_a = pos_a;
        *best_pos_b = pos_b;
        *best_path = path.to_string();
    }
    let na = get_na(path);
    let nb = get_nb(path);
    let ix_a = pos_a + na;
    let ix_b = pos_b + nb;
    assert!((ix_a as usize) < fwd_m.len());
    let row_a = &mut fwd_m[ix_a as usize];
    assert!((ix_b as usize) < row_a.len());
    if row_a[ix_b as usize] == f32::MAX || score > row_a[ix_b as usize] {
        row_a[ix_b as usize] = score;
    }
}

/// Enumerate every Smith-Waterman path to fill the forward matrix and
/// return the best score with its endpoints and traceback string.
#[track_caller]
pub fn sw_enum_dp_fwd_m<F>(
    la: uint,
    lb: uint,
    fwd_m: &mut Vec<Vec<f32>>,
    get_local_score: F,
) -> (f32, uint, uint, String)
where
    F: Fn(uint, uint, &str) -> f32,
{
    let mut best_score = 0.0;
    let mut best_pos_a = uint::MAX;
    let mut best_pos_b = uint::MAX;
    let mut best_path = String::new();
    *fwd_m = vec![vec![f32::MAX; lb as usize + 1]; la as usize + 1];
    for i in 0..=la {
        fwd_m[i as usize][0] = 0.0;
    }
    for j in 0..=lb {
        fwd_m[0][j as usize] = 0.0;
    }
    for (pos_a, pos_b, path) in enum_paths_local_l63(la, lb) {
        on_path_l18(
            pos_a,
            pos_b,
            &path,
            fwd_m,
            &mut best_pos_a,
            &mut best_pos_b,
            &mut best_score,
            &mut best_path,
            &get_local_score,
        );
    }
    (best_score, best_pos_a, best_pos_b, best_path)
}

/// Brute-force Smith-Waterman by full path enumeration; thin wrapper that
/// allocates its own forward matrix and returns the best alignment.
#[track_caller]
pub fn sw_enum_dp<F>(la: uint, lb: uint, get_local_score: F) -> (f32, uint, uint, String)
where
    F: Fn(uint, uint, &str) -> f32,
{
    let mut fwd_m = Vec::new();
    sw_enum_dp_fwd_m(la, lb, &mut fwd_m, get_local_score)
}
