// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

fn format_eadistmx_g4(d: f32) -> String {
    if d == 0.0 {
        return "0".to_string();
    }
    if !d.is_finite() {
        return d.to_string();
    }
    let d64 = f64::from(d);
    let abs_d = d64.abs();
    let exp = abs_d.log10().floor() as i32;
    let mut s = if exp < -4 || exp >= 4 {
        let raw = format!("{d64:.3e}");
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
        let decimals = (3 - exp).max(0) as usize;
        format!("{d64:.decimals$}")
    };
    if !s.contains('e') && !s.contains('E') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// Compute the full EA-distance matrix for all sequence pairs (via `align_pair`); also returns labeled TSV rows.
#[track_caller]
pub fn calc_ea_dist_mx<FAlignPair>(
    sequences: &MultiSequence,
    mut sparse_post_vec: Option<&mut Vec<MySparseMx>>,
    mut align_pair: FAlignPair,
) -> (Vec<Vec<f32>>, String)
where
    FAlignPair: FnMut(&str, &str, &mut String) -> (f32, Option<MySparseMx>),
{
    let seq_count = sequences.seqs.len() as uint;
    let mut dist_mx = vec![vec![0.0; seq_count as usize]; seq_count as usize];
    for i in 0..seq_count {
        dist_mx[i as usize][i as usize] = 1.0;
    }
    if let Some(vec) = sparse_post_vec.as_ref() {
        assert!(vec.is_empty());
    }

    let (seq_indexes1, seq_indexes2) = get_all_pairs_l3(seq_count);
    let pair_count = seq_indexes1.len() as uint;
    assert_eq!(seq_indexes2.len() as uint, pair_count);
    let pair_count2 = (seq_count * (seq_count - 1)) / 2;
    assert_eq!(pair_count, pair_count2);

    let mut sum_ea = 0.0_f32;
    let mut rows = String::new();
    for pair_index in 0..pair_count {
        let seq_index1 = seq_indexes1[pair_index as usize];
        let seq_index2 = seq_indexes2[pair_index as usize];
        let seq1 = &sequences.seqs[seq_index1 as usize];
        let seq2 = &sequences.seqs[seq_index2 as usize];
        let label1 = &seq1.label;
        let label2 = &seq2.label;

        let _mean_ea = if pair_index == 0 {
            0.0
        } else {
            sum_ea / pair_index as f32
        };

        let mut path = String::new();
        let (ea, sparse_post) = align_pair(label1, label2, &mut path);
        if let Some(vec) = sparse_post_vec.as_deref_mut() {
            vec.push(sparse_post.expect("CalcEADistMx sparse post expected"));
        }

        dist_mx[seq_index1 as usize][seq_index2 as usize] = ea;
        dist_mx[seq_index2 as usize][seq_index1 as usize] = ea;
        rows.push_str(&format!("{label1}\t{label2}\t{}\n", format_eadistmx_g4(ea)));
        sum_ea += ea;
    }
    (dist_mx, rows)
}

/// C++-literal test sidecar for `CalcEADistMx`.
///
/// The normal Rust helper preserves `GetAllPairs` order. This helper replays an
/// explicit OpenMP-visible pair completion order for progress, row writes,
/// matrix commits, and sparse-posterior append order.
#[track_caller]
pub fn calc_ea_dist_mx_cpp_literal_with_pair_schedule<FAlignPair>(
    sequences: &MultiSequence,
    mut sparse_post_vec: Option<&mut Vec<MySparseMx>>,
    write_rows: bool,
    pair_schedule: &[uint],
    mut align_pair: FAlignPair,
) -> (Vec<Vec<f32>>, String)
where
    FAlignPair: FnMut(&str, &str, &mut String) -> (f32, Option<MySparseMx>),
{
    let seq_count = sequences.seqs.len() as uint;
    let mut dist_mx = vec![vec![0.0; seq_count as usize]; seq_count as usize];
    for i in 0..seq_count {
        dist_mx[i as usize][i as usize] = 1.0;
    }
    if let Some(vec) = sparse_post_vec.as_ref() {
        assert!(vec.is_empty());
    }

    let (seq_indexes1, seq_indexes2) = get_all_pairs_l3(seq_count);
    let pair_count = seq_indexes1.len() as uint;
    assert_eq!(seq_indexes2.len() as uint, pair_count);
    assert_eq!(pair_count, (seq_count * (seq_count - 1)) / 2);

    let mut pair_done = vec![false; pair_count as usize];
    let mut pair_counter = 0_u32;
    let mut sum_ea = 0.0_f32;
    let mut rows = String::new();

    let mut run_pair = |pair_index: uint,
                        pair_done: &mut [bool],
                        pair_counter: &mut uint,
                        sum_ea: &mut f32,
                        rows: &mut String| {
        if pair_index >= pair_count || pair_done[pair_index as usize] {
            return;
        }
        pair_done[pair_index as usize] = true;
        let seq_index1 = seq_indexes1[pair_index as usize];
        let seq_index2 = seq_indexes2[pair_index as usize];
        let seq1 = &sequences.seqs[seq_index1 as usize];
        let seq2 = &sequences.seqs[seq_index2 as usize];
        let label1 = &seq1.label;
        let label2 = &seq2.label;

        let mean_ea = if *pair_counter == 0 {
            0.0
        } else {
            *sum_ea / *pair_counter as f32
        };
        let _ = progress_step(
            *pair_counter,
            pair_count,
            &format!("{seq_count} consensus seqs, mean EE {:.2}", 1.0 - mean_ea),
        );
        *pair_counter += 1;

        let mut path = String::new();
        let (ea, sparse_post) = align_pair(label1, label2, &mut path);
        if let Some(vec) = sparse_post_vec.as_deref_mut() {
            vec.push(sparse_post.expect("CalcEADistMx sparse post expected"));
        }

        dist_mx[seq_index1 as usize][seq_index2 as usize] = ea;
        dist_mx[seq_index2 as usize][seq_index1 as usize] = ea;
        if write_rows {
            rows.push_str(&format!("{label1}\t{label2}\t{}\n", format_eadistmx_g4(ea)));
        }
        *sum_ea += ea;
    };

    for &pair_index in pair_schedule {
        run_pair(
            pair_index,
            &mut pair_done,
            &mut pair_counter,
            &mut sum_ea,
            &mut rows,
        );
    }
    for pair_index in 0..pair_count {
        run_pair(
            pair_index,
            &mut pair_done,
            &mut pair_counter,
            &mut sum_ea,
            &mut rows,
        );
    }

    (dist_mx, rows)
}

/// Driver: load sequences, set up alphabet/ProbCons, compute the EA distance matrix, and write it to disk.
#[track_caller]
pub fn cmd_eadistmx<FAlignPair>(
    input_file_name: &str,
    output_file_name: &str,
    mut align_pair: FAlignPair,
) -> Vec<Vec<f32>>
where
    FAlignPair: FnMut(&str, &str, &mut String) -> (f32, Option<MySparseMx>),
{
    assert!(!output_file_name.is_empty());
    let mut sequences = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut sequences, input_file_name, true);
    let _ = progress_log_input_summary(input_file_name, &sequences);
    set_global_input_ms(&sequences);
    let is_nucleo = multi_sequence_guess_is_nucleo(&sequences);
    if is_nucleo {
        set_alpha_l209(ALPHA::ALPHA_Nucleo);
    } else {
        set_alpha_l209(ALPHA::ALPHA_Amino);
    }
    init_probcons();

    let (dist_mx, rows) = calc_ea_dist_mx(&sequences, None, |label1, label2, path| {
        align_pair(label1, label2, path)
    });
    std::fs::write(output_file_name, rows).expect("failed to write EA distance matrix");
    dist_mx
}

/// C++-literal test sidecar for `cmd_eadistmx`.
///
/// Preserves the command body's original setup shape and delegates the
/// scheduler-visible pair order to `calc_ea_dist_mx_cpp_literal_with_pair_schedule`.
#[track_caller]
pub fn cmd_eadistmx_cpp_literal_with_pair_schedule<FAlignPair>(
    input_file_name: &str,
    output_file_name: &str,
    pair_schedule: &[uint],
    mut align_pair: FAlignPair,
) -> Vec<Vec<f32>>
where
    FAlignPair: FnMut(&str, &str, &mut String) -> (f32, Option<MySparseMx>),
{
    assert!(!output_file_name.is_empty());
    let mut sequences = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut sequences, input_file_name, true);
    let _ = progress_log_input_summary(input_file_name, &sequences);
    init_probcons();

    let (dist_mx, rows) = calc_ea_dist_mx_cpp_literal_with_pair_schedule(
        &sequences,
        None,
        true,
        pair_schedule,
        |label1, label2, path| align_pair(label1, label2, path),
    );
    std::fs::write(output_file_name, rows).expect("failed to write EA distance matrix");
    dist_mx
}
