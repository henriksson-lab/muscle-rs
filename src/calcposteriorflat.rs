// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Converts forward and backward HMM matrices into a dense posterior matrix,
/// normalizing by total probability and clamping out-of-range scores.
#[track_caller]
pub fn calc_post_flat(flat_fwd: &[f32], flat_bwd: &[f32], lx: uint, ly: uint, post: &mut [f32]) {
    assert!(post.len() >= (lx * ly) as usize);
    assert!(flat_fwd.len() >= get_fb_size(lx, ly) as usize);
    assert!(flat_bwd.len() >= get_fb_size(lx, ly) as usize);
    let total = calc_total_prob_flat(flat_fwd, flat_bwd, lx, ly);

    let state_count = HMMSTATE_COUNT as usize;
    let ly1 = (ly + 1) as usize;
    let mut ix_fb = state_count * (ly1 + 1);
    let mut ix_post = 0usize;
    for _i in 0..lx {
        for _j in 0..ly {
            // SAFETY: the full flat/post lengths are asserted above and the
            // loop follows C++'s `M[1,1]..M[LX,LY]` flat traversal exactly.
            unsafe {
                let score = *flat_fwd.get_unchecked(ix_fb) + *flat_bwd.get_unchecked(ix_fb) - total;
                *post.get_unchecked_mut(ix_post) = if score < MIN_SPARSE_SCORE {
                    0.0
                } else if score >= LOG_ONE {
                    1.0
                } else {
                    score.exp()
                };
            }
            ix_post += 1;
            ix_fb += state_count;
        }
        ix_fb += state_count;
    }
}

/// MPCFlat hook that fetches the two byte sequences by global index and runs
/// the forward HMM pass into `flat`.
#[track_caller]
pub fn mpc_flat_calc_fwd_flat_mpc_flat(
    _mpc: &MPCFlat,
    gsix: uint,
    lx: uint,
    gsiy: uint,
    ly: uint,
    flat: &mut [f32],
) {
    let x = get_byte_seq_by_gsi(gsix);
    let y = get_byte_seq_by_gsi(gsiy);
    calc_fwd_flat_l12(&x, lx, &y, ly, flat);
}

/// MPCFlat hook that fetches the two byte sequences by global index and runs
/// the backward HMM pass into `flat`.
#[track_caller]
pub fn mpc_flat_calc_bwd_flat_mpc_flat(
    _mpc: &MPCFlat,
    gsix: uint,
    lx: uint,
    gsiy: uint,
    ly: uint,
    flat: &mut [f32],
) {
    let x = get_byte_seq_by_gsi(gsix);
    let y = get_byte_seq_by_gsi(gsiy);
    calc_bwd_flat_l10(&x, lx, &y, ly, flat);
}

/// Computes the posterior for one sequence pair: stores its sparse form on the
/// MPCFlat and updates the EA-based distance matrix in both directions.
#[track_caller]
pub fn mpc_flat_calc_posterior(mpc: &mut MPCFlat, pair_index: uint) {
    let (sparse_post, ea, seq_index_x, seq_index_y) =
        mpc_flat_calc_posterior_result(mpc, pair_index);
    mpc.sparse_posts1[pair_index as usize] = Some(sparse_post);
    assert!((seq_index_x as usize) < mpc.dist_mx.len());
    assert!((seq_index_y as usize) < mpc.dist_mx.len());
    mpc.dist_mx[seq_index_x as usize][seq_index_y as usize] = ea;
    mpc.dist_mx[seq_index_y as usize][seq_index_x as usize] = ea;
}

#[derive(Default)]
pub struct PosteriorScratch {
    fwd: Vec<f32>,
    bwd: Vec<f32>,
    post: Vec<f32>,
    dp_rows: Vec<f32>,
}

fn scratch_f32(slice: &mut Vec<f32>, len: usize, fill: bool) -> &mut [f32] {
    if slice.len() < len {
        slice.resize(len, 0.0);
    }
    let live = &mut slice[..len];
    if fill {
        live.fill(0.0);
    }
    live
}

/// Compute one pair posterior without mutating `mpc`, so callers can run many
/// pairs in parallel and commit results deterministically by pair index.
#[track_caller]
pub fn mpc_flat_calc_posterior_result(
    mpc: &MPCFlat,
    pair_index: uint,
) -> (MySparseMx, f32, uint, uint) {
    let input_seqs = mpc
        .my_input_seqs
        .as_ref()
        .expect("MPCFlat::CalcPosterior, no input seqs");
    mpc_flat_calc_posterior_result_for_pair(mpc_flat_get_pair(mpc, pair_index), input_seqs)
}

/// Compute one pair posterior from immutable pair/input-sequence state.
#[track_caller]
pub fn mpc_flat_calc_posterior_result_for_pair(
    pair: (uint, uint),
    input_seqs: &MultiSequence,
) -> (MySparseMx, f32, uint, uint) {
    let mut scratch = PosteriorScratch::default();
    mpc_flat_calc_posterior_result_for_pair_with_scratch(pair, input_seqs, &mut scratch)
}

#[track_caller]
pub fn mpc_flat_calc_posterior_result_for_pair_with_scratch(
    pair: (uint, uint),
    input_seqs: &MultiSequence,
    scratch: &mut PosteriorScratch,
) -> (MySparseMx, f32, uint, uint) {
    let (seq_index_x, seq_index_y) = pair;
    let seq_x = &input_seqs.seqs[seq_index_x as usize];
    let seq_y = &input_seqs.seqs[seq_index_y as usize];
    let lx = seq_x.char_vec.len() as uint;
    let ly = seq_y.char_vec.len() as uint;
    if f64::from(lx) * f64::from(ly) * 5.0 + 100.0 > f64::from(i32::MAX) {
        panic!("HMM overflow, sequence lengths {lx}, {ly} (max ~21k)");
    }

    let x: Vec<byte> = seq_x.char_vec.iter().map(|&c| c as byte).collect();
    let y: Vec<byte> = seq_y.char_vec.iter().map(|&c| c as byte).collect();
    let fb_len = get_fb_size(lx, ly) as usize;
    let fwd = scratch_f32(&mut scratch.fwd, fb_len, true);
    let bwd = scratch_f32(&mut scratch.bwd, fb_len, true);
    if MEGA_LOADED.load(std::sync::atomic::Ordering::Relaxed) {
        mega_with_profiles_by_label(&seq_x.label, &seq_y.label, |profile_x, profile_y| {
            assert_eq!(profile_x.len(), lx as usize);
            assert_eq!(profile_y.len(), ly as usize);
            mega_calc_fwd_flat_mega(profile_x, profile_y, fwd);
            mega_calc_bwd_flat_mega(profile_x, profile_y, bwd);
        });
    } else {
        calc_fwd_flat_l12(&x, lx, &y, ly, fwd);
        calc_bwd_flat_l10(&x, lx, &y, ly, bwd);
    }
    let post_len = get_post_size(lx, ly) as usize;
    let post = scratch_f32(&mut scratch.post, post_len, false);
    calc_post_flat(fwd, bwd, lx, ly, post);

    let mut sparse_post = MySparseMx::default();
    my_sparse_mx_from_post(&mut sparse_post, post, lx, ly);
    sparse_post.x = Some(x);
    sparse_post.y = Some(y);

    let dp_len = get_dp_rows_size(lx, ly) as usize;
    let dp_rows = scratch_f32(&mut scratch.dp_rows, dp_len, false);
    let score = calc_aln_score_flat(post, lx, ly, dp_rows);
    let ea = score / lx.min(ly) as f32;
    (sparse_post, ea, seq_index_x, seq_index_y)
}
