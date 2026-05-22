// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Format input sequence count and length statistics; aborts if any sequence is too long for global alignment.
#[track_caller]
pub fn show_seq_stats(input_seqs: &MultiSequence) -> String {
    let input_seq_count = input_seqs.seqs.len() as uint;
    let mean_seq_length = multi_sequence_get_mean_seq_length(input_seqs);
    let max_seq_length = multi_sequence_get_max_seq_length(input_seqs);
    let min_seq_length = multi_sequence_get_min_seq_length(input_seqs);
    let out = format!(
        "Input: {} seqs, avg length {:.0}, max {}, min {}\n\n",
        input_seq_count, mean_seq_length, max_seq_length, min_seq_length
    );
    if max_seq_length > 100000 {
        panic!("Too long, not appropriate for global alignment");
    }
    let _warn_long = max_seq_length > 20000;
    out
}
