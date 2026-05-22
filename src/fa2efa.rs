// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Builds an Ensemble from a list of FASTA MSA paths and writes it out as an EFA file.
#[track_caller]
pub fn cmd_fa2efa(
    input_file_name: &str,
    output_file_name: &str,
    basename: bool,
    intsuffix: bool,
) -> Ensemble {
    let mut e = Ensemble::default();
    ensemble_from_msa_paths(&mut e, input_file_name, basename, intsuffix);
    ensemble_to_efa(&e, output_file_name);
    e
}
