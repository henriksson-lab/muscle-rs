// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MakeSubstMxState {
    pub aln_count: uint,
    pub pct_id_counts: Vec<uint>,
    pub letter_counts: Vec<uint>,
    pub total_letters: uint,
    pub letter_pair_counts: Vec<Vec<uint>>,
    pub total_pairs: uint,
    pub min_pct_id: uint,
    pub max_pct_id: uint,
}

pub static MAKE_SUBST_MX_STATE: std::sync::Mutex<MakeSubstMxState> =
    std::sync::Mutex::new(MakeSubstMxState {
        aln_count: 0,
        pct_id_counts: Vec::new(),
        letter_counts: Vec::new(),
        total_letters: 0,
        letter_pair_counts: Vec::new(),
        total_pairs: 0,
        min_pct_id: 0,
        max_pct_id: 100,
    });

/// Return a copy of `row_in` containing only uppercase letters and gap characters.
pub fn delete_not_upper(row_in: &str) -> String {
    let mut row_out = String::new();
    for c in row_in.bytes() {
        if c.is_ascii_uppercase() || c == b'-' {
            row_out.push(char::from(c));
        }
    }
    row_out
}

/// Accumulate global letter and pair counts for one residue pair.
pub fn add_pair(a: char, b: char) {
    if a == '-' || b == 'b' {
        return;
    }
    let letter = |c: char| -> uint {
        match c {
            'A' | 'a' => 0,
            'C' | 'c' => 1,
            'D' | 'd' => 2,
            'E' | 'e' => 3,
            'F' | 'f' => 4,
            'G' | 'g' => 5,
            'H' | 'h' => 6,
            'I' | 'i' => 7,
            'K' | 'k' => 8,
            'L' | 'l' => 9,
            'M' | 'm' => 10,
            'N' | 'n' => 11,
            'P' | 'p' => 12,
            'Q' | 'q' => 13,
            'R' | 'r' => 14,
            'S' | 's' => 15,
            'T' | 't' => 16,
            'V' | 'v' => 17,
            'W' | 'w' => 18,
            'Y' | 'y' => 19,
            _ => uint::MAX,
        }
    };
    let lettera = letter(a);
    let letterb = letter(b);
    if lettera >= 20 || letterb >= 20 {
        return;
    }

    let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
    state.total_letters += 2;
    state.letter_counts[lettera as usize] += 1;
    state.letter_counts[letterb as usize] += 1;

    state.total_pairs += 2;
    state.letter_pair_counts[lettera as usize][letterb as usize] += 1;
    state.letter_pair_counts[letterb as usize][lettera as usize] += 1;
}

/// Update global counts from a pair of aligned rows, gated by the percent-identity filter.
pub fn add_rows(a: &str, b: &str) {
    let col_count = a.len();
    assert_eq!(b.len(), col_count);
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    let mut n = 0;
    let mut big_n = 0;
    for col in 0..col_count {
        let ac = char::from(a_bytes[col]);
        let bc = char::from(b_bytes[col]);
        add_pair(ac, bc);
        if ac.is_ascii_uppercase() || bc.is_ascii_uppercase() {
            big_n += 1;
            if ac == bc {
                n += 1;
            }
        }
    }
    if big_n == 0 {
        return;
    }

    let (min_pct_id, max_pct_id) = {
        let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
        state.aln_count += 1;
        (state.min_pct_id, state.max_pct_id)
    };
    let pct_id = (n * 100) / big_n;
    assert!(pct_id <= 100);
    if pct_id < min_pct_id || pct_id > max_pct_id {
        return;
    }
    {
        let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
        state.pct_id_counts[pct_id as usize] += 1;
    }

    for col in 0..col_count {
        add_pair(char::from(a_bytes[col]), char::from(b_bytes[col]));
    }
}

/// Command implementation: build an amino-acid substitution matrix from a set of MSAs.
#[track_caller]
pub fn cmd_make_substmx(
    input_file_name: &str,
    output_file_name: &str,
    label: Option<&str>,
    min_pct_id: Option<uint>,
    max_pct_id: Option<uint>,
) -> String {
    if min_pct_id.is_some() {
        assert!(max_pct_id.is_some());
    }
    {
        let mut state = MAKE_SUBST_MX_STATE.lock().unwrap();
        *state = MakeSubstMxState {
            aln_count: 0,
            pct_id_counts: vec![0; 101],
            letter_counts: vec![0; 20],
            total_letters: 0,
            letter_pair_counts: vec![vec![0; 20]; 20],
            total_pairs: 0,
            min_pct_id: min_pct_id.unwrap_or(0),
            max_pct_id: max_pct_id.unwrap_or(100),
        };
        assert!(state.min_pct_id <= state.max_pct_id);
    }

    let mx_name = label.unwrap_or("MX");
    let msa_file_names = read_strings_from_file(input_file_name);
    let mut label_pair_to_rows =
        std::collections::BTreeMap::<(String, String), (String, String)>::new();
    for file_name in msa_file_names {
        let msa = msa_from_fasta_file_l95(&file_name);
        let seq_count = msa.seqs.len();
        let col_count = multi_sequence_get_col_count(&msa) as usize;
        for i in 0..seq_count {
            let label_i = msa.seqs[i].label.clone();
            let row_i = delete_not_upper(&msa_get_row_str(&msa, i as uint));
            for j in 0..i {
                let label_j = msa.seqs[j].label.clone();
                let row_j = delete_not_upper(&msa_get_row_str(&msa, j as uint));
                assert_eq!(row_i.len(), row_j.len());
                let key = (label_i.clone(), label_j);
                if let Some((curr_row_i, _curr_row_j)) = label_pair_to_rows.get(&key) {
                    if curr_row_i.len() < col_count {
                        label_pair_to_rows.insert(key, (row_i.clone(), row_j));
                    }
                } else {
                    label_pair_to_rows.insert(key, (row_i.clone(), row_j));
                }
            }
        }
    }

    for (_label_pair, rows) in &label_pair_to_rows {
        add_rows(&rows.0, &rows.1);
    }

    let state = MAKE_SUBST_MX_STATE.lock().unwrap();
    assert!(state.total_letters > 0);
    assert!(state.total_pairs > 0);
    let mut log = String::new();
    let mut freqs = vec![0.0_f64; 20];
    let mut sum = 0.0_f64;
    for i in 0..20 {
        let c = AMINO_ALPHA.as_bytes()[i] as char;
        let n = state.letter_counts[i];
        let freq = f64::from(n) / f64::from(state.total_letters);
        freqs[i] = freq;
        sum += freq;
        log.push_str(&format!("{c}  {freq:8.6}  {n}\n"));
    }
    log.push_str(&format!("Sum = {sum:8.6}\n\n\n"));

    for i in 0..20 {
        let ci = AMINO_ALPHA.as_bytes()[i] as char;
        log.push_str(&format!("{ci}:  "));
        for j in 0..20 {
            let cj = AMINO_ALPHA.as_bytes()[j] as char;
            log.push_str(&format!("  {cj}={:10}", state.letter_pair_counts[i][j]));
        }
        log.push('\n');
    }

    log.push_str("\nPctId distribution\n");
    let format_g4 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d:.3e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mut mantissa = mantissa
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string();
            if mantissa == "-0" {
                mantissa = "0".to_string();
            }
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (3 - exp).max(0) as usize;
            format!("{d:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    for pct_id in (0..=100).rev() {
        let n = state.pct_id_counts[pct_id];
        let freq = f64::from(n) / f64::from(state.aln_count);
        log.push_str(&format!("{pct_id}\t{n}\t{}\n", format_g4(freq)));
    }

    log.push_str("\nPair frequencies\n ");
    for i in 0..20 {
        let ci = AMINO_ALPHA.as_bytes()[i] as char;
        log.push_str(&format!("{ci:>10}"));
    }
    log.push('\n');
    for i in 0..20 {
        let ci = AMINO_ALPHA.as_bytes()[i] as char;
        log.push(ci);
        for j in 0..=i {
            let freq_ij = f64::from(state.letter_pair_counts[i][j]) / f64::from(state.total_pairs);
            log.push_str(&format!("{freq_ij:10.5}"));
        }
        log.push('\n');
    }

    log.push_str("\nScore matrix\n ");
    for i in 0..20 {
        let ci = AMINO_ALPHA.as_bytes()[i] as char;
        log.push_str(&format!("{ci:>10}"));
    }
    log.push('\n');
    for i in 0..20 {
        let ci = AMINO_ALPHA.as_bytes()[i] as char;
        log.push(ci);
        for j in 0..=i {
            let freq_ij = f64::from(state.letter_pair_counts[i][j]) / f64::from(state.total_pairs);
            let score = (freq_ij / (freqs[i] * freqs[j])).log2();
            log.push_str(&format!("{score:10.6}"));
        }
        log.push('\n');
    }

    if !output_file_name.is_empty() {
        let mut out = String::new();
        out.push_str(mx_name);
        for i in 0..20 {
            let ci = AMINO_ALPHA.as_bytes()[i] as char;
            out.push('\t');
            out.push(ci);
        }
        out.push('\n');
        for i in 0..20 {
            let ci = AMINO_ALPHA.as_bytes()[i] as char;
            out.push(ci);
            for j in 0..20 {
                let freq_ij =
                    f64::from(state.letter_pair_counts[i][j]) / f64::from(state.total_pairs);
                let score = (freq_ij / (freqs[i] * freqs[j])).log2();
                out.push_str(&format!("\t{score:.3}"));
            }
            out.push('\n');
        }
        std::fs::write(output_file_name, out).expect("failed to write substitution matrix");
    }
    log
}
