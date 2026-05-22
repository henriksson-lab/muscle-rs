// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const INVALID_LETTER: uint = uint::MAX;

pub const AX_GAP: uint = 23;

pub const AMINO_ALPHA: &str = "ACDEFGHIKLMNPQRSTVWY";

pub const NT_ALPHA: &str = "ACGT";

#[derive(Clone, Debug)]
pub(crate) struct AlphaState {
    pub(crate) char_to_letter: [uint; 256],
    pub(crate) char_to_letter_ex: [uint; 256],
    pub(crate) letter_to_char: [byte; 256],
    pub(crate) letter_ex_to_char: [byte; 256],
    pub(crate) align_char: [byte; 256],
    pub(crate) unalign_char: [byte; 256],
    pub(crate) is_wildcard_char: [bool; 256],
    pub(crate) is_residue_char: [bool; 256],
    pub(crate) alpha: ALPHA,
    pub(crate) alpha_size: uint,
    pub(crate) invalid_letters: [bool; 256],
    pub(crate) invalid_letter_count: i32,
}

pub(crate) static ALPHA_STATE: std::sync::Mutex<AlphaState> = std::sync::Mutex::new(AlphaState {
    char_to_letter: [INVALID_LETTER; 256],
    char_to_letter_ex: [INVALID_LETTER; 256],
    letter_to_char: [b'?'; 256],
    letter_ex_to_char: [b'?'; 256],
    align_char: [b'?'; 256],
    unalign_char: [b'?'; 256],
    is_wildcard_char: [false; 256],
    is_residue_char: [false; 256],
    alpha: ALPHA::ALPHA_Undefined,
    alpha_size: 0,
    invalid_letters: [false; 256],
    invalid_letter_count: 0,
});

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ALPHA {
    #[default]
    ALPHA_Undefined,
    ALPHA_Nucleo,
    ALPHA_Amino,
} // original: ALPHA (muscle/src/alpha.h)

/// Returns the residue alphabet size (20 for amino, 4 for nucleotide).
#[track_caller]
pub fn get_alpha_size(alpha: ALPHA) -> uint {
    match alpha {
        ALPHA::ALPHA_Amino => 20,
        ALPHA::ALPHA_Nucleo => 4,
        _ => panic!("Invalid Alpha={alpha:?}"),
    }
}

/// Resets all character-to-letter lookup tables to their uninitialised state.
#[track_caller]
pub fn init_arrays() {
    let mut state = ALPHA_STATE.lock().unwrap();
    state.char_to_letter = [INVALID_LETTER; 256];
    state.char_to_letter_ex = [INVALID_LETTER; 256];
    state.letter_to_char = [b'?'; 256];
    state.letter_ex_to_char = [b'?'; 256];
    state.align_char = [b'?'; 256];
    state.unalign_char = [b'?'; 256];
    state.is_wildcard_char = [false; 256];
    state.is_residue_char = [false; 256];
}

/// Registers `c` as a gap character in the extended letter and align/unalign tables.
#[track_caller]
pub fn set_gap_char(c: char) {
    let mut state = ALPHA_STATE.lock().unwrap();
    let u = c as usize;
    state.char_to_letter_ex[u] = AX_GAP;
    state.letter_ex_to_char[AX_GAP as usize] = c as byte;
    state.align_char[u] = c as byte;
    state.unalign_char[u] = c as byte;
}

/// Populates the alphabet tables for the generic nucleotide alphabet (DNA+RNA + IUPAC wildcards).
#[track_caller]
pub fn set_alpha_nucleo() {
    let mut state = ALPHA_STATE.lock().unwrap();
    for (c, letter, residue) in [
        ('A', 0, true),
        ('C', 1, true),
        ('G', 2, true),
        ('T', 3, true),
        ('U', 3, true),
        ('M', 4, false),
        ('R', 5, false),
        ('W', 6, false),
        ('S', 7, false),
        ('Y', 8, false),
        ('K', 9, false),
        ('V', 10, false),
        ('H', 11, false),
        ('D', 12, false),
        ('B', 13, false),
        ('X', 14, false),
        ('N', 15, false),
    ] {
        let upper = c.to_ascii_uppercase() as usize;
        let lower = c.to_ascii_lowercase() as usize;
        if residue {
            state.char_to_letter[upper] = letter;
            state.char_to_letter[lower] = letter;
            state.letter_to_char[letter as usize] = upper as byte;
        } else {
            state.is_wildcard_char[upper] = true;
            state.is_wildcard_char[lower] = true;
        }
        state.char_to_letter_ex[upper] = letter;
        state.char_to_letter_ex[lower] = letter;
        state.letter_ex_to_char[letter as usize] = upper as byte;
        state.is_residue_char[upper] = true;
        state.is_residue_char[lower] = true;
        state.align_char[upper] = upper as byte;
        state.align_char[lower] = upper as byte;
        state.unalign_char[upper] = lower as byte;
        state.unalign_char[lower] = lower as byte;
    }
}

/// Populates the alphabet tables for DNA (ACGT plus IUPAC wildcards).
#[track_caller]
pub fn set_alpha_dna() {
    let mut state = ALPHA_STATE.lock().unwrap();
    for (c, letter, residue) in [
        ('A', 0, true),
        ('C', 1, true),
        ('G', 2, true),
        ('T', 3, true),
        ('M', 4, false),
        ('R', 5, false),
        ('W', 6, false),
        ('S', 7, false),
        ('Y', 8, false),
        ('K', 9, false),
        ('V', 10, false),
        ('H', 11, false),
        ('D', 12, false),
        ('B', 13, false),
        ('X', 14, false),
        ('N', 15, false),
    ] {
        let upper = c.to_ascii_uppercase() as usize;
        let lower = c.to_ascii_lowercase() as usize;
        if residue {
            state.char_to_letter[upper] = letter;
            state.char_to_letter[lower] = letter;
            state.letter_to_char[letter as usize] = upper as byte;
        } else {
            state.is_wildcard_char[upper] = true;
            state.is_wildcard_char[lower] = true;
        }
        state.char_to_letter_ex[upper] = letter;
        state.char_to_letter_ex[lower] = letter;
        state.letter_ex_to_char[letter as usize] = upper as byte;
        state.is_residue_char[upper] = true;
        state.is_residue_char[lower] = true;
        state.align_char[upper] = upper as byte;
        state.align_char[lower] = upper as byte;
        state.unalign_char[upper] = lower as byte;
        state.unalign_char[lower] = lower as byte;
    }
}

/// Populates the alphabet tables for RNA (ACGU plus IUPAC wildcards).
#[track_caller]
pub fn set_alpha_rna() {
    let mut state = ALPHA_STATE.lock().unwrap();
    for (c, letter, residue) in [
        ('A', 0, true),
        ('C', 1, true),
        ('G', 2, true),
        ('U', 3, true),
        ('T', 3, true),
        ('M', 4, false),
        ('R', 5, false),
        ('W', 6, false),
        ('S', 7, false),
        ('Y', 8, false),
        ('K', 9, false),
        ('V', 10, false),
        ('H', 11, false),
        ('D', 12, false),
        ('B', 13, false),
        ('X', 14, false),
        ('N', 15, false),
    ] {
        let upper = c.to_ascii_uppercase() as usize;
        let lower = c.to_ascii_lowercase() as usize;
        if residue {
            state.char_to_letter[upper] = letter;
            state.char_to_letter[lower] = letter;
            state.letter_to_char[letter as usize] = upper as byte;
        } else {
            state.is_wildcard_char[upper] = true;
            state.is_wildcard_char[lower] = true;
        }
        state.char_to_letter_ex[upper] = letter;
        state.char_to_letter_ex[lower] = letter;
        state.letter_ex_to_char[letter as usize] = upper as byte;
        state.is_residue_char[upper] = true;
        state.is_residue_char[lower] = true;
        state.align_char[upper] = upper as byte;
        state.align_char[lower] = upper as byte;
        state.unalign_char[upper] = lower as byte;
        state.unalign_char[lower] = lower as byte;
    }
}

/// Populates the alphabet tables for the 20 amino acids plus B/X/Z wildcards.
#[track_caller]
pub fn set_alpha_amino() {
    let mut state = ALPHA_STATE.lock().unwrap();
    for (c, letter, residue) in [
        ('A', 0, true),
        ('C', 1, true),
        ('D', 2, true),
        ('E', 3, true),
        ('F', 4, true),
        ('G', 5, true),
        ('H', 6, true),
        ('I', 7, true),
        ('K', 8, true),
        ('L', 9, true),
        ('M', 10, true),
        ('N', 11, true),
        ('P', 12, true),
        ('Q', 13, true),
        ('R', 14, true),
        ('S', 15, true),
        ('T', 16, true),
        ('V', 17, true),
        ('W', 18, true),
        ('Y', 19, true),
        ('B', 20, false),
        ('X', 21, false),
        ('Z', 22, false),
    ] {
        let upper = c.to_ascii_uppercase() as usize;
        let lower = c.to_ascii_lowercase() as usize;
        if residue {
            state.char_to_letter[upper] = letter;
            state.char_to_letter[lower] = letter;
            state.letter_to_char[letter as usize] = upper as byte;
        } else {
            state.is_wildcard_char[upper] = true;
            state.is_wildcard_char[lower] = true;
        }
        state.char_to_letter_ex[upper] = letter;
        state.char_to_letter_ex[lower] = letter;
        state.letter_ex_to_char[letter as usize] = upper as byte;
        state.is_residue_char[upper] = true;
        state.is_residue_char[lower] = true;
        state.align_char[upper] = upper as byte;
        state.align_char[lower] = upper as byte;
        state.unalign_char[upper] = lower as byte;
        state.unalign_char[lower] = lower as byte;
    }
}

/// Initialises the global alphabet tables for the requested `alpha`, registers `.`/`-` as gaps.
#[track_caller]
pub fn set_alpha_l209(alpha: ALPHA) {
    init_arrays();
    set_gap_char('.');
    set_gap_char('-');
    match alpha {
        ALPHA::ALPHA_Amino => set_alpha_amino(),
        ALPHA::ALPHA_Nucleo => set_alpha_nucleo(),
        _ => panic!("Invalid Alpha={alpha:?}"),
    }
    let mut state = ALPHA_STATE.lock().unwrap();
    state.alpha_size = get_alpha_size(alpha);
    state.alpha = alpha;
}

/// Returns `'X'` for protein or `'N'` for nucleotide, panics otherwise.
#[track_caller]
pub fn get_wildcard_char() -> char {
    match ALPHA_STATE.lock().unwrap().alpha {
        ALPHA::ALPHA_Amino => 'X',
        ALPHA::ALPHA_Nucleo => 'N',
        alpha => panic!("Invalid Alpha={alpha:?}"),
    }
}

/// True if `c` is one of `ACGTU` plus a few IUPAC ambiguity codes.
#[track_caller]
pub fn is_nucleo(c: char) -> bool {
    "ACGTURYNacgturyn".contains(c)
}

/// True if `c` is one of `AGCTN` (either case).
#[track_caller]
pub fn is_dna(c: char) -> bool {
    "AGCTNagctn".contains(c)
}

/// True if `c` is one of `AGCUN` (either case).
#[track_caller]
pub fn is_rna(c: char) -> bool {
    "AGCUNagcun".contains(c)
}

/// Resets the accumulated set of invalid-letter warnings.
#[track_caller]
pub fn clear_invalid_letter_warning() {
    let mut state = ALPHA_STATE.lock().unwrap();
    state.invalid_letters = [false; 256];
}

/// Records that letter `c` was rejected by the current alphabet (for later reporting).
#[track_caller]
pub fn invalid_letter_warning(c: char, _w: char) {
    let mut state = ALPHA_STATE.lock().unwrap();
    state.invalid_letters[c as usize] = true;
    state.invalid_letter_count += 1;
}

/// Returns a warning message listing accumulated invalid letters, or `None` if none seen.
#[track_caller]
pub fn report_invalid_letters() -> Option<String> {
    let state = ALPHA_STATE.lock().unwrap();
    if state.invalid_letter_count == 0 {
        return None;
    }
    let mut s = String::new();
    for i in 0..256 {
        if state.invalid_letters[i] {
            s.push(i as u8 as char);
        }
    }
    Some(format!("Invalid letters found: {s}"))
}
