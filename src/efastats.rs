// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Scores every MSA in the ensemble against a reference, returning Q and TC vectors.
#[track_caller]
pub fn cmp_ref(e: &Ensemble, ref_msa: &MultiSequence, max_gap_fract: f64) -> (Vec<f64>, Vec<f64>) {
    let mut qs_vec = Vec::<f64>::new();
    let mut tcs = Vec::<f64>::new();

    let mut qs = QScorer {
        max_gap_fract,
        ..QScorer::default()
    };
    let msa_count = e.msas.len() as uint;
    for msa_index in 0..msa_count {
        let test_msa = ensemble_get_msa(e, msa_index);
        let test_name = ensemble_get_msa_name(e, msa_index);
        q_scorer_run_l337(&mut qs, test_name, test_msa, ref_msa);
        qs_vec.push(qs.q as f64);
        tcs.push(qs.tc as f64);
    }
    (qs_vec, tcs)
}

/// Reports ensemble statistics (cols, N1, conf, CC, dispersion) and optional Q/TC vs reference.
#[track_caller]
pub fn cmd_efastats(
    input_file_name: &str,
    max_gap_fract: f64,
    ref_file_name: Option<&str>,
) -> String {
    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, input_file_name);

    let mut qs_vec = Vec::<f64>::new();
    let mut tcs = Vec::<f64>::new();
    if let Some(ref_file_name) = ref_file_name {
        let ref_msa = msa_from_fasta_file_l95(ref_file_name);
        (qs_vec, tcs) = cmp_ref(&e, &ref_msa, max_gap_fract);
    }

    let seq_count = ensemble_get_seq_count(&e);
    let msa_count = e.msas.len() as uint;
    let ix_count = e.ix_to_msa_index.len() as uint;

    let (d_letter_pairs, d_columns) = ensemble_get_dispersion(&e, max_gap_fract);

    let mut ccs = Vec::<f64>::new();
    let avg_cols = f64::from(ix_count) / f64::from(msa_count);

    let mut out = String::new();
    out.push_str("  MSA     Cols     N1   N1f  Conf     CC");
    if ref_file_name.is_some() {
        out.push_str("       Q      TC");
    }
    out.push_str("  Name\n");
    for msa_index in 0..msa_count {
        let m = &e.msas[msa_index as usize];
        let name = &e.msa_names[msa_index as usize];
        let n1 = ensemble_get_n1(&e, msa_index);
        let col_count = multi_sequence_get_col_count(m);
        let total_conf = ensemble_get_total_conf(&e, msa_index);
        let n1f = f64::from(n1) / f64::from(col_count);
        let cc = total_conf / f64::from(col_count);
        ccs.push(cc);
        out.push_str(&format!(
            "{:5}  {:7}  {:5}  {:4.2}  {:4.2}  {:5.3}",
            msa_index + 1,
            col_count,
            n1,
            n1f,
            total_conf,
            cc
        ));
        if ref_file_name.is_some() {
            out.push_str(&format!(
                "  {:6.4}  {:6.4}",
                qs_vec[msa_index as usize], tcs[msa_index as usize]
            ));
        }
        out.push_str(&format!("  {name}\n"));
    }

    ccs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_cc = ccs[msa_count as usize / 2];

    out.push_str(&format!(
        "{seq_count} seqs, {msa_count} MSAs, avg cols {avg_cols:.1}, D_LP {}, D_Cols {}, CC {}",
        format_g3(d_letter_pairs),
        format_g3(d_columns),
        format_g3(median_cc)
    ));
    if ref_file_name.is_some() {
        qs_vec.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        tcs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        assert_eq!(qs_vec.len(), msa_count as usize);
        assert_eq!(tcs.len(), msa_count as usize);
        let median_q = qs_vec[msa_count as usize / 2];
        let median_tc = tcs[msa_count as usize / 2];
        let e_lp = 1.0 - median_q;
        let e_cols = 1.0 - median_tc;
        out.push_str(&format!(" E_LP {e_lp:.4}, E_Cols {e_cols:.4}"));
    }
    out.push('\n');
    out
}
