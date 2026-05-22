// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns a short summary block (file, seq count, min/max length) for an input set.
#[track_caller]
pub fn progress_log_input_summary(file_name: &str, seqs: &MultiSequence) -> String {
    let seq_count = seqs.seqs.len() as uint;
    let mut min_l = 0;
    let mut max_l = 0;
    for i in 0..seq_count as usize {
        let l = seqs.seqs[i].char_vec.len() as uint;
        if i == 0 {
            min_l = l;
            max_l = l;
        } else {
            min_l = min_l.min(l);
            max_l = max_l.max(l);
        }
    }
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("Input  {file_name}\n"));
    out.push_str(&format!("Seqs   {seq_count}\n"));
    out.push_str(&format!("MinL   {min_l}\n"));
    out.push_str(&format!("MaxL   {max_l}\n"));
    out.push('\n');
    out
}

/// Returns a short summary block (label, seq count, col count) for an MSA.
#[track_caller]
pub fn progress_log_msa_summary(s: &str, msa: &MultiSequence) -> String {
    let seq_count = msa.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(msa);
    let mut out = String::new();
    out.push('\n');
    out.push_str(&format!("{s}\n"));
    out.push_str(&format!("Seqs   {seq_count}\n"));
    out.push_str(&format!("Cols   {col_count}\n"));
    out.push('\n');
    out
}
