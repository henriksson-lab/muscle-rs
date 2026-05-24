// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

fn format_protdists_dist_g4(d: f64) -> String {
    if d == 0.0 {
        return "0".to_string();
    }
    if !d.is_finite() {
        return d.to_string();
    }
    let abs_d = d.abs();
    let exp = abs_d.log10().floor() as i32;
    let mut s = if exp < -4 || exp >= 4 {
        let raw = format!("{d:.3e}");
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
        format!("{d:.decimals$}")
    };
    if !s.contains('e') && !s.contains('E') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

fn protdists_pair_indexes(seq_count: uint) -> Vec<(uint, uint)> {
    let pair_count = (seq_count * (seq_count - 1)) / 2;
    let mut seq_indexi = uint::MAX;
    let mut seq_indexj = uint::MAX;
    let mut pairs = Vec::with_capacity(pair_count as usize);
    for _pair_index in 0..pair_count {
        if seq_indexi == uint::MAX {
            seq_indexi = 1;
            seq_indexj = 0;
        } else {
            seq_indexj += 1;
            if seq_indexj == seq_indexi {
                seq_indexi += 1;
                seq_indexj = 0;
            }
        }
        pairs.push((seq_indexi, seq_indexj));
    }
    pairs
}

fn protdists_setup(input_file_name: &str) -> MultiSequence {
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });
    input_seqs
}

/// `protdists` subcommand: pairwise protein distances for every pair of input
/// sequences, written as `labelI\tlabelJ\tdist\n` lines.
#[track_caller]
pub fn cmd_protdists<FViterbi, FDist>(
    input_file_name: &str,
    output_file_name: &str,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> String
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let input_seqs = protdists_setup(input_file_name);
    let seq_count = input_seqs.seqs.len() as uint;
    let mut out = String::new();
    for (seq_indexi, seq_indexj) in protdists_pair_indexes(seq_count) {
        let seqi_obj = &input_seqs.seqs[seq_indexi as usize];
        let seqi: Vec<byte> = seqi_obj.char_vec.iter().map(|&c| c as byte).collect();
        let li = seqi.len() as uint;
        let labeli = seqi_obj.label.clone();

        let seqj_obj = &input_seqs.seqs[seq_indexj as usize];
        let seqj: Vec<byte> = seqj_obj.char_vec.iter().map(|&c| c as byte).collect();
        let lj = seqj.len() as uint;
        let labelj = seqj_obj.label.clone();

        let dij = get_prot_dist_seq_pair(
            &seqi,
            li,
            &seqj,
            lj,
            None,
            |seqi, li, seqj, lj| viterbi_fast_mem(seqi, li, seqj, lj),
            |row_x, row_y, col_count| get_prot_dist(row_x, row_y, col_count),
        );
        out.push_str(&format!(
            "{labeli}\t{labelj}\t{}\n",
            format_protdists_dist_g4(dij)
        ));
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write protein distances");
    }
    out
}

/// Threaded CLI implementation for `protdists`.
///
/// C++ writes rows from OpenMP workers under a lock, so multi-threaded output
/// order depends on scheduler completion. Rust computes in parallel but merges
/// rows by the C++ serial pair sequence, keeping tests and CLI output stable.
#[track_caller]
pub fn cmd_protdists_threaded(
    input_file_name: &str,
    output_file_name: &str,
    threads: Option<uint>,
) -> String {
    let input_seqs = protdists_setup(input_file_name);
    let seq_count = input_seqs.seqs.len() as uint;
    let pairs = protdists_pair_indexes(seq_count);
    let pair_count = pairs.len() as uint;
    if pair_count == 0 {
        if !output_file_name.is_empty() {
            std::fs::write(output_file_name, "").expect("failed to write protein distances");
        }
        return String::new();
    }

    let thread_count = threads
        .unwrap_or_else(get_requested_thread_count)
        .min(pair_count)
        .max(1);
    let progress_counter = std::sync::Mutex::new(0);
    let mut results = vec![String::new(); pair_count as usize];

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for thread_index in 0..thread_count {
            let start = (pair_count * thread_index) / thread_count;
            let end = (pair_count * (thread_index + 1)) / thread_count;
            let input_seqs = &input_seqs;
            let pairs = &pairs;
            let progress_counter = &progress_counter;
            handles.push(scope.spawn(move || {
                let mut thread_results = Vec::new();
                for pair_index in start..end {
                    {
                        let mut counter = progress_counter.lock().unwrap();
                        let _ = progress_step(*counter, pair_count, "Protdists");
                        *counter += 1;
                    }
                    let (seq_indexi, seq_indexj) = pairs[pair_index as usize];

                    let seqi_obj = &input_seqs.seqs[seq_indexi as usize];
                    let seqi: Vec<byte> = seqi_obj.char_vec.iter().map(|&c| c as byte).collect();
                    let li = seqi.len() as uint;
                    let labeli = seqi_obj.label.clone();

                    let seqj_obj = &input_seqs.seqs[seq_indexj as usize];
                    let seqj: Vec<byte> = seqj_obj.char_vec.iter().map(|&c| c as byte).collect();
                    let lj = seqj.len() as uint;
                    let labelj = seqj_obj.label.clone();

                    let dij = get_prot_dist_seq_pair(
                        &seqi,
                        li,
                        &seqj,
                        lj,
                        None,
                        |seqi, li, seqj, lj| {
                            let mut mem = XDPMem::default();
                            let mut pi = PathInfo::default();
                            viterbi_fast_mem(&mut mem, seqi, li, seqj, lj, &mut pi);
                            pi
                        },
                        |row_x, row_y, col_count| {
                            get_prot_dist_l42(row_x.as_bytes(), row_y.as_bytes(), col_count)
                        },
                    );
                    thread_results.push((
                        pair_index,
                        format!("{labeli}\t{labelj}\t{}\n", format_protdists_dist_g4(dij)),
                    ));
                }
                thread_results
            }));
        }
        for handle in handles {
            for (pair_index, row) in handle.join().unwrap() {
                results[pair_index as usize] = row;
            }
        }
    });

    let out = results.concat();
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write protein distances");
    }
    out
}
