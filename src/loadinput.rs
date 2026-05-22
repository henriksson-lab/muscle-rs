// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Loads the input sequences from a FASTA or `.mega` profile file.
#[track_caller]
pub fn load_input(input_file_name: &str, mega: bool) -> MultiSequence {
    let mut input_seqs = MultiSequence::default();
    if mega || ends_with(input_file_name, ".mega") {
        mega_from_file(input_file_name);
        let (labels, seqs) = {
            let mega_state = MEGA_STATE.lock().unwrap();
            (mega_state.labels.clone(), mega_state.seqs.clone())
        };
        multi_sequence_from_strings(&mut input_seqs, &labels, &seqs);
    } else {
        multi_sequence_load_mfa_l8(&mut input_seqs, input_file_name, true);
    }
    set_global_input_ms(&input_seqs);
    input_seqs
}
