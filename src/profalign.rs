// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Align two MSAs by combining their ungapped sequences, computing pairwise
/// posteriors and delegating the final alignment to `align_alns`.
#[track_caller]
pub fn prof_align<FCalcPosterior, FAlignAlns>(
    mpc: &mut MPCFlat,
    msa1: &MultiSequence,
    msa2: &MultiSequence,
    mut calc_posterior: FCalcPosterior,
    mut align_alns: FAlignAlns,
) -> MultiSequence
where
    FCalcPosterior: FnMut(&mut MPCFlat, uint),
    FAlignAlns: FnMut(&mut MPCFlat, &MultiSequence, &MultiSequence) -> MultiSequence,
{
    let seq_count1 = msa1.seqs.len() as uint;
    let _col_count1 = multi_sequence_get_col_count(msa1);
    let seq_count2 = msa2.seqs.len() as uint;
    let _col_count2 = multi_sequence_get_col_count(msa2);

    let mut combined_seqs = MultiSequence::default();
    for seq_index in 0..seq_count1 {
        let seq = &msa1.seqs[seq_index as usize];
        let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
        let mut seq2 = Sequence::default();
        sequence_from_string(&mut seq2, &seq.label, &ungapped);
        combined_seqs.seqs.push(seq2);
        combined_seqs.owners.push(false);
    }
    for seq_index in 0..seq_count2 {
        let seq = &msa2.seqs[seq_index as usize];
        let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
        let mut seq2 = Sequence::default();
        sequence_from_string(&mut seq2, &seq.label, &ungapped);
        combined_seqs.seqs.push(seq2);
        combined_seqs.owners.push(false);
    }

    set_global_input_ms(&combined_seqs);
    mpc_flat_init_seqs(mpc, &combined_seqs);
    mpc_flat_init_pairs(mpc);
    let pair_count = mpc.pairs.len() as uint;
    assert!(pair_count > 0);
    mpc_flat_alloc_pair_count(mpc, pair_count);
    mpc_flat_init_dist_mx(mpc);

    let _pair_count2 = seq_count1 * seq_count2;
    for seq_index1 in 0..seq_count1 {
        for seq_index2 in 0..seq_count2 {
            let pair_index = mpc_flat_get_pair_index(mpc, seq_index1, seq_count1 + seq_index2);
            calc_posterior(mpc, pair_index);
        }
    }

    align_alns(mpc, msa1, msa2)
}

/// `profalign` subcommand: load two input MFAs, configure HMM params and write
/// the aligned profile-profile MSA to the output file.
#[track_caller]
pub fn cmd_profalign<FCmdLineUpdate, FCalcPosterior, FAlignAlns>(
    profalign_file_name: &str,
    input2_file_name: &str,
    output_file_name: &str,
    perturb_seed: Option<uint>,
    mut cmd_line_update: FCmdLineUpdate,
    mut calc_posterior: FCalcPosterior,
    mut align_alns: FAlignAlns,
) -> (HMMParams, MultiSequence)
where
    FCmdLineUpdate: FnMut(&mut HMMParams),
    FCalcPosterior: FnMut(&mut MPCFlat, uint),
    FAlignAlns: FnMut(&mut MPCFlat, &MultiSequence, &MultiSequence) -> MultiSequence,
{
    let mut msa1 = MultiSequence::default();
    let mut msa2 = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa1, profalign_file_name, false);
    multi_sequence_load_mfa_l8(&mut msa2, input2_file_name, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&msa1);
    let is_nucleo2 = multi_sequence_guess_is_nucleo(&msa2);
    assert_eq!(is_nucleo2, is_nucleo);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let perturb_seed = perturb_seed.unwrap_or(0);
    let mut hp = hmm_params_from_defaults(is_nucleo);
    cmd_line_update(&mut hp);
    if perturb_seed > 0 {
        reset_rand(perturb_seed);
        hmm_params_perturb_probs(&mut hp, perturb_seed);
    }
    hmm_params_to_pair_hmm(&hp);

    let mut mpc = MPCFlat::default();
    let msa = prof_align(&mut mpc, &msa1, &msa2, &mut calc_posterior, &mut align_alns);
    multi_sequence_write_mfa(&msa, output_file_name);
    (hp, msa)
}
