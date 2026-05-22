// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MASMCol {
    pub masm: Option<Box<MASM>>,
    pub col_index: uint,
    pub gap_open: f32,
    pub gap_ext: f32,
    pub gap_close: f32,
    pub letter_freq: f32,
    pub gap_open_freq: f32,
    pub gap_ext_freq: f32,
    pub gap_close_freq: f32,
    pub freqs_vec: Vec<Vec<f32>>,
    pub scores_vec: Vec<Vec<f32>>,
} // original: MASMCol (muscle/src/masmcol.h)

/// Return the consensus amino-acid character for a MASM column (lowercase if ambiguous, '-' if gap).
#[track_caller]
pub fn masm_col_get_consensus_aa_char(col: &MASMCol) -> char {
    let masm = col
        .masm
        .as_ref()
        .expect("MASMCol::GetConsensusAAChar, m_MASM is null");
    let fi = masm.aa_feature_idx as usize;
    assert!(fi < col.freqs_vec.len());
    let freqs = &col.freqs_vec[fi];
    assert_eq!(freqs.len(), 20);
    let mut max_freq = 0.0_f32;
    let mut sum_freq = 0.0_f32;
    let mut max_letter = 0_usize;
    for letter in 0..20 {
        let freq = freqs[letter];
        if freq > max_freq {
            max_freq = freq;
            max_letter = letter;
        }
        sum_freq += freq;
    }
    if sum_freq < 0.5 {
        return '-';
    }
    let c = ALPHA_STATE.lock().unwrap().letter_to_char[max_letter] as char;
    if max_freq < 0.5 {
        return c.to_ascii_lowercase();
    }
    c
}

/// Compute per-letter log-odds scores for each feature from the column's letter frequencies.
#[track_caller]
pub fn masm_col_set_score_vec(col: &mut MASMCol) {
    let feature_count = col.freqs_vec.len();
    col.scores_vec.clear();
    col.scores_vec.resize(feature_count, Vec::new());
    let mega = MEGA_STATE.lock().unwrap();
    for feature_idx in 0..feature_count {
        assert!(feature_idx < mega.alpha_sizes.len());
        assert!(feature_idx < mega.log_odds_mx_vec.len());
        let alpha_size = mega.alpha_sizes[feature_idx] as usize;
        let score_mx = &mega.log_odds_mx_vec[feature_idx];
        assert_eq!(score_mx.len(), alpha_size);
        let freqs = &col.freqs_vec[feature_idx];
        assert_eq!(freqs.len(), alpha_size);
        for letter in 0..alpha_size {
            assert_eq!(score_mx[letter].len(), alpha_size);
            let mut total = 0.0_f32;
            for letter2 in 0..alpha_size {
                let freq2 = freqs[letter2];
                total += freq2 * score_mx[letter][letter2];
            }
            col.scores_vec[feature_idx].push(total);
        }
    }
}

/// Return the per-letter score vector for the AA feature.
#[track_caller]
pub fn masm_col_get_aa_scores(col: &MASMCol) -> &[f32] {
    let masm = col
        .masm
        .as_ref()
        .expect("MASMCol::GetAAScores, m_MASM is null");
    let aa_feature_idx = masm.aa_feature_idx as usize;
    assert!(aa_feature_idx < col.scores_vec.len());
    &col.scores_vec[aa_feature_idx]
}

/// Return a human-readable dump of one MASM column for logging.
#[track_caller]
pub fn masm_col_log_me(col: &MASMCol) -> String {
    let mut out = format!("  MSAMCol[{}]", col.col_index);
    let format_g3 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d64:.2e}");
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
            let decimals = (2 - exp).max(0) as usize;
            format!("{d64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let feature_count = col.freqs_vec.len();
    let mega = MEGA_STATE.lock().unwrap();
    if feature_count == 1 {
        let feature_idx = 0_usize;
        assert!(feature_idx < mega.alpha_sizes.len());
        assert!(feature_idx < mega.feature_names.len());
        assert!(feature_idx < col.scores_vec.len());
        let alpha_size = mega.alpha_sizes[feature_idx] as usize;
        let name = &mega.feature_names[feature_idx];
        let scores = &col.scores_vec[feature_idx];
        out.push_str(&format!("  | {name} |"));
        let alpha_state = ALPHA_STATE.lock().unwrap();
        for letter in 0..alpha_size {
            out.push_str(&format!(
                " {}={}",
                alpha_state.letter_to_char[letter] as char,
                format_g3(scores[letter])
            ));
        }
        out.push('\n');
        return out;
    }

    out.push('\n');
    let alpha_state = ALPHA_STATE.lock().unwrap();
    for feature_idx in 0..feature_count {
        assert!(feature_idx < mega.alpha_sizes.len());
        assert!(feature_idx < mega.feature_names.len());
        assert!(feature_idx < col.scores_vec.len());
        let alpha_size = mega.alpha_sizes[feature_idx] as usize;
        let name = &mega.feature_names[feature_idx];
        let scores = &col.scores_vec[feature_idx];
        let truncated_name: String = name.chars().take(8).collect();
        out.push_str(&format!("  | {truncated_name:>8} |"));
        for letter in 0..alpha_size {
            out.push_str(&format!(
                " {}={}",
                alpha_state.letter_to_char[letter] as char,
                format_g3(scores[letter])
            ));
        }
        out.push('\n');
    }
    out
}

/// Write one MASM column (frequencies and scores per feature) to an open text file.
#[track_caller]
pub fn masm_col_to_file(col: &MASMCol, file: &mut TextFile) {
    let feature_count = col.freqs_vec.len();
    assert_eq!(col.scores_vec.len(), feature_count);
    text_file_put_format(file, &format!("col\t{}\n", col.col_index));
    let mega = MEGA_STATE.lock().unwrap();
    let format_g3 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d64:.2e}");
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
            let decimals = (2 - exp).max(0) as usize;
            format!("{d64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    for feature_idx in 0..feature_count {
        assert!(feature_idx < mega.alpha_sizes.len());
        let alpha_size = mega.alpha_sizes[feature_idx] as usize;
        text_file_put_format(file, &format!("colfeature\t{feature_idx}\n"));

        let freqs = &col.freqs_vec[feature_idx];
        text_file_put_string(file, "freqs");
        for freq in freqs.iter().take(alpha_size) {
            text_file_put_format(file, &format!("\t{}", format_g3(*freq)));
        }
        text_file_put_char(file, b'\n');

        let scores = &col.scores_vec[feature_idx];
        text_file_put_string(file, "scores");
        for score in scores.iter().take(alpha_size) {
            text_file_put_format(file, &format!("\t{}", format_g3(*score)));
        }
        text_file_put_char(file, b'\n');
    }
}

/// Read one MASM column (frequencies and scores per feature) from an open text file.
#[track_caller]
pub fn masm_col_from_file(col: &mut MASMCol, file: &mut TextFile) {
    let masm = col
        .masm
        .as_ref()
        .expect("MASMCol::FromFile, m_MASM is null");
    let feature_count = masm.feature_count as usize;
    let line = text_file_get_line_x(file, uint::MAX);
    let fields = split(line.trim_end_matches(['\r', '\n']), '\t');
    assert_eq!(fields[0], "col");
    assert_eq!(fields.len(), 2);
    col.col_index = str_to_uint_l1278(&fields[1], false);
    col.freqs_vec.clear();
    col.scores_vec.clear();
    col.freqs_vec.resize(feature_count, Vec::new());
    col.scores_vec.resize(feature_count, Vec::new());
    for feature_idx in 0..feature_count {
        let line = text_file_get_line_x(file, uint::MAX);
        let fields = split(line.trim_end_matches(['\r', '\n']), '\t');
        assert_eq!(fields[0], "colfeature");
        assert_eq!(fields.len(), 2);
        assert_eq!(str_to_uint_l1278(&fields[1], false), feature_idx as uint);
        assert!(feature_idx < masm.alpha_sizes.len());
        let alpha_size = masm.alpha_sizes[feature_idx] as usize;

        let line = text_file_get_line_x(file, uint::MAX);
        let fields = split(line.trim_end_matches(['\r', '\n']), '\t');
        assert_eq!(fields[0], "freqs");
        assert_eq!(fields.len(), alpha_size + 1);
        let mut sum_freqs = 0.0_f32;
        for letter in 0..alpha_size {
            let freq = str_to_float_l1204(&fields[letter + 1], false) as f32;
            col.freqs_vec[feature_idx].push(freq);
            sum_freqs += freq;
        }
        assert!(sum_freqs < 1.001);

        let line = text_file_get_line_x(file, uint::MAX);
        let fields = split(line.trim_end_matches(['\r', '\n']), '\t');
        assert_eq!(fields[0], "scores");
        assert_eq!(fields.len(), alpha_size + 1);
        for letter in 0..alpha_size {
            col.scores_vec[feature_idx].push(str_to_float_l1204(&fields[letter + 1], false) as f32);
        }
    }
}

/// Sum per-feature scores for matching this MASM column against one position of a Mega profile.
#[track_caller]
pub fn masm_col_get_match_score_mega_profile_pos(col: &MASMCol, prof_pos: &[byte]) -> f32 {
    let masm = col
        .masm
        .as_ref()
        .expect("MASMCol::GetMatchScore_MegaProfilePos, m_MASM is null");
    let feature_count = prof_pos.len();
    assert_eq!(feature_count, masm.feature_count as usize);
    assert_eq!(col.scores_vec.len(), feature_count);
    let mut total = 0.0_f32;
    for (feature_idx, letter) in prof_pos.iter().enumerate() {
        total += col.scores_vec[feature_idx][*letter as usize];
    }
    total
}
