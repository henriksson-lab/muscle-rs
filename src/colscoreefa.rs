// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Map a confidence value in (0, 1] to a calibration bin 0..=MAXBIN (10 means conf == 1.0).
pub fn get_bin(conf: f64) -> uint {
    assert!(conf > 0.0 && conf <= 1.0);
    if conf == 1.0 {
        return 10;
    }
    let bin = (conf * 10.0) as uint;
    assert!(bin < 10);
    bin
}

/// Score an ensemble FASTA against a reference MSA: compute per-bin accuracy and mean TC.
#[track_caller]
pub fn cmd_colscore_efa(
    efa_file_name: &str,
    ref_file_name: &str,
    output_file_name: &str,
    max_gap_fract: f64,
) -> String {
    const MAXBIN: usize = 10;
    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, efa_file_name);

    let mut ref_msa = msa_from_fasta_file_preserve_case(ref_file_name);
    ensemble_sort_msa(&e, &mut ref_msa);

    let ref_unique_ixs = ensemble_get_ref_unique_ixs(&e, &ref_msa, max_gap_fract);
    let ref_upper_col_count = (0..multi_sequence_get_col_count(&ref_msa))
        .filter(|&ref_col_index| msa_col_is_upper(&ref_msa, ref_col_index, max_gap_fract))
        .count() as uint;
    let ref_pos_set = ensemble_get_ref_pos_set(&e, &ref_msa, max_gap_fract);

    let msa_count = e.msas.len() as uint;
    let mut bin_to_count = vec![0_u32; MAXBIN + 1];
    let mut bin_to_correct_count = vec![0_u32; MAXBIN + 1];
    let mut sum_tc = 0.0;
    for msa_index in 0..msa_count {
        let (test_unique_ixs, confs) = ensemble_get_test_unique_ixs(&e, msa_index, &ref_pos_set);

        let test_ix_count = test_unique_ixs.len();
        let mut correct_count = 0_u32;
        for i in 0..test_ix_count {
            let test_unique_ix = test_unique_ixs[i];
            let conf = confs[i];
            let bin = get_bin(conf) as usize;
            assert!(bin <= MAXBIN);
            bin_to_count[bin] += 1;
            let correct = ref_unique_ixs.contains(&test_unique_ix);
            if correct {
                correct_count += 1;
                bin_to_correct_count[bin] += 1;
            }
        }
        let tc = f64::from(correct_count) / f64::from(ref_upper_col_count);
        sum_tc += tc;
    }
    let mean_tc = sum_tc / f64::from(msa_count);

    let mut out = String::new();
    out.push_str(&format!("meantc\t{mean_tc:.4}\n"));
    for bin in 0..=MAXBIN {
        let count = bin_to_count[bin];
        let correct_count = bin_to_correct_count[bin];
        assert!(correct_count <= count);
        let mut p = 0.0;
        if count > 0 {
            p = f64::from(correct_count) / f64::from(count);
        }
        out.push_str(&format!("bin\t{bin}\t{count}\t{correct_count}\t{p:.4}\n"));
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write colscore_efa output");
    }
    out
}
