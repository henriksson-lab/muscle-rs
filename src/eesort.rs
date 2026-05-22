// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Sort database sequences by EA score against query sequences and emit ranked TSV/FASTA outputs.
#[track_caller]
pub fn cmd_eesort<FAlignPairFlat>(
    query_file_name: &str,
    db_file_name: &str,
    output_file_name: &str,
    tsv_out_file_name: &str,
    mut align_pair_flat: FAlignPairFlat,
) -> (Vec<f64>, Vec<uint>, String, String)
where
    FAlignPairFlat: FnMut(&str, &str, &mut String) -> f64,
{
    let mut query = MultiSequence::default();
    let mut db = MultiSequence::default();

    multi_sequence_load_mfa_l8(&mut query, query_file_name, true);
    multi_sequence_load_mfa_l8(&mut db, db_file_name, true);
    let mut global = MultiSequence::default();
    global.seqs.extend(query.seqs.iter().cloned());
    global
        .owners
        .extend(std::iter::repeat(false).take(query.seqs.len()));
    global.seqs.extend(db.seqs.iter().cloned());
    global
        .owners
        .extend(std::iter::repeat(false).take(db.seqs.len()));
    set_global_input_ms(&global);

    let is_nucleo = multi_sequence_guess_is_nucleo(&db);
    if is_nucleo {
        set_alpha_l209(ALPHA::ALPHA_Nucleo);
    } else {
        set_alpha_l209(ALPHA::ALPHA_Amino);
    }
    init_probcons();

    let query_seq_count = query.seqs.len() as uint;
    let db_seq_count = db.seqs.len() as uint;
    let mut eas = vec![f64::MAX; db_seq_count as usize];

    for db_seq_index in 0..db_seq_count {
        let db_label = db.seqs[db_seq_index as usize].label.clone();
        for query_seq_index in 0..query_seq_count {
            let q_label = query.seqs[query_seq_index as usize].label.clone();
            let mut path = String::new();
            let ea = align_pair_flat(&q_label, &db_label, &mut path);
            if query_seq_index == 0 {
                eas[db_seq_index as usize] = ea;
            }
        }
    }

    // C++ uses unstable QuickSortOrderDesc here (eesort.cpp:65); match it
    // so tied EA values land in the same order the C++ binary produces.
    let order = quick_sort_order_desc_by(db_seq_count as usize, |a, b| {
        eas[a]
            .partial_cmp(&eas[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
            let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
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

    let mut tsv_out = String::new();
    let mut fa_out = String::new();
    for db_seq_index in &order {
        let db_seq = &db.seqs[*db_seq_index as usize];
        let ea = eas[*db_seq_index as usize];
        assert_ne!(ea, f64::MAX);
        tsv_out.push_str(&format!("{}\t{}\n", format_g3(ea), db_seq.label));
        fa_out.push_str(&seq_to_fasta_l2561(
            &sequence_get_seq_as_string(db_seq),
            &db_seq.label,
        ));
    }

    if !tsv_out_file_name.is_empty() {
        std::fs::write(tsv_out_file_name, &tsv_out).expect("failed to write eesort TSV");
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &fa_out).expect("failed to write eesort FASTA");
    }
    (eas, order, tsv_out, fa_out)
}
