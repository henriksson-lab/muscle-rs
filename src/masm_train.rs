// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: print summary statistics for a stored MASM file.
#[track_caller]
pub fn cmd_masm_stats(file_name: &str) -> String {
    let mut m = MASM::default();
    masm_from_file(&mut m, file_name);

    let mut out = String::new();
    out.push_str(&format!("{:10}  Sequences\n", m.seq_count));
    out.push_str(&format!("{:10}  Columns\n", m.col_count));
    out.push_str(&format!("{:10}  Features ", m.feature_count));
    for feature_idx in 0..m.feature_count as usize {
        out.push_str(&format!(
            " {}/{}",
            m.feature_names[feature_idx], m.alpha_sizes[feature_idx]
        ));
    }
    out.push('\n');
    out
}

/// Command implementation: train a MASM from an MSA using the given Mega parameters.
#[track_caller]
pub fn cmd_masm_train<FLoadMega>(
    aln_file_name: &str,
    mega_file_name: &str,
    output_file_name: &str,
    label: Option<&str>,
    mut load_mega: FLoadMega,
) -> MASM
where
    FLoadMega: FnMut(&str),
{
    load_mega(mega_file_name);

    let mut aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut aln, aln_file_name, false);

    let label = if let Some(label) = label {
        label.to_string()
    } else {
        base_name(aln_file_name).to_string()
    };

    let (gap_open, gap_ext) = {
        let mega = MEGA_STATE.lock().unwrap();
        (mega.gap_open, mega.gap_ext)
    };
    let mut m = MASM::default();
    masm_from_msa(&mut m, &aln, &label, gap_open, gap_ext);
    masm_to_file_l150(&m, output_file_name);
    m
}
