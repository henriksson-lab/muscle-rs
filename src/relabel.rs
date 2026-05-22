// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `relabel` command: rewrite FASTA labels using a TSV mapping of old to new labels.
#[track_caller]
pub fn cmd_relabel(
    input_file_name: &str,
    labels2_file_name: &str,
    output_file_name: &str,
) -> String {
    let mut m = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut m, input_file_name, false);

    let labels_text = std::fs::read_to_string(labels2_file_name).expect("failed to read labels2");
    let mut old_label_to_new_label = std::collections::BTreeMap::<String, String>::new();
    let mut _label_count = 0_u32;
    for line in labels_text.lines() {
        let fields = split(line, '\t');
        if fields.len() != 2 {
            panic!("Expected 2 fields in line '{}'", line);
        }
        let old_label = fields[0].clone();
        let new_label = fields[1].clone();
        if old_label_to_new_label.contains_key(&old_label) {
            panic!("Dupe label >{}", old_label);
        }
        old_label_to_new_label.insert(old_label, new_label);
        _label_count += 1;
    }

    let mut _not_found = 0_u32;
    let mut _found = 0_u32;
    let mut out = String::new();
    for seq in &m.seqs {
        if let Some(new_label) = old_label_to_new_label.get(&seq.label) {
            out.push_str(&seq_to_fasta_l2561(
                &sequence_get_seq_as_string(seq),
                new_label,
            ));
            _found += 1;
        } else {
            out.push_str(&seq_to_fasta_l2561(
                &sequence_get_seq_as_string(seq),
                &seq.label,
            ));
            _not_found += 1;
        }
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write relabel output");
    }
    out
}
