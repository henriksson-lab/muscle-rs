// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Apply ProbCons-style consistency relaxation to a single sequence pair using all other sequences as third-party Z.
#[track_caller]
pub fn mpc_flat_cons_pair(mpc: &mut MPCFlat, pair_index: uint) {
    let updated = mpc_flat_cons_pair_result(mpc, pair_index);
    mpc.sparse_posts2[pair_index as usize] = Some(updated);
}

/// Compute one consistency-relaxed pair from `sparse_posts1` without mutating
/// `mpc`, allowing callers to parallelize pairs and commit by pair index.
#[track_caller]
pub fn mpc_flat_cons_pair_result(mpc: &MPCFlat, pair_index: uint) -> MySparseMx {
    let mut scratch = ConsPairScratch::default();
    mpc_flat_cons_pair_result_with_scratch(mpc, pair_index, &mut scratch)
}

#[derive(Default)]
pub struct ConsPairScratch {
    post: Vec<f32>,
}

#[track_caller]
pub fn mpc_flat_cons_pair_result_with_scratch(
    mpc: &MPCFlat,
    pair_index: uint,
    scratch: &mut ConsPairScratch,
) -> MySparseMx {
    let (seq_index_x, seq_index_y) = mpc_flat_get_pair(mpc, pair_index);

    let lx = mpc_flat_get_seq_length(mpc, seq_index_x);
    let ly = mpc_flat_get_seq_length(mpc, seq_index_y);

    let seq_count = mpc_flat_get_seq_count(mpc);
    assert!(seq_index_x < seq_index_y);

    let posts1 = &mpc.sparse_posts1;
    let sparse_post_xy = posts1[pair_index as usize]
        .as_ref()
        .unwrap_or_else(|| panic!("missing sparse post {pair_index}"));
    assert_eq!(sparse_post_xy.lx, lx);
    assert_eq!(sparse_post_xy.ly, ly);

    let post_len = (lx * ly) as usize;
    if scratch.post.len() < post_len {
        scratch.post.resize(post_len, 0.0);
    }
    let post = &mut scratch.post[..post_len];
    post.fill(0.0);
    let ly_u = ly as usize;
    for row in 0..lx as usize {
        let offset = sparse_post_xy.offsets[row] as usize;
        let size = sparse_post_xy.offsets[row + 1] as usize - offset;
        let row_base = row * ly_u;
        for k in 0..size {
            let (p, col) = sparse_post_xy.value_vec[offset + k];
            // SAFETY: sparse_post_xy dimensions were checked against lx/ly.
            unsafe {
                *post.get_unchecked_mut(row_base + col as usize) = p * 2.0;
            }
        }
    }

    // C++ briefly reads m_Weights[SeqIndexZ], then immediately overwrites it
    // with 1.0f. Do not reintroduce that dead read unless intentionally
    // preserving C++'s malformed-state behavior for too-short weights.
    let weight_z = 1.0_f32;
    for seq_index_z in 0..seq_count {
        if seq_index_z == seq_index_x || seq_index_z == seq_index_y {
            continue;
        }
        if seq_index_z < seq_index_x {
            let pair_index_zx = mpc_flat_get_pair_index(mpc, seq_index_z, seq_index_x);
            let pair_index_zy = mpc_flat_get_pair_index(mpc, seq_index_z, seq_index_y);
            let zx = posts1[pair_index_zx as usize]
                .as_ref()
                .unwrap_or_else(|| panic!("missing sparse post {pair_index_zx}"));
            let zy = posts1[pair_index_zy as usize]
                .as_ref()
                .unwrap_or_else(|| panic!("missing sparse post {pair_index_zy}"));
            relax_flat_zx_zy(zx, zy, weight_z, post);
        } else if seq_index_z < seq_index_y {
            let pair_index_xz = mpc_flat_get_pair_index(mpc, seq_index_x, seq_index_z);
            let pair_index_zy = mpc_flat_get_pair_index(mpc, seq_index_z, seq_index_y);
            let xz = posts1[pair_index_xz as usize]
                .as_ref()
                .unwrap_or_else(|| panic!("missing sparse post {pair_index_xz}"));
            let zy = posts1[pair_index_zy as usize]
                .as_ref()
                .unwrap_or_else(|| panic!("missing sparse post {pair_index_zy}"));
            relax_flat_xz_zy(xz, zy, weight_z, post);
        } else {
            let pair_index_xz = mpc_flat_get_pair_index(mpc, seq_index_x, seq_index_z);
            let pair_index_yz = mpc_flat_get_pair_index(mpc, seq_index_y, seq_index_z);
            let xz = posts1[pair_index_xz as usize]
                .as_ref()
                .unwrap_or_else(|| panic!("missing sparse post {pair_index_xz}"));
            let yz = posts1[pair_index_yz as usize]
                .as_ref()
                .unwrap_or_else(|| panic!("missing sparse post {pair_index_yz}"));
            relax_flat_xz_yz(xz, yz, weight_z, post);
        }
    }

    let mut updated_sparse_post_xy = MySparseMx::default();
    my_sparse_mx_update_from_post(&mut updated_sparse_post_xy, sparse_post_xy, post, seq_count);
    updated_sparse_post_xy.x = sparse_post_xy.x.clone();
    updated_sparse_post_xy.y = sparse_post_xy.y.clone();
    updated_sparse_post_xy
}
