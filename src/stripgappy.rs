// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `strip_gappy` command: drop columns and rows whose gap fraction exceeds the given thresholds.
#[track_caller]
pub fn cmd_strip_gappy(
    input_file_name: &str,
    output_file_name: &str,
    max_gap_fract: f64,
    max_gap_fract_row: f64,
) -> (uint, uint) {
    let aln = msa_from_fasta_file_l95(input_file_name);
    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);

    let mut keep_cols = Vec::<uint>::new();
    let mut discard_col_count = 0_u32;
    for col in 0..col_count {
        let gap_count = msa_get_gap_count(&aln, col);
        let gap_fract = f64::from(gap_count) / f64::from(seq_count);
        if gap_fract <= max_gap_fract {
            keep_cols.push(col);
        } else {
            discard_col_count += 1;
        }
    }

    let new_col_count = keep_cols.len() as uint;
    assert!(new_col_count > 0);

    let mut discard_row_count = 0_u32;
    let mut out = String::new();
    for seq_index in 0..seq_count {
        let label = msa_get_seq_name(&aln, seq_index);
        let mut row_gap_count = 0_u32;
        let mut new_seq = String::new();
        for &col in &keep_cols {
            let c = msa_get_char(&aln, seq_index, col);
            new_seq.push(c);
            if c == '-' {
                row_gap_count += 1;
            }
        }
        let gap_fract_row = f64::from(row_gap_count) / f64::from(new_col_count);
        if gap_fract_row > max_gap_fract_row {
            discard_row_count += 1;
            continue;
        }
        out.push_str(&seq_to_fasta_l2561(&new_seq, &label));
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write strip_gappy output");
    }
    (discard_col_count, discard_row_count)
}
