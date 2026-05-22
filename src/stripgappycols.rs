// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Drop columns whose gap fraction exceeds `max_gap_fract` and write the
/// trimmed MSA to `output_file_name`; returns the number of removed columns.
#[track_caller]
pub fn cmd_strip_gappy_cols(
    input_file_name: &str,
    output_file_name: &str,
    max_gap_fract: f64,
) -> uint {
    let aln = msa_from_fasta_file_l95(input_file_name);
    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);

    let mut keep_cols = Vec::<uint>::new();
    let mut gappy_count = 0_u32;
    for col in 0..col_count {
        let gap_count = msa_get_gap_count(&aln, col);
        let gap_fract = f64::from(gap_count) / f64::from(seq_count);
        if gap_fract <= max_gap_fract {
            keep_cols.push(col);
        } else {
            gappy_count += 1;
        }
    }

    let new_col_count = keep_cols.len() as uint;
    assert!(new_col_count > 0);

    let mut out = String::new();
    for seq_index in 0..seq_count {
        let label = msa_get_seq_name(&aln, seq_index);
        let mut new_seq = String::new();
        for &col in &keep_cols {
            new_seq.push(msa_get_char(&aln, seq_index, col));
        }
        out.push_str(&seq_to_fasta_l2561(&new_seq, &label));
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write strip_gappy_cols output");
    }
    gappy_count
}
