// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `strip_anchors` command: remove anchor digit columns from a MUSCLE alignment, validating they match across rows.
/// C++ loads via `MSA::FromFASTAFile` which honours `g_FASTA_Upper=true` (the
/// default) and uppercases all letters; the Rust `sequence_from_file_buffer`
/// only uppercases when `strip_gaps` is set, so we force the case here.
#[track_caller]
pub fn cmd_strip_anchors(input_file_name: &str, output_file_name: &str) -> uint {
    let mut aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut aln, input_file_name, false);
    for seq in &mut aln.seqs {
        for c in &mut seq.char_vec {
            if c.is_ascii_alphabetic() {
                *c = c.to_ascii_uppercase();
            }
        }
    }

    let seq_count = aln.seqs.len() as uint;
    assert!(seq_count > 0);
    assert!(multi_sequence_is_aligned(&aln));
    let col_count = multi_sequence_get_col_count(&aln);
    assert!(col_count > 0);

    let seq0 = &aln.seqs[0].char_vec;
    let mut anchor_str = Vec::with_capacity(col_count as usize);
    let mut anchor_count = 0;
    for col in 0..col_count {
        let c = seq0[col as usize];
        if c.is_ascii_digit() {
            anchor_str.push(c);
            anchor_count += 1;
        } else {
            anchor_str.push(' ');
        }
    }

    let mut out = String::new();
    for seq_index in 0..seq_count {
        let seq = &aln.seqs[seq_index as usize];
        out.push_str(&format!(">{}\n", seq.label));

        let mut seq_str = String::new();
        for col in 0..col_count {
            let a = anchor_str[col as usize];
            let c = seq.char_vec[col as usize];
            if a.is_ascii_digit() {
                if c != a {
                    panic!("Mis-aligned anchor");
                }
            } else {
                seq_str.push(c);
            }
        }
        out.push_str(&seq_str);
        out.push('\n');
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write strip anchors output");
    }
    anchor_count
}
