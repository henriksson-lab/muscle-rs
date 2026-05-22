// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Drives PProg using the join order implied by `guide_tree`.
#[track_caller]
pub fn p_prog_run_guide_tree<FAlignAndJoin>(
    pp: &mut PProg,
    guide_tree: &Tree,
    mut align_and_join: FAlignAndJoin,
) where
    FAlignAndJoin: FnMut(&mut PProg, uint, uint),
{
    assert!(pp.input_msa_count > 0);
    pp.join_count = pp.input_msa_count - 1;
    pp.node_count = pp.input_msa_count + pp.join_count;

    let label_to_index = pp
        .msa_label_to_index
        .iter()
        .map(|(label, index)| (label.clone(), *index))
        .collect::<std::collections::HashMap<_, _>>();
    let (indexes1, indexes2) = get_guide_tree_join_order(guide_tree, &label_to_index);
    p_prog_run2(pp, &indexes1, &indexes2, |pp, index1, index2| {
        align_and_join(pp, index1, index2)
    });
}

/// Implements the `pprogt` subcommand: progressive alignment of MSAs guided by an external tree.
#[track_caller]
pub fn cmd_pprogt<FAlignMSAsFlat>(
    list_file_name: &str,
    guide_tree_file_name: &str,
    output_file_name: &str,
    guide_tree_output_file_name: Option<&str>,
    target_pair_count: Option<uint>,
    mut align_msas_flat: FAlignMSAsFlat,
) -> (PProg, Option<Tree>)
where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let msa_file_names = read_strings_from_file(list_file_name)
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    assert!(msa_file_names.len() > 1);

    let mut pp = PProg {
        target_pair_count: target_pair_count.unwrap_or(2000),
        ..PProg::default()
    };
    let is_nucleo = p_prog_load_ms_as(&mut pp, &msa_file_names);
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
    let guide_tree =
        guide_tree_output_file_name.and_then(|file_name| p_prog_write_guide_tree(&pp, file_name));
    (pp, guide_tree)
}
