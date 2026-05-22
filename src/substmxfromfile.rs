// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Parse a tab-delimited 20x20 amino-acid substitution matrix file in
/// canonical `ACDEFGHIKLMNPQRSTVWY` column/row order.
#[track_caller]
pub fn read_subst_mx_letter_from_file(file_name: &str) -> [[f32; 20]; 20] {
    let text = std::fs::read_to_string(file_name)
        .unwrap_or_else(|err| panic!("OpenStdioFile({file_name}): {err}"));
    let lines: Vec<&str> = text.lines().collect();
    assert!(!lines.is_empty());

    let amino_letters = *b"ACDEFGHIKLMNPQRSTVWY";
    let fields = split(lines[0], '\t');
    assert_eq!(fields.len(), 21);
    for i in 0..20 {
        let s = &fields[i + 1];
        assert!(s.len() == 1 && s.as_bytes()[0] == amino_letters[i]);
    }

    let mut mx = [[0.0_f32; 20]; 20];
    for i in 0..20 {
        let line = lines
            .get(i + 1)
            .copied()
            .expect("missing substitution matrix row");
        let fields = split(line, '\t');
        assert_eq!(fields.len(), 21);
        let s = &fields[0];
        assert!(s.len() == 1 && s.as_bytes()[0] == amino_letters[i]);
        for j in 0..20 {
            mx[i][j] = str_to_float_l1209(&fields[j + 1], false) as f32;
        }
    }
    mx
}
