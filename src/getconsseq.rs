// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Returns the most-frequent letter in column `col_index`, or `'-'` if gaps dominate.
#[track_caller]
pub fn get_cons_char(msa: &MultiSequence, col_index: uint) -> char {
    let state = ALPHA_STATE.lock().unwrap();
    assert!(state.alpha_size == 4 || state.alpha_size == 20);
    let alpha_size = state.alpha_size as usize;
    let mut counts = vec![0_u32; alpha_size + 1];
    let col_count = multi_sequence_get_col_count(msa);
    assert!(col_index < col_count);
    let seq_count = msa.seqs.len();
    for seq_index in 0..seq_count {
        let c = msa.seqs[seq_index].char_vec[col_index as usize];
        if c == '-' || c == '.' {
            counts[alpha_size] += 1;
            continue;
        }
        let letter = state.char_to_letter[c as usize];
        if letter < state.alpha_size {
            counts[letter as usize] += 1;
        }
    }

    let mut max_count = 0;
    let mut max_letter = 0;
    for letter in 0..=alpha_size {
        let count = counts[letter];
        if count > max_count {
            max_count = count;
            max_letter = letter;
        }
    }
    if max_letter == alpha_size {
        return '-';
    }
    state.letter_to_char[max_letter] as char
}

/// Builds an ungapped consensus string by taking the majority character of each column.
#[track_caller]
pub fn get_consensus_sequence(msa: &MultiSequence) -> String {
    let mut seq = String::new();
    let seq_count = msa.seqs.len();
    let col_count = multi_sequence_get_col_count(msa);
    let _freqs = vec![0_u32; ALPHA_STATE.lock().unwrap().alpha_size as usize];
    for col_index in 0..col_count {
        let c = get_cons_char(msa, col_index);
        if c != '-' {
            seq.push(c);
        }
    }
    let _ = seq_count;
    seq
}

/// Consseq command: reads an MSA and writes the consensus sequence as a FASTA file.
#[track_caller]
pub fn cmd_consseq(msa_file_name: &str, output_file_name: &str, label: Option<&str>) {
    let label = label.unwrap_or("CONSENSUS");
    let mut msa = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa, msa_file_name, false);
    let cons_seq = get_consensus_sequence(&msa);
    let out = seq_to_fasta_l2561(&cons_seq, label);
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, out).expect("failed to write consensus FASTA");
    }
}
