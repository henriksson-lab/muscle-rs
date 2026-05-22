// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Build an EA distance matrix between MSAs listed in `list_file_name` using `align_msas_flat`; write a labelled TSV.
#[track_caller]
pub fn cmd_eadistmx_msas<FAlignMSAsFlat>(
    list_file_name: &str,
    output_file_name: &str,
    target_pair_count: Option<uint>,
    mut align_msas_flat: FAlignMSAsFlat,
) -> PProg
where
    FAlignMSAsFlat: FnMut(&str, &MultiSequence, &MultiSequence, uint, &mut String) -> f32,
{
    let text = std::fs::read_to_string(list_file_name).expect("failed to read MSA file list");
    let msa_file_names = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let msa_count = msa_file_names.len() as uint;
    assert!(!output_file_name.is_empty());
    let mut pp = PProg::default();
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

    p_prog_align_all_input_pairs(&mut pp, |label, msa1, msa2, pair_count, path| {
        align_msas_flat(label, msa1, msa2, pair_count, path)
    });

    let mut out = String::new();
    for i in 0..msa_count {
        assert!((i as usize) < pp.score_mx.len());
        let labeli = p_prog_get_msa_label(&pp, i).to_string();
        for j in i + 1..msa_count {
            assert!((j as usize) < pp.score_mx[i as usize].len());
            let labelj = p_prog_get_msa_label(&pp, j);
            let score = pp.score_mx[i as usize][j as usize];
            out.push_str(&format!("{labeli}\t{labelj}\t{score:.4}\n"));
        }
    }
    std::fs::write(output_file_name, out).expect("failed to write EA MSA distance matrix");
    pp
}
