// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Compute a protein distance for a sequence pair by running a Viterbi
/// alignment and feeding the resulting aligned rows into `get_prot_dist`.
#[track_caller]
pub fn get_prot_dist_seq_pair<FViterbi, FDist>(
    seqi: &[byte],
    li: uint,
    seqj: &[byte],
    lj: uint,
    path: Option<&mut String>,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> f64
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let pi = viterbi_fast_mem(seqi, li, seqj, lj);
    let (row_x, row_y) = make_aln_rows_l45(seqi, li, seqj, lj, &pi);

    let col_count = pi.path.len() as uint;
    assert_eq!(row_x.len() as uint, col_count);
    assert_eq!(row_y.len() as uint, col_count);
    let dij = get_prot_dist(&row_x, &row_y, col_count);
    if let Some(path) = path {
        *path = pi.path.clone();
    }
    dij
}

/// Distance between two sequences identified by index in an MFA.
#[track_caller]
pub fn get_prot_dist_pair_from_mfa<FViterbi, FDist>(
    mfa: &MultiSequence,
    i: uint,
    j: uint,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> f64
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let seqi = mfa.seqs[i as usize]
        .char_vec
        .iter()
        .map(|&c| c as byte)
        .collect::<Vec<_>>();
    let li = seqi.len() as uint;
    let seqj = mfa.seqs[j as usize]
        .char_vec
        .iter()
        .map(|&c| c as byte)
        .collect::<Vec<_>>();
    let lj = seqj.len() as uint;
    let _ = lj;
    get_prot_dist_seq_pair(
        &seqi,
        li,
        &seqj,
        li,
        None,
        |seqi, li, seqj, lj| viterbi_fast_mem(seqi, li, seqj, lj),
        |row_x, row_y, col_count| get_prot_dist(row_x, row_y, col_count),
    )
}

/// Average pairwise protein distance between sampled cross-pairs from two MFAs.
#[track_caller]
pub fn get_prot_dist_mfa_pair<FSeqPair>(
    mfa1: &MultiSequence,
    mfa2: &MultiSequence,
    target_pair_count: uint,
    mut get_prot_dist_seq_pair_fn: FSeqPair,
) -> f64
where
    FSeqPair: FnMut(&[byte], uint, &[byte], uint) -> f64,
{
    assert!(target_pair_count > 0);
    let seq_count1 = mfa1.seqs.len() as uint;
    let seq_count2 = mfa2.seqs.len() as uint;
    let (seq_indexes1, seq_indexes2) = get_pairs(seq_count1, seq_count2, target_pair_count);
    let pair_count = seq_indexes1.len() as uint;
    assert_eq!(seq_indexes2.len() as uint, pair_count);

    let mut sum = 0.0;
    for i in 0..pair_count {
        let seq_index1 = seq_indexes1[i as usize];
        let seq_index2 = seq_indexes2[i as usize];
        let seq1 = mfa1.seqs[seq_index1 as usize]
            .char_vec
            .iter()
            .map(|&c| c as byte)
            .collect::<Vec<_>>();
        let l1 = seq1.len() as uint;
        let seq2 = mfa2.seqs[seq_index2 as usize]
            .char_vec
            .iter()
            .map(|&c| c as byte)
            .collect::<Vec<_>>();
        let l2 = seq2.len() as uint;
        let d = get_prot_dist_seq_pair_fn(&seq1, l1, &seq2, l2);
        sum += d;
    }
    sum / f64::from(pair_count)
}
