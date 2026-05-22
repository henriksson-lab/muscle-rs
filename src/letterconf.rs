// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Computes per-letter confidence values for each reference position across the ensemble.
#[track_caller]
pub fn ensemble_get_letter_confs_vec(
    e: &Ensemble,
    ref_msa: &MultiSequence,
    max_gap_fract: f64,
) -> Vec<Vec<f64>> {
    let _ = max_gap_fract;
    let mut qs = QScorer::default();

    let mut letter_counts_vec = Vec::<Vec<uint>>::new();
    let test_msa_count = e.msas.len() as uint;
    for test_msa_index in 0..test_msa_count {
        let test = ensemble_get_msa(e, test_msa_index);
        q_scorer_run_l337(&mut qs, "Ensemble::GetLetterConfsVec()", test, ref_msa);
        q_scorer_update_ref_letter_counts(&qs, &mut letter_counts_vec);
    }

    let ref_seq_count = qs.ref_msa.as_ref().expect("QScorer ref not set").seqs.len();
    let ref_col_count =
        multi_sequence_get_col_count(qs.ref_msa.as_ref().expect("QScorer ref not set"));
    assert_eq!(letter_counts_vec.len(), ref_seq_count);
    assert_eq!(letter_counts_vec[0].len(), ref_col_count as usize);

    let mut letter_confs_vec = vec![Vec::<f64>::new(); ref_seq_count];
    for ref_seq_index in 0..ref_seq_count {
        for ref_col_index in 0..ref_col_count {
            let n = letter_counts_vec[ref_seq_index][ref_col_index as usize];
            let c = msa_get_char(ref_msa, ref_seq_index as uint, ref_col_index);
            let mut letter_conf = 0.0;
            if c == '-' || c == '.' {
                assert_eq!(n, 0);
            } else {
                letter_conf = f64::from(n) / f64::from(test_msa_count);
            }
            letter_confs_vec[ref_seq_index].push(letter_conf);
        }
    }
    letter_confs_vec
}

/// Implements `cmd_letterconf`: emits a confidence-coded MSA plus HTML/JalView views.
#[track_caller]
pub fn cmd_letterconf(
    ensemble_file_name: &str,
    ref_file_name: &str,
    output_file_name: &str,
    html_file_name: &str,
    jalview_file_name: &str,
    max_gap_fract: f64,
) -> MultiSequence {
    let ref_msa = msa_from_fasta_file_l95(ref_file_name);
    let ref_pc = msa_from_fasta_file_preserve_case(ref_file_name);
    let ref_seq_count = ref_msa.seqs.len() as uint;
    let ref_col_count = multi_sequence_get_col_count(&ref_msa);

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, ensemble_file_name);

    let letter_confs_vec = ensemble_get_letter_confs_vec(&e, &ref_msa, max_gap_fract);
    assert_eq!(letter_confs_vec.len(), ref_seq_count as usize);
    assert_eq!(letter_confs_vec[0].len(), ref_col_count as usize);

    let mut conf_aln = msa_copy(&ref_msa);

    for ref_seq_index in 0..ref_seq_count {
        for ref_col_index in 0..ref_col_count {
            let c = msa_get_char(&conf_aln, ref_seq_index, ref_col_index);
            if c == '-' || c == '.' {
                continue;
            }
            let conf = letter_confs_vec[ref_seq_index as usize][ref_col_index as usize];
            let conf_char = conf_to_char_1(conf);
            msa_set_char(&mut conf_aln, ref_seq_index, ref_col_index, conf_char);
        }
    }
    msa_to_fasta_file_l103(&conf_aln, output_file_name);
    write_letter_conf_html(html_file_name, &ref_pc, &conf_aln);
    write_letter_conf_jal_view(jalview_file_name, &ref_pc, &conf_aln);
    conf_aln
}

/// Implements `cmd_addletterconfseq`: prepends a per-column mean-confidence row to the MSA.
#[track_caller]
pub fn cmd_addletterconfseq(
    ensemble_file_name: &str,
    ref_file_name: &str,
    output_file_name: &str,
    max_gap_fract: f64,
) -> String {
    let ref_msa = msa_from_fasta_file_l95(ref_file_name);
    let ref_pc = msa_from_fasta_file_preserve_case(ref_file_name);
    let ref_seq_count = ref_msa.seqs.len() as uint;
    let ref_col_count = multi_sequence_get_col_count(&ref_msa);

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, ensemble_file_name);

    let letter_confs_vec = ensemble_get_letter_confs_vec(&e, &ref_msa, max_gap_fract);
    assert_eq!(letter_confs_vec.len(), ref_seq_count as usize);
    assert_eq!(letter_confs_vec[0].len(), ref_col_count as usize);

    let mut conf_row = String::new();
    for ref_col_index in 0..ref_col_count {
        let mut sum_conf = 0.0;
        for ref_seq_index in 0..ref_seq_count {
            sum_conf += letter_confs_vec[ref_seq_index as usize][ref_col_index as usize];
        }
        let mean_conf = sum_conf / f64::from(ref_seq_count);
        let conf_char = conf_to_char_1(mean_conf);
        conf_row.push(conf_char);
    }

    let mut out = seq_to_fasta_l2561(&conf_row, "_letterconf_");
    out.push_str(&msa_to_fasta_file_l112(&ref_pc));
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write addletterconfseq output");
    }
    out
}
