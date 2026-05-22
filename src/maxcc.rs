// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: pick the ensemble MSA with the highest total column confidence.
#[track_caller]
pub fn cmd_maxcc(input_file_name: &str, output_file_name: &str) -> (MultiSequence, uint, f64) {
    if output_file_name.is_empty() {
        panic!("Must set -output");
    }

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, input_file_name);
    let msa_count = e.msas.len() as uint;
    if msa_count == 0 {
        panic!("Ensemble is empty");
    }

    let mut best_msa_index = 0_u32;
    let mut best_conf = 0.0;
    let mut sum_conf = 0.0;
    let mut min_conf = 0.0;
    for msa_index in 0..msa_count {
        let total_conf = ensemble_get_total_conf(&e, msa_index);
        if msa_index == 0 || total_conf < min_conf {
            min_conf = total_conf;
        }
        sum_conf += total_conf;
        if total_conf >= best_conf {
            best_msa_index = msa_index;
            best_conf = total_conf;
        }
    }

    let best_msa = e.msas[best_msa_index as usize].clone();
    let avg_conf = sum_conf / f64::from(msa_count);
    let _ = min_conf;
    msa_to_fasta_file_l103(&best_msa, output_file_name);
    (best_msa, best_msa_index, avg_conf)
}
