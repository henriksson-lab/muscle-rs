// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// One consistency-transformation pass: run `cons_pair` on every pair, then swap in the updated posteriors.
#[track_caller]
pub fn mpc_flat_cons_iter<FConsPair>(mpc: &mut MPCFlat, _iter: uint, mut cons_pair: FConsPair)
where
    FConsPair: FnMut(&mut MPCFlat, uint),
{
    let pair_count = mpc.pairs.len() as uint;
    assert!(pair_count > 0);
    for pair_index in 0..pair_count {
        cons_pair(mpc, pair_index);
    }
    std::mem::swap(&mut mpc.sparse_posts1, &mut mpc.sparse_posts2);
}

/// Deterministic parallel consistency pass for the standard `mpc_flat_cons_pair`
/// implementation. Pair results are computed independently, then installed in
/// ascending pair-index order before the double-buffer swap.
#[track_caller]
pub fn mpc_flat_cons_iter_parallel_pairs(mpc: &mut MPCFlat, _iter: uint) {
    let pair_count = mpc.pairs.len() as uint;
    assert!(pair_count > 0);
    let thread_count = get_requested_thread_count().min(pair_count).max(1);

    if thread_count == 1 || pair_count == 1 {
        for pair_index in 0..pair_count {
            mpc_flat_cons_pair(mpc, pair_index);
        }
    } else {
        let mpc_addr = mpc as *mut MPCFlat as usize;
        let sparse_posts2_addr = mpc.sparse_posts2.as_mut_ptr() as usize;
        std::thread::scope(|scope| {
            for thread_index in 0..thread_count {
                let start = (pair_count * thread_index) / thread_count;
                let end = (pair_count * (thread_index + 1)) / thread_count;
                scope.spawn(move || {
                    for pair_index in start..end {
                        // SAFETY: Each worker reads the immutable current
                        // sparse_posts1 set and writes exactly one disjoint
                        // sparse_posts2[pair_index] slot per pair. Vector
                        // lengths and all read-only MPC metadata are stable for
                        // the duration of the scoped threads.
                        unsafe {
                            let mpc_ptr = mpc_addr as *mut MPCFlat;
                            let result = mpc_flat_cons_pair_result(&*mpc_ptr, pair_index);
                            let slot = (sparse_posts2_addr as *mut Option<MySparseMx>)
                                .add(pair_index as usize);
                            let old = std::ptr::replace(slot, Some(result));
                            drop(old);
                        }
                    }
                });
            }
        });
    }

    std::mem::swap(&mut mpc.sparse_posts1, &mut mpc.sparse_posts2);
}

#[track_caller]
pub fn mpc_flat_consistency_parallel_pairs(mpc: &mut MPCFlat) {
    let seq_count = mpc_flat_get_seq_count(mpc);
    if seq_count < 3 {
        return;
    }
    for iter in 0..mpc.consistency_iter_count {
        mpc_flat_cons_iter_parallel_pairs(mpc, iter);
    }
}
