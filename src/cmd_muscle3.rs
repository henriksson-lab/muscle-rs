// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Top-level MUSCLE3 driver: loads input FASTA, picks alphabet, dispatches to
/// either random-order or tree-based alignment, optionally writes guide tree.
#[track_caller]
pub fn cmd_muscle3<FRun, FRunRo>(
    input_file_name: &str,
    output_file_name: &str,
    guide_tree_out_file_name: Option<&str>,
    random_order: bool,
    mut set_params: impl FnMut(&mut M3AlnParams, bool),
    mut run: FRun,
    mut run_ro: FRunRo,
) -> (MultiSequence, Option<Tree>)
where
    FRun: FnMut(&mut Muscle3, &M3AlnParams, &MultiSequence) -> MultiSequence,
    FRunRo: FnMut(&mut Muscle3, &M3AlnParams, &MultiSequence) -> MultiSequence,
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, false);
    set_global_input_ms(&input_seqs);
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let mut ap = M3AlnParams::default();
    set_params(&mut ap, is_nucleo);

    let mut m3 = Muscle3 {
        ap_addr: Some(&ap as *const M3AlnParams as usize),
        ap: Some(ap.clone()),
        ..Muscle3::default()
    };
    let (final_msa, guide_tree) = if random_order {
        (run_ro(&mut m3, &ap, &input_seqs), None)
    } else {
        let msa = run(&mut m3, &ap, &input_seqs);
        if let Some(file_name) = guide_tree_out_file_name {
            tree_to_file_l13(&m3.guide_tree, file_name);
        }
        (msa, Some(m3.guide_tree.clone()))
    };
    if m3.final_msa.is_none() {
        m3.final_msa = Some(final_msa.clone());
        m3.final_msa_addr = Some(m3.final_msa.as_ref().unwrap() as *const MultiSequence as usize);
    }
    muscle3_write_msa(&m3, output_file_name);
    (final_msa, guide_tree)
}
