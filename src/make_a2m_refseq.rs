// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: convert an MSA to A2M with match columns defined by a reference sequence.
#[track_caller]
pub fn cmd_make_a2m_refseq(
    input_file_name: &str,
    output_file_name: &str,
    label: Option<&str>,
    fasta_allow_digits: bool,
) {
    let mut aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut aln, input_file_name, false);

    let seq_count = aln.seqs.len() as uint;
    assert!(seq_count > 0);
    assert!(multi_sequence_is_aligned(&aln));
    let col_count = multi_sequence_get_col_count(&aln);
    assert!(col_count > 0);

    let ref_seq_index = if let Some(ref_label) = label {
        multi_sequence_get_seq_index(&aln, ref_label, true)
    } else {
        0
    };

    let ref_seq = &aln.seqs[ref_seq_index as usize];
    let mut ref_pos_to_col = Vec::new();
    let mut ref_col_to_pos = Vec::with_capacity(col_count as usize);
    let mut pos = 0;
    for col in 0..col_count {
        let c = ref_seq.char_vec[col as usize];
        if c == '-' || c == '.' {
            ref_col_to_pos.push(uint::MAX);
        } else {
            ref_pos_to_col.push(col);
            ref_col_to_pos.push(pos);
            pos += 1;
        }
    }

    let rl = ref_pos_to_col.len() as uint;
    assert!(rl > 0);
    let mut is_inserts = vec![false; col_count as usize];

    let first_col = ref_pos_to_col[0];
    for col in 0..first_col {
        is_inserts[col as usize] = true;
    }

    for ref_pos in 1..rl {
        let prev_col = ref_pos_to_col[(ref_pos - 1) as usize];
        let this_col = ref_pos_to_col[ref_pos as usize];
        assert!(prev_col < this_col);
        for col in (prev_col + 1)..this_col {
            is_inserts[col as usize] = true;
        }
    }

    let last_col = ref_pos_to_col[(rl - 1) as usize];
    for col in (last_col + 1)..col_count {
        is_inserts[col as usize] = true;
    }

    for col in 0..col_count {
        let is_match = ref_col_to_pos[col as usize] != uint::MAX;
        let is_insert = is_inserts[col as usize];
        assert_eq!(is_match as uint + is_insert as uint, 1);
    }

    let mut out = String::new();
    for i in 0..seq_count {
        let seq_index = if i == 0 {
            ref_seq_index
        } else if i == ref_seq_index {
            0
        } else {
            i
        };
        let seq = &aln.seqs[seq_index as usize];
        out.push_str(&format!(">{}\n", seq.label));

        let mut seq_str = String::new();
        for col in 0..col_count {
            let mut c = seq.char_vec[col as usize];
            let is_match = ref_col_to_pos[col as usize] != uint::MAX;
            let is_insert = is_inserts[col as usize];

            if is_match {
                assert!(!is_insert);
                if c == '.' || c == '-' {
                    c = '-';
                } else if c.is_ascii_alphabetic() {
                    c = c.to_ascii_uppercase();
                } else if fasta_allow_digits && c.is_ascii_digit() {
                } else {
                    panic!("Bad char 0x{:02x}", c as u32);
                }
            } else if is_insert {
                assert!(!is_match);
                if c == '.' || c == '-' {
                    continue;
                }
                c = c.to_ascii_lowercase();
            } else {
                unreachable!();
            }
            seq_str.push(c);
        }
        out.push_str(&seq_str);
        out.push('\n');
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write A2M refseq output");
    }
}
