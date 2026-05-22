// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

unsafe extern "C" {
    /// C library `rand()` — the *one* place MUSCLE C++ uses `<cstdlib>` rand
    /// instead of its own `RandInt32` is `refineflat.cpp:14`. The
    /// `libc_rand()` helper below wraps it; production callers route through
    /// it (see `app.rs`) so the refinement bipartition matches the C++ binary
    /// byte-for-byte. No `srand` is ever called by MUSCLE, so we get the
    /// deterministic default seed sequence.
    fn rand() -> std::ffi::c_int;
}

/// Thin wrapper around libc `rand()` so callers don't need `unsafe`.
pub fn libc_rand() -> uint {
    (unsafe { rand() }) as uint
}

/// One MPCFlat refinement iteration: randomly bipartition seqs, project, and re-align the two MSAs.
#[track_caller]
pub fn mpc_flat_refine_iter<FRand, FAlignAlns>(
    mpc: &mut MPCFlat,
    mut rand_value: FRand,
    mut align_alns: FAlignAlns,
) where
    FRand: FnMut() -> uint,
    FAlignAlns: FnMut(&mut MPCFlat, &MultiSequence, &MultiSequence) -> MultiSequence,
{
    let mut seq_indexes1 = std::collections::BTreeSet::new();
    let mut seq_indexes2 = std::collections::BTreeSet::new();

    let seq_count = mpc_flat_get_seq_count(mpc);
    let msa = mpc.msa.as_ref().expect("MPCFlat::RefineIter, no MSA");
    assert_eq!(msa.seqs.len() as uint, seq_count);

    for seq_index in 0..seq_count {
        if rand_value() % 2 == 0 {
            seq_indexes1.insert(seq_index as i32);
        } else {
            seq_indexes2.insert(seq_index as i32);
        }
    }

    if seq_indexes1.is_empty() || seq_indexes2.is_empty() {
        return;
    }

    let msa1 = multi_sequence_project_l16(msa, &seq_indexes1);
    let msa2 = multi_sequence_project_l16(msa, &seq_indexes2);
    let msa12 = align_alns(mpc, &msa1, &msa2);
    mpc.msa = Some(msa12);
}
