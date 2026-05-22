// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Aligns the pair `(index1, index2)` and stores the joined MSA in a new slot.
#[track_caller]
pub fn p_prog_align_and_join<FAlignMSAsFlat>(
    pp: &mut PProg,
    index1: uint,
    index2: uint,
    mut align_msas_flat: FAlignMSAsFlat,
) where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    pp.join_msa_indexes1.push(index1);
    pp.join_msa_indexes2.push(index2);

    let msa1 = p_prog_get_msa(pp, index1).clone();
    let msa2 = p_prog_get_msa(pp, index2).clone();
    let _msa_label1 = pp.msa_labels[index1 as usize].clone();
    let _msa_label2 = pp.msa_labels[index2 as usize].clone();
    let progress_str = format!("Join {} / {}", pp.join_index + 1, pp.join_count);

    let mut path = String::new();
    align_msas_flat(&progress_str, &msa1, &msa2, pp.target_pair_count, &mut path);

    let _msa_label12 = format!("Join_{}", pp.join_index + 1);
    let mut msa12 = MultiSequence::default();
    align_ms_as_by_path(&msa1, &msa2, &path, &mut msa12);

    if index1 >= pp.input_msa_count {
        pp.msas[index1 as usize] = None;
    }
    if index2 >= pp.input_msa_count {
        pp.msas[index2 as usize] = None;
    }

    let new_msa_index = pp.input_msa_count + pp.join_index;
    p_prog_set_msa(pp, new_msa_index, &msa12);
}

/// Runs PProg with a pre-specified join order (lists of paired indices).
#[track_caller]
pub fn p_prog_run2<FAlignAndJoin>(
    pp: &mut PProg,
    indexes1: &[uint],
    indexes2: &[uint],
    mut align_and_join: FAlignAndJoin,
) where
    FAlignAndJoin: FnMut(&mut PProg, uint, uint),
{
    assert!(pp.input_msa_count > 0);
    pp.join_count = pp.input_msa_count - 1;
    pp.node_count = pp.input_msa_count + pp.join_count;

    assert_eq!(indexes1.len() as uint, pp.join_count);
    assert_eq!(indexes2.len() as uint, pp.join_count);
    validate_join_order(indexes1, indexes2);

    pp.join_index = 0;
    while pp.join_index < pp.join_count {
        let index1 = indexes1[pp.join_index as usize];
        let index2 = indexes2[pp.join_index as usize];
        align_and_join(pp, index1, index2);
        pp.join_index += 1;
    }
}

/// Implements the `pprog2` subcommand: reads MSAs and an explicit join order, runs PProg.
#[track_caller]
pub fn cmd_pprog2<FAlignMSAsFlat>(
    list_file_name: &str,
    joins_file_name: &str,
    output_file_name: &str,
    target_pair_count: Option<uint>,
    mut align_msas_flat: FAlignMSAsFlat,
) -> PProg
where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let text = std::fs::read_to_string(list_file_name).expect("failed to read PProg2 file list");
    let msa_file_names = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(msa_file_names.len() > 1);

    let mut pp = PProg {
        target_pair_count: 2000,
        ..PProg::default()
    };
    if let Some(pair_count) = target_pair_count {
        pp.target_pair_count = pair_count;
    }

    let _is_nucleo = p_prog_load_ms_as(&mut pp, &msa_file_names);
    set_alpha_l209(if _is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    init_probcons();

    let joins_text = std::fs::read_to_string(joins_file_name).expect("failed to read joins file");
    let mut indexes1 = Vec::new();
    let mut indexes2 = Vec::new();
    for line in joins_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let fields = split(line, '\t');
        assert_eq!(fields.len(), 2);
        indexes1.push(str_to_uint_l1278(&fields[0], false));
        indexes2.push(str_to_uint_l1278(&fields[1], false));
    }

    p_prog_run2(&mut pp, &indexes1, &indexes2, |pp, index1, index2| {
        p_prog_align_and_join(pp, index1, index2, |label, msa1, msa2, pair_count, path| {
            align_msas_flat(label, msa1, msa2, pair_count, path)
        });
    });

    let final_msa = p_prog_get_final_msa(&pp).clone();
    multi_sequence_write_mfa(&final_msa, output_file_name);
    pp
}
