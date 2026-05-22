// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Select the highest-confidence MSA from an ensemble (by total and median per-column confidence) and emit the median pick as FASTA.
#[track_caller]
pub fn cmd_efa_bestconf(file_name: &str, output_file_name: &str) -> (String, uint, uint) {
    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, file_name);

    let seq_count = ensemble_get_seq_count(&e);
    let msa_count = e.msas.len() as uint;
    let ix_count = e.ix_to_msa_index.len() as uint;
    let avg_cols = f64::from(ix_count) / f64::from(msa_count);

    let mut out = String::new();
    out.push_str(&format!(
        "{seq_count} seqs, {msa_count} MSAs, avg cols {avg_cols:.1}\n"
    ));
    out.push_str("  MSA     Cols     N1   N1f  TotConf  MedConf  Name\n");

    let mut best_msa_index_total = 0_u32;
    let mut best_msa_index_median = 0_u32;
    let mut best_msa_name_total = String::new();
    let mut best_msa_name_median = String::new();
    let mut best_conf_total = -1.0_f64;
    let mut best_conf_median = -1.0_f64;
    for msa_index in 0..msa_count {
        let m = &e.msas[msa_index as usize];
        let name = &e.msa_names[msa_index as usize];
        let n1 = ensemble_get_n1(&e, msa_index);
        let col_count = multi_sequence_get_col_count(m);
        let total_conf = ensemble_get_total_conf(&e, msa_index);
        let median_conf = ensemble_get_median_conf(&e, msa_index);
        if total_conf > best_conf_total {
            best_conf_total = total_conf;
            best_msa_index_total = msa_index;
            best_msa_name_total = name.clone();
        }
        if median_conf > best_conf_median {
            best_conf_median = median_conf;
            best_msa_index_median = msa_index;
            best_msa_name_median = name.clone();
        }
        let n1f = f64::from(n1) / f64::from(col_count);
        out.push_str(&format!(
            "{:5}  {:7}  {:5}  {:4.2}  {:7.3}  {:7.4}  {}\n",
            msa_index + 1,
            col_count,
            n1,
            n1f,
            total_conf,
            median_conf,
            name
        ));
    }

    out.push_str(&format!(
        "Best MSA, total  {} ({})\n",
        best_msa_index_total + 1,
        best_msa_name_total
    ));
    out.push_str(&format!(
        "Best MSA, median {} ({})\n",
        best_msa_index_median + 1,
        best_msa_name_median
    ));

    msa_to_fasta_file_l103(&e.msas[best_msa_index_median as usize], output_file_name);
    (out, best_msa_index_total, best_msa_index_median)
}
