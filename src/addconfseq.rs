// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Encodes a confidence value (0.0..=1.0) as a single decimal digit, or `+` for 1.0.
pub fn conf_to_char1(conf: f64) -> char {
    assert!((0.0..=1.0).contains(&conf));
    let tenth = (conf * 10.0) as uint;
    assert!(tenth <= 10);
    if tenth == 10 {
        '+'
    } else {
        char::from(b'0' + tenth as byte)
    }
}

/// Encodes the hundredths digit of a confidence value, or `+` for 1.0.
pub fn conf_to_char2(conf: f64) -> char {
    assert!((0.0..=1.0).contains(&conf));
    let h = (conf * 100.0) as uint;
    assert!(h <= 100);
    if h == 100 {
        '+'
    } else {
        char::from(b'0' + (h % 10) as byte)
    }
}

/// Maps a confidence value to one of 11 symbolic glyphs from `___.,/:=@*^`.
pub fn conf_to_char_1(conf: f64) -> char {
    assert!((0.0..=1.0).contains(&conf));
    let tenth = (conf * 10.0) as usize;
    assert!(tenth <= 10);
    char::from(b"___.,/:=@*^"[tenth])
}

/// Builds a FASTA record encoding per-column confidences using the chosen encoding (`dec`).
#[track_caller]
pub fn do1(confidences: &[f64], conf_label: &str, dec: i32) -> String {
    let mut conf_seq = String::new();
    for conf in confidences {
        let c = match dec {
            -1 => conf_to_char_1(*conf),
            1 => conf_to_char1(*conf),
            2 => conf_to_char2(*conf),
            _ => panic!("invalid confidence decimal mode {dec}"),
        };
        conf_seq.push(c);
    }
    format!(">{conf_label}\n{conf_seq}\n")
}

/// `addconfseq` subcommand: emits each MSA in the ensemble with confidence pseudo-sequences prepended.
#[track_caller]
pub fn cmd_addconfseq(
    input_file_name: &str,
    output_file_name: &str,
    ref_file_name: Option<&str>,
    conf_label: Option<&str>,
    confseq1: bool,
) -> String {
    let conf_label = conf_label.unwrap_or("_conf_");

    let mut ref_msa = ref_file_name.map(msa_from_fasta_file_preserve_case);

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, input_file_name);
    if let Some(ref mut ref_msa) = ref_msa {
        ensemble_sort_msa(&e, ref_msa);
    }

    let msa_count = e.msas.len() as uint;
    let seq_count = ensemble_get_seq_count(&e);
    let mut out = String::new();
    for msa_index in 0..msa_count {
        let msa_name = ensemble_get_msa_name(&e, msa_index);
        out.push('<');
        out.push_str(msa_name);
        out.push('\n');
        let m = ensemble_get_msa(&e, msa_index);

        let col_count = multi_sequence_get_col_count(m);
        let msa_seq_count = m.seqs.len() as uint;
        assert_eq!(msa_seq_count, seq_count);

        let confidences = (0..col_count)
            .map(|col_index| ensemble_get_conf_msa_col(&e, msa_index, col_index))
            .collect::<Vec<_>>();
        if confseq1 {
            out.push_str(&do1(&confidences, conf_label, -1));
        } else {
            out.push_str(&do1(&confidences, conf_label, 1));
            out.push_str(&do1(&confidences, &format!("{conf_label}2"), 2));
        }

        for seq_index in 0..seq_count {
            let seq = &m.seqs[seq_index as usize];
            out.push('>');
            out.push_str(&seq.label);
            out.push('\n');
            for col in 0..col_count {
                out.push(seq.char_vec[col as usize]);
            }
            out.push('\n');
        }
    }

    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write addconfseq output");
    }
    out
}
