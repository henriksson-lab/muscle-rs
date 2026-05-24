// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns `row` with `-` and `.` gap characters removed.
pub fn strip_gaps(row: &str) -> String {
    let mut ungapped_seq = String::new();
    for r in row.bytes() {
        if r != b'-' && r != b'.' {
            ungapped_seq.push(char::from(r));
        }
    }
    ungapped_seq
}

/// Entry point for `transalnref`: aligns a single fresh sequence to a
/// reference MSA via the named reference row and writes the extended MSA.
#[track_caller]
pub fn cmd_transalnref<FViterbi>(
    ref_aln_file_name: &str,
    add_fasta_file_name: &str,
    ref_label: &str,
    output_file_name: &str,
    mut viterbi_fast_mem: FViterbi,
) -> (TransAln, MultiSequence, String)
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
{
    assert!(!output_file_name.is_empty());
    let mut ref_aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut ref_aln, ref_aln_file_name, false);
    let ref_seq_index = multi_sequence_get_seq_index(&ref_aln, ref_label, true);
    assert_ne!(ref_seq_index, uint::MAX);

    let row_r = &ref_aln.seqs[ref_seq_index as usize];
    let row_r_str = sequence_get_seq_as_string(row_r);
    let r_str = strip_gaps(&row_r_str);
    let lr = r_str.len() as uint;

    let mut seqs_to_add = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut seqs_to_add, add_fasta_file_name, false);
    if seqs_to_add.seqs.len() != 1 {
        die("-input2 must have exactly one sequence");
    }
    let add_label = seqs_to_add.seqs[0].label.clone();
    let row_a_str = sequence_get_seq_as_string(&seqs_to_add.seqs[0]);
    let a_str = strip_gaps(&row_a_str);
    let la = a_str.len() as uint;

    let pi = viterbi_fast_mem(r_str.as_bytes(), lr, a_str.as_bytes(), la);
    let path_str = pi.path;
    let pair_col_count = path_str.len() as uint;
    let mut pos_r = 0usize;
    let mut pos_a = 0usize;
    let mut ids = 0_u32;
    for c in path_str.bytes() {
        match c {
            b'M' => {
                let r = r_str.as_bytes()[pos_r] as char;
                let a = a_str.as_bytes()[pos_a] as char;
                if r.eq_ignore_ascii_case(&a) {
                    ids += 1;
                }
                pos_r += 1;
                pos_a += 1;
            }
            b'D' => pos_r += 1,
            b'I' => pos_a += 1,
            _ => panic!("Invalid TransAlnRef path char '{}'", c as char),
        }
    }
    assert_eq!(pos_r, lr as usize);
    assert_eq!(pos_a, la as usize);
    let pct_id = if pair_col_count == 0 {
        0.0
    } else {
        100.0 * f64::from(ids) / f64::from(pair_col_count)
    };
    log(&write_aln_pretty(
        r_str.as_bytes(),
        a_str.as_bytes(),
        &path_str,
    ));
    let mut log_text = String::new();
    let ref_msg = format!("ref {ref_label}, add {add_label} ({pct_id:.1}% id)\n");
    let _ = progress_log(&ref_msg);
    log_text.push_str(&ref_msg);

    let mut path_str_xyb = String::new();
    for c in path_str.bytes() {
        match c {
            b'M' => path_str_xyb.push('B'),
            b'D' => path_str_xyb.push('Y'),
            b'I' => path_str_xyb.push('X'),
            _ => panic!("Invalid TransAlnRef path char '{}'", c as char),
        }
    }

    let fresh_index_to_msa_index = vec![ref_seq_index];
    let pw_paths = vec![path_str_xyb];
    let mut ta = TransAln::default();
    trans_aln_init(
        &mut ta,
        &ref_aln,
        &seqs_to_add,
        &fresh_index_to_msa_index,
        &pw_paths,
    );
    trans_aln_make_extended_msa(&mut ta);
    let extended_msa = ta
        .extended_msa
        .as_ref()
        .expect("TransAlnRef extended MSA not built")
        .clone();
    multi_sequence_write_mfa(&extended_msa, output_file_name);
    let done_msg = "Done.\n";
    let _ = progress_log(done_msg);
    log_text.push_str(done_msg);
    (ta, extended_msa, log_text)
}
