// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

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
    let mut input_seqs = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&input_seqs);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let format_g4 = |d: f64| -> String {
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
    };

    let seq_count = input_seqs.seqs.len() as uint;
    let pair_count = (seq_count * (seq_count - 1)) / 2;
    let mut seq_indexi = uint::MAX;
    let mut seq_indexj = uint::MAX;
    let mut out = String::new();
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
        out.push_str(&format!("{labeli}\t{labelj}\t{}\n", format_g4(dij)));
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write protein distances");
    }
    out
}
