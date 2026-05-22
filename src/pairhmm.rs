// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const HMMSTATE_M: uint = 0;

pub const HMMSTATE_IX: uint = 1;

pub const HMMSTATE_IY: uint = 2;

pub const HMMSTATE_JX: uint = 3;

pub const HMMSTATE_JY: uint = 4;

pub const HMMSTATE_COUNT: uint64 = 5;

#[derive(Clone, Debug, Default)]
pub struct HMMSTATE; // original: HMMSTATE (muscle/src/pairhmm.h)

#[derive(Clone, Debug, Default)]
pub struct PairHMM; // original: PairHMM (muscle/src/pairhmm.h)

pub static PAIR_HMM_START_SCORE: std::sync::Mutex<[f32; 5]> = std::sync::Mutex::new([0.0; 5]);

pub static PAIR_HMM_TRANS_SCORE: std::sync::Mutex<[[f32; 5]; 5]> =
    std::sync::Mutex::new([[0.0; 5]; 5]);

pub static PAIR_HMM_MATCH_SCORE: std::sync::LazyLock<std::sync::RwLock<[[f32; 256]; 256]>> =
    std::sync::LazyLock::new(|| std::sync::RwLock::new([[0.0; 256]; 256]));

pub static PAIR_HMM_INS_SCORE: std::sync::Mutex<[f32; 256]> = std::sync::Mutex::new([0.0; 256]);

pub static PAIR_HMM_TRANS_MAT: std::sync::LazyLock<std::sync::Mutex<Vec<Vec<f32>>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

pub(crate) static INIT_PROBCONS_DONE: std::sync::Mutex<bool> = std::sync::Mutex::new(false);

/// Builds the 5x5 pair-HMM transition matrix from the gap-open and gap-extend probabilities.
#[track_caller]
pub fn construct_trans_mat(gap_open: &[f32], gap_extend: &[f32]) {
    assert!(gap_open.len() >= 4);
    assert!(gap_extend.len() >= 4);
    let mut trans_mat = PAIR_HMM_TRANS_MAT.lock().unwrap();
    trans_mat.clear();
    trans_mat.resize(HMMSTATE_COUNT as usize, Vec::new());
    for row in trans_mat.iter_mut().take(HMMSTATE_COUNT as usize) {
        row.resize(HMMSTATE_COUNT as usize, 0.0);
    }

    trans_mat[0][0] = 1.0;
    for i in 0..2usize {
        trans_mat[0][2 * i + 1] = gap_open[2 * i];
        trans_mat[0][2 * i + 2] = gap_open[2 * i + 1];
        trans_mat[0][0] -= gap_open[2 * i] + gap_open[2 * i + 1];

        trans_mat[2 * i + 1][2 * i + 1] = gap_extend[2 * i];
        trans_mat[2 * i + 2][2 * i + 2] = gap_extend[2 * i + 1];
        trans_mat[2 * i + 1][2 * i + 2] = 0.0;
        trans_mat[2 * i + 2][2 * i + 1] = 0.0;
        trans_mat[2 * i + 1][0] = 1.0 - gap_extend[2 * i];
        trans_mat[2 * i + 2][0] = 1.0 - gap_extend[2 * i + 1];
    }
    assert!(trans_mat[0][0] > 0.0);
}

/// Populates global start/transition/emit log-score tables from the given probability matrices.
#[track_caller]
pub fn pair_hmm_create2(
    init_distrib_mat: &[f32],
    trans_mat: &[Vec<f32>],
    emit_single: &[f32],
    emit_pairs: &[Vec<f32>],
) {
    assert!(init_distrib_mat.len() >= HMMSTATE_COUNT as usize);
    assert!(trans_mat.len() >= HMMSTATE_COUNT as usize);
    assert!(emit_single.len() >= 256);
    assert!(emit_pairs.len() >= 256);

    let mut start_score = PAIR_HMM_START_SCORE.lock().unwrap();
    for i in 0..HMMSTATE_COUNT as usize {
        start_score[i] = init_distrib_mat[i].ln();
    }
    drop(start_score);

    let mut trans_score = PAIR_HMM_TRANS_SCORE.lock().unwrap();
    for i in 0..HMMSTATE_COUNT as usize {
        assert!(trans_mat[i].len() >= HMMSTATE_COUNT as usize);
        for j in 0..HMMSTATE_COUNT as usize {
            trans_score[i][j] = trans_mat[i][j].ln();
        }
    }
    drop(trans_score);

    let mut ins_score = PAIR_HMM_INS_SCORE.lock().unwrap();
    for i in 0..256usize {
        ins_score[i] = emit_single[i].ln();
    }
    drop(ins_score);

    let mut match_score = PAIR_HMM_MATCH_SCORE.write().unwrap();
    for i in 0..256usize {
        assert!(emit_pairs[i].len() >= 256);
        for j in 0..256usize {
            match_score[i][j] = emit_pairs[i][j].ln();
        }
    }
}

/// Constructs the transition matrix from gap parameters then calls `pair_hmm_create2`.
#[track_caller]
pub fn pair_hmm_create(
    init_distrib_mat: &[f32],
    gap_open: &[f32],
    gap_extend: &[f32],
    emit_pairs: &[Vec<f32>],
    emit_single: &[f32],
) {
    construct_trans_mat(gap_open, gap_extend);
    let trans_mat = PAIR_HMM_TRANS_MAT.lock().unwrap().clone();
    pair_hmm_create2(init_distrib_mat, &trans_mat, emit_single, emit_pairs);
}
