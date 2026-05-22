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
    ap: &M3AlnParams,
    mut run: FRun,
    mut run_ro: FRunRo,
) -> (MultiSequence, Option<Tree>)
where
    FRun: FnMut(&M3AlnParams, &MultiSequence) -> (MultiSequence, Tree),
    FRunRo: FnMut(&M3AlnParams, &MultiSequence) -> MultiSequence,
{
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let (final_msa, guide_tree) = if random_order {
        (run_ro(ap, &input_seqs), None)
    } else {
        let (msa, tree) = run(ap, &input_seqs);
        if let Some(file_name) = guide_tree_out_file_name {
            tree_to_file_l13(&tree, file_name);
        }
        (msa, Some(tree))
    };
    multi_sequence_write_mfa(&final_msa, output_file_name);
    (final_msa, guide_tree)
}
