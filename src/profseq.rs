// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Align a single sequence against an MSA via per-row posteriors, returning the
/// MSA-vs-sequence alignment path.
#[track_caller]
pub fn prof_seq<FCalcPosterior>(
    mpc: &mut MPCFlat,
    msa1: &MultiSequence,
    seq2: &Sequence,
    mut calc_posterior: FCalcPosterior,
) -> String
where
    FCalcPosterior: FnMut(&mut MPCFlat, uint),
{
    let mut path = String::new();
    let seq_count1 = msa1.seqs.len() as uint;
    let col_count1 = multi_sequence_get_col_count(msa1);
    let seq_length2 = seq2.char_vec.len() as uint;

    let mut combined_seqs = MultiSequence::default();
    for seq_index in 0..seq_count1 {
        let seq = &msa1.seqs[seq_index as usize];
        let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq));
        let mut seq2dg = Sequence::default();
        sequence_from_string(&mut seq2dg, &seq.label, &ungapped);
        combined_seqs.seqs.push(seq2dg);
        combined_seqs.owners.push(false);
    }

    let ungapped = sequence_copy_delete_gaps(&sequence_get_seq_as_string(seq2));
    let mut seq2dg = Sequence::default();
    sequence_from_string(&mut seq2dg, &seq2.label, &ungapped);
    combined_seqs.seqs.push(seq2dg);
    combined_seqs.owners.push(false);

    set_global_input_ms(&combined_seqs);
    mpc_flat_init_seqs(mpc, &combined_seqs);
    mpc_flat_init_pairs(mpc);
    let pair_count = mpc.pairs.len() as uint;
    assert!(pair_count > 0);
    mpc_flat_alloc_pair_count(mpc, pair_count);
    mpc_flat_init_dist_mx(mpc);
    if mpc.weights.len() < combined_seqs.seqs.len() {
        mpc.weights = vec![1.0; combined_seqs.seqs.len()];
    }

    for seq_index1 in 0..seq_count1 {
        let pair_index = mpc_flat_get_pair_index(mpc, seq_index1, seq_count1);
        calc_posterior(mpc, pair_index);
    }

    let mut msa2 = MultiSequence::default();
    msa2.seqs.push(seq2.clone());
    msa2.owners.push(false);

    let post = mpc_flat_build_post(mpc, msa1, &msa2);
    let mut dp_rows = alloc_dp_rows(col_count1, seq_length2);
    let mut tb = alloc_tb(col_count1, seq_length2);
    let (_score, aln_path) = calc_aln_flat(&post, col_count1, seq_length2, &mut dp_rows, &mut tb);
    path.push_str(&aln_path);
    path
}

/// `profseq` subcommand: align each query sequence against the input MSA and
/// return the alignment paths.
#[track_caller]
pub fn cmd_profseq<FCmdLineUpdate, FCalcPosterior>(
    profseq_file_name: &str,
    input2_file_name: &str,
    perturb_seed: Option<uint>,
    mut cmd_line_update: FCmdLineUpdate,
    mut calc_posterior: FCalcPosterior,
) -> (HMMParams, Vec<String>)
where
    FCmdLineUpdate: FnMut(&mut HMMParams),
    FCalcPosterior: FnMut(&mut MPCFlat, uint),
{
    let mut msa1 = MultiSequence::default();
    let mut query = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa1, profseq_file_name, false);
    multi_sequence_load_mfa_l8(&mut query, input2_file_name, true);
    let is_nucleo = multi_sequence_guess_is_nucleo(&msa1);
    let is_nucleo2 = multi_sequence_guess_is_nucleo(&query);
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

    let mut paths = Vec::new();
    for seq in &query.seqs {
        let mut mpc = MPCFlat::default();
        let path = prof_seq(&mut mpc, &msa1, seq, |mpc, pair_index| {
            calc_posterior(mpc, pair_index)
        });
        paths.push(path);
    }
    (hp, paths)
}
