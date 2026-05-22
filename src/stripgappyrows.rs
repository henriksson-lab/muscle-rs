// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Drop rows whose gap fraction exceeds `max_gap_fract` and write the
/// surviving sequences to `output_file_name`; returns the number of dropped rows.
#[track_caller]
pub fn cmd_strip_gappy_rows(
    input_file_name: &str,
    output_file_name: &str,
    max_gap_fract: f64,
) -> uint {
    let aln = msa_from_fasta_file_l95(input_file_name);
    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);
    let mut discard_count = 0_u32;
    let mut out = String::new();
    for seq in &aln.seqs {
        let mut gap_count = 0_u32;
        for &c in &seq.char_vec {
            if c == '-' || c == '.' {
                gap_count += 1;
            }
        }
        let gap_fract = f64::from(gap_count) / f64::from(col_count);
        if gap_fract > max_gap_fract {
            discard_count += 1;
            continue;
        }
        let seq_string = sequence_get_seq_as_string(seq);
        out.push_str(&seq_to_fasta_l2561(&seq_string, &seq.label));
    }
    assert!(discard_count <= seq_count);
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write strip_gappy_rows output");
    }
    discard_count
}
