// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Extracts MUSTANG core columns: upper-cases fully-aligned columns, lower-
/// cases inserts, squeezes out the inserts, and emits 80-wide FASTA output.
#[track_caller]
pub fn cmd_mustang_core(input_file_name: &str, output_file_name: &str) -> String {
    const MINCORECOLS: uint = 10;

    let input_msa = msa_from_fasta_file_preserve_case(input_file_name);
    let seq_count = input_msa.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&input_msa);

    let mut aligned_col_count = 0;
    let mut labels = Vec::new();
    for seq in &input_msa.seqs {
        labels.push(seq.label.clone());
    }
    let mut tmp_rows = vec![String::new(); seq_count as usize];
    for col in 0..col_count {
        let aligned = msa_get_gap_count(&input_msa, col) == 0;
        if aligned {
            aligned_col_count += 1;
        }
        for seq_index in 0..seq_count {
            let mut c = msa_get_char(&input_msa, seq_index, col);
            if aligned {
                if c == '-' || c == '.' {
                    c = '-';
                } else if c.is_ascii_alphabetic() {
                    c = c.to_ascii_uppercase();
                }
            } else if c == '-' || c == '.' {
                c = '.';
            } else if c.is_ascii_alphabetic() {
                c = c.to_ascii_lowercase();
            }
            tmp_rows[seq_index as usize].push(c);
        }
    }

    let mut tmp_msa = MultiSequence::default();
    multi_sequence_from_strings(&mut tmp_msa, &labels, &tmp_rows);
    let out_msa = squeeze_inserts(&tmp_msa);
    assert_eq!(out_msa.seqs.len() as uint, seq_count);
    let new_col_count = multi_sequence_get_col_count(&out_msa) as usize;

    if aligned_col_count < MINCORECOLS {
        return warning(&format!("{aligned_col_count} aligned cols < {MINCORECOLS}"));
    }

    let mut out = String::new();
    for seq in &out_msa.seqs {
        let mut label = seq.label.clone();
        if label.ends_with(".pdb") {
            label.truncate(label.len() - 4);
        }
        out.push('>');
        out.push_str(&label);
        out.push('\n');
        let mut n = 0usize;
        while n < new_col_count {
            let end = std::cmp::min(n + 80, new_col_count);
            for col in n..end {
                out.push(seq.char_vec[col]);
            }
            out.push('\n');
            n = end;
        }
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write MUSTANG core output");
    }
    out
}
