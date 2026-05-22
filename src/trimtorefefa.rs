// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// CLI entry: apply `trim_to_ref` to each MSA in an EFA ensemble and write the result.
#[track_caller]
pub fn cmd_trimtoref_efa(
    efa_file_name: &str,
    ref_file_name: &str,
    output_file_name: &str,
) -> Ensemble {
    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, efa_file_name);

    let ref_msa = msa_from_fasta_file_preserve_case(ref_file_name);

    let msa_count = e.msas.len() as uint;
    let mut out = String::new();
    let mut trimmed_ensemble = Ensemble::default();
    for msa_index in 0..msa_count {
        let msa_name = ensemble_get_msa_name(&e, msa_index);
        out.push('<');
        out.push_str(msa_name);
        out.push('\n');

        let test_msa = ensemble_get_msa(&e, msa_index);
        let trimmed_msa = trim_to_ref(test_msa, &ref_msa);
        out.push_str(&msa_to_fasta_file_l112(&trimmed_msa));
        trimmed_ensemble.msa_names.push(msa_name.to_string());
        trimmed_ensemble.msas.push(trimmed_msa);
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write trimtoref_efa output");
    }
    trimmed_ensemble
}
