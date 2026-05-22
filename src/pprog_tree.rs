// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Implements the `pprog_tree` subcommand: aligns sequences progressively along a supplied guide tree.
#[track_caller]
pub fn cmd_pprog_tree<FAlignMSAsFlat>(
    input_file_name: &str,
    guide_tree_file_name: &str,
    output_file_name: &str,
    target_pair_count: Option<uint>,
    mut align_msas_flat: FAlignMSAsFlat,
) -> PProg
where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    set_global_input_ms(&input_seqs);
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);

    let seq_count = input_seqs.seqs.len() as uint;
    let mut msas = Vec::<MultiSequence>::new();
    let mut msa_labels = Vec::<String>::new();
    for i in 0..seq_count {
        let mut ms = MultiSequence::default();
        ms.seqs.push(input_seqs.seqs[i as usize].clone());
        ms.owners.push(false);
        msas.push(ms);
        msa_labels.push(input_seqs.seqs[i as usize].label.clone());
    }

    let mut pp = PProg {
        target_pair_count: target_pair_count.unwrap_or(2000),
        ..PProg::default()
    };
    p_prog_set_ms_as(&mut pp, &msas, &msa_labels);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    init_probcons();

    let mut tree = Tree::default();
    tree_from_file_l143(&mut tree, guide_tree_file_name);
    p_prog_run_guide_tree(&mut pp, &tree, |pp, index1, index2| {
        p_prog_align_and_join(pp, index1, index2, |label, msa1, msa2, pair_count, path| {
            align_msas_flat(label, msa1, msa2, pair_count, path)
        });
    });

    let final_msa = p_prog_get_final_msa(&pp).clone();
    multi_sequence_write_mfa(&final_msa, output_file_name);
    pp
}
