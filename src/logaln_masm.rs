// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Reconstructs the amino-acid consensus string from a Mega-profile's `AA` feature column.
#[track_caller]
pub fn get_mega_profile_aa_seq(profile: &[Vec<byte>]) -> String {
    const LETTER_TO_CHAR_AMINO: &[u8; 21] = b"ACDEFGHIKLMNPQRSTVWY*";

    let mega = MEGA_STATE.lock().unwrap();
    let mut pi = uint::MAX;
    for i in 0..mega.feature_names.len() {
        if mega.feature_names[i] == "AA" {
            pi = i as uint;
            break;
        }
    }
    assert_ne!(pi, uint::MAX);
    let mut seq = String::new();
    for row in profile {
        let letter = row[pi as usize] as usize;
        let c = LETTER_TO_CHAR_AMINO.get(letter).copied().unwrap_or(b'?');
        seq.push(c as char);
    }
    seq
}

/// Pretty-prints a local MASM-vs-Mega-profile alignment in 80-column blocks.
#[track_caller]
pub fn write_local_aln_masm(
    label_a: &str,
    ma: &MASM,
    label_b: &str,
    pb: &[Vec<byte>],
    loi: uint,
    loj: uint,
    path: &str,
) -> String {
    const BLOCK_SIZE: uint = 80;
    let str_a = masm_get_consensus_seq(ma);
    let str_b = get_mega_profile_aa_seq(pb);
    let a = str_a.as_bytes();
    let b = str_b.as_bytes();
    let col_lo = 0;
    let col_hi = path.len() as uint - 1;
    assert!(col_hi >= col_lo);

    let mut out = String::new();
    let mut pos_a = loi;
    let mut pos_b = loj;
    let mut col_from = col_lo;
    loop {
        if col_from > col_hi {
            break;
        }
        let mut col_to = col_from + BLOCK_SIZE - 1;
        if col_to > col_hi {
            col_to = col_hi;
        }

        out.push('\n');
        let i0 = pos_a;
        let j0 = pos_b;
        let (a_row, new_pos_a) = write_a_row(a, path, pos_a, col_from, col_to, label_a);
        pos_a = new_pos_a;
        out.push_str(&a_row);
        out.push_str(&write_annot_row(a, b, path, i0, j0, col_from, col_to));
        let (b_row, new_pos_b) = write_b_row(b, path, pos_b, col_from, col_to, label_b);
        pos_b = new_pos_b;
        out.push_str(&b_row);

        col_from += BLOCK_SIZE;
    }
    out
}
