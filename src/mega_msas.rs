// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Map a feature letter index to a display character (BLOSUM order for AA, A-Z/a-z otherwise).
pub fn get_feature_char(is_aa: bool, letter: uint) -> byte {
    if is_aa {
        let state = ALPHA_STATE.lock().unwrap();
        return state.char_to_letter[letter as usize] as byte;
    }
    if letter < 26 {
        b'A' + letter as byte
    } else if letter < 26 * 2 {
        b'a' + letter as byte
    } else {
        panic!("invalid feature letter");
    }
}

/// Command implementation: write a per-feature FASTA MSA for every Mega feature.
#[track_caller]
pub fn cmd_mega_msas(fasta_file_name: &str, output_prefix: &str) -> Vec<(String, Vec<byte>)> {
    let aln = msa_from_fasta_file_preserve_case(fasta_file_name);
    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);
    let feature_count = MEGA_STATE.lock().unwrap().feature_count;
    if feature_count == 0 {
        panic!("No features in {}", fasta_file_name);
    }

    let mut outputs = Vec::new();
    for feature_idx in 0..feature_count {
        let feature_name = mega_get_feature_name(feature_idx);
        let is_aa = feature_name == "AA";
        let output_file_name = format!("{output_prefix}{feature_name}");

        let mut out = Vec::new();
        for seq_idx in 0..seq_count {
            let row = sequence_get_seq_as_string(&aln.seqs[seq_idx as usize]);
            let label = msa_get_seq_label(&aln, seq_idx);
            let profile = mega_get_profile_by_label(&label);
            let mut pos = 0_usize;
            let mut feature_row = Vec::new();
            for c in row.chars().take(col_count as usize) {
                if c == '-' || c == '.' {
                    feature_row.push(c as byte);
                } else {
                    assert_eq!(profile[pos].len(), feature_count as usize);
                    let feature_letter = profile[pos][feature_idx as usize] as uint;
                    feature_row.push(get_feature_char(is_aa, feature_letter));
                    pos += 1;
                }
            }

            out.push(b'>');
            out.extend_from_slice(label.as_bytes());
            out.push(b'\n');
            let rowlen = 80;
            let block_count = feature_row.len().div_ceil(rowlen);
            for block_index in 0..block_count {
                let from = block_index * rowlen;
                let mut to = from + rowlen;
                if to >= feature_row.len() {
                    to = feature_row.len();
                }
                out.extend_from_slice(&feature_row[from..to]);
                out.push(b'\n');
            }
        }

        std::fs::write(&output_file_name, &out).expect("failed to write mega feature MSA");
        outputs.push((output_file_name, out));
    }
    outputs
}
