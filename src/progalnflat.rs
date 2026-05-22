// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Release intermediate progressive MSAs stored on the MPCFlat state.
#[track_caller]
pub fn mpc_flat_free_prog_ms_as(mpc: &mut MPCFlat) {
    mpc.prog_msas.clear();
}

/// Release cached sparse posterior matrices held on the MPCFlat state.
#[track_caller]
pub fn mpc_flat_free_sparse_posts(mpc: &mut MPCFlat) {
    for mx in &mut mpc.sparse_posts1 {
        *mx = None;
    }
    for mx in &mut mpc.sparse_posts2 {
        *mx = None;
    }
    mpc.sparse_posts1.clear();
    mpc.sparse_posts2.clear();
}

/// Perform a single progressive-alignment join: combine the two MSAs referenced
/// by `join_index` into a new MSA appended to `prog_msas`.
#[track_caller]
pub fn mpc_flat_prog_aln<FAlignAlns>(
    mpc: &mut MPCFlat,
    join_index: uint,
    mut align_alns: FAlignAlns,
) where
    FAlignAlns: FnMut(&mut MPCFlat, &MultiSequence, &MultiSequence) -> MultiSequence,
{
    let index1 = mpc.join_indexes1[join_index as usize];
    let index2 = mpc.join_indexes2[join_index as usize];
    assert!((index1 as usize) < mpc.prog_msas.len());
    assert!((index2 as usize) < mpc.prog_msas.len());

    let msa1 = mpc.prog_msas[index1 as usize]
        .take()
        .expect("MPCFlat::ProgAln MSA1 is null");
    let msa2 = mpc.prog_msas[index2 as usize]
        .take()
        .expect("MPCFlat::ProgAln MSA2 is null");
    let msa12 = align_alns(mpc, &msa1, &msa2);
    mpc.prog_msas.push(Some(msa12));
    let _seq_count = mpc_flat_get_seq_count(mpc);
}

/// Run progressive alignment over all input sequences following the join order
/// in `join_indexes{1,2}`; the final MSA ends up in `mpc.msa`.
#[track_caller]
pub fn mpc_flat_progressive_align<FAlignAlns>(mpc: &mut MPCFlat, mut align_alns: FAlignAlns)
where
    FAlignAlns: FnMut(&mut MPCFlat, &MultiSequence, &MultiSequence) -> MultiSequence,
{
    let input = mpc
        .my_input_seqs
        .as_ref()
        .expect("MPCFlat::ProgressiveAlign no input seqs")
        .clone();
    let seq_count = input.seqs.len() as uint;
    let join_count = seq_count - 1;
    let node_count = seq_count + join_count;

    for i in 0..seq_count {
        let seq = input.seqs[i as usize].clone();
        let mut ms = MultiSequence::default();
        ms.seqs.push(seq);
        ms.owners.push(false);
        mpc.prog_msas.push(Some(ms));
    }

    assert_eq!(mpc.join_indexes1.len() as uint, join_count);
    assert_eq!(mpc.join_indexes2.len() as uint, join_count);
    validate_join_order(&mpc.join_indexes1, &mpc.join_indexes2);

    for join_index in 0..join_count {
        mpc_flat_prog_aln(mpc, join_index, |mpc, msa1, msa2| {
            align_alns(mpc, msa1, msa2)
        });
    }

    assert_eq!(mpc.prog_msas.len() as uint, node_count);
    mpc.msa = mpc.prog_msas[(node_count - 1) as usize].take();
    mpc_flat_free_prog_ms_as(mpc);
    assert!(mpc.msa.is_some());
}
