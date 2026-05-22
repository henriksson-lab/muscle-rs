// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct QScorer3 {
    pub test1: MultiSequence,
    pub test2: MultiSequence,
    pub ref_msa: MultiSequence,
    pub qs1: QScorer,
    pub qs2: QScorer,
    pub indexes2: Vec<uint>,
    pub ref_cols: Vec<uint>,
    pub ref_aligned_col_count: uint,
    pub pairs: Vec<(uint, uint)>,
    pub pair_index_to_q1: Vec<f32>,
    pub pair_index_to_q2: Vec<f32>,
    pub pair_index_to_pwc: Vec<f32>,
} // original: QScorer3 (muscle/src/qscorer3.h)
