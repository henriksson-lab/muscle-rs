// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Explode an ensemble FASTA into one FASTA file per MSA, named with optional prefix/suffix.
#[track_caller]
pub fn cmd_efa_explode(
    input_file_name: &str,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> Vec<String> {
    let prefix = prefix.unwrap_or("");
    let suffix = suffix.unwrap_or("");

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, input_file_name);

    let msa_count = e.msas.len() as uint;
    let mut file_names = Vec::new();
    for msa_index in 0..msa_count {
        let m = ensemble_get_msa(&e, msa_index);
        let mut file_name = ensemble_get_msa_name(&e, msa_index).to_string();
        if file_name.is_empty() {
            file_name = format!("{msa_index}");
        }
        file_name = format!("{prefix}{file_name}{suffix}");
        msa_to_fasta_file_l103(m, &file_name);
        file_names.push(file_name);
    }
    file_names
}
