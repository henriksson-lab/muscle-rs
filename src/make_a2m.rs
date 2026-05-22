// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: convert a FASTA MSA to A2M format using a gap fraction threshold.
#[track_caller]
pub fn cmd_make_a2m(
    input_file_name: &str,
    output_file_name: &str,
    max_gap_fract: f64,
    fasta_allow_digits: bool,
) -> String {
    let aln = msa_from_fasta_file_l95(input_file_name);

    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);
    assert!(seq_count > 0);
    assert!(col_count > 0);

    let mut match_vec = Vec::<bool>::new();
    let mut match_count = 0_u32;
    for col in 0..col_count {
        let gap_count = msa_get_gap_count(&aln, col);
        let gap_fract = f64::from(gap_count) / f64::from(seq_count);
        let is_match = gap_fract <= max_gap_fract;
        match_vec.push(is_match);
        if is_match {
            match_count += 1;
        }
    }

    assert!(match_count > 0);
    assert_eq!(match_vec.len(), col_count as usize);

    let mut out = String::new();
    for seq_index in 0..seq_count {
        out.push_str(&format!(">{}\n", msa_get_seq_name(&aln, seq_index)));

        let seq_char_ptr = msa_get_seq_char_ptr(&aln, seq_index);
        let mut seq_str = String::new();
        for col in 0..col_count {
            let mut c = seq_char_ptr[col as usize];

            let is_match = match_vec[col as usize];
            if is_match {
                if c == '.' || c == '-' {
                    c = '-';
                } else if c.is_ascii_alphabetic() {
                    c = c.to_ascii_uppercase();
                } else if fasta_allow_digits && c.is_ascii_digit() {
                } else {
                    panic!("Bad char 0x{:02x}", c as u32);
                }
            } else {
                assert!(!is_match);
                if c == '.' || c == '-' {
                    continue;
                }
                c = c.to_ascii_lowercase();
            }
            seq_str.push(c);
        }

        out.push_str(&seq_str);
        out.push('\n');
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write A2M output");
    }
    out
}
