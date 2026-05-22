// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `qscore_efa` subcommand: score every MSA in an EFA ensemble against a
/// reference and return one Q/TC line per ensemble member.
#[track_caller]
pub fn cmd_qscore_efa(efa_file_name: &str, ref_file_name: &str, max_gap_fract: f64) -> String {
    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, efa_file_name);

    let ref_msa = msa_from_fasta_file_preserve_case(ref_file_name);
    let ref_name = get_base_name(ref_file_name);

    let mut qs = QScorer {
        max_gap_fract,
        ..QScorer::default()
    };

    let msa_count = e.msas.len() as uint;
    let mut out = String::new();
    for msa_index in 0..msa_count {
        let test_msa = ensemble_get_msa(&e, msa_index);
        let test_name = ensemble_get_msa_name(&e, msa_index);
        q_scorer_run_l337(&mut qs, test_name, test_msa, &ref_msa);
        out.push_str(&format!(
            "{} {} Q={:.4} TC={:.4}\n",
            ref_name, test_name, qs.q, qs.tc
        ));
    }
    out
}
