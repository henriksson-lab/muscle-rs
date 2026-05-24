// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MegaState {
    pub file_name: String,
    pub lines: Vec<String>,
    pub feature_names: Vec<String>,
    pub weights: Vec<f32>,
    pub alpha_sizes: Vec<uint>,
    pub labels: Vec<String>,
    pub profiles: Vec<Vec<Vec<byte>>>,
    pub seqs: Vec<String>,
    pub log_probs_vec: Vec<Vec<f32>>,
    pub log_prob_mx_vec: Vec<Vec<Vec<f32>>>,
    pub log_odds_mx_vec: Vec<Vec<Vec<f32>>>,
    pub next_line_nr: uint,
    pub feature_count: uint,
    pub loaded: bool,
    pub gap_open: f32,
    pub gap_ext: f32,
    pub label_to_idx: std::collections::HashMap<String, uint>,
    pub seq_to_idx: std::collections::HashMap<String, uint>,
}

pub static MEGA_STATE: std::sync::LazyLock<std::sync::Mutex<MegaState>> =
    std::sync::LazyLock::new(|| {
        std::sync::Mutex::new(MegaState {
            file_name: String::new(),
            lines: Vec::new(),
            feature_names: Vec::new(),
            weights: Vec::new(),
            alpha_sizes: Vec::new(),
            labels: Vec::new(),
            profiles: Vec::new(),
            seqs: Vec::new(),
            log_probs_vec: Vec::new(),
            log_prob_mx_vec: Vec::new(),
            log_odds_mx_vec: Vec::new(),
            next_line_nr: 0,
            feature_count: 0,
            loaded: false,
            gap_open: f32::MAX,
            gap_ext: f32::MAX,
            label_to_idx: std::collections::HashMap::new(),
            seq_to_idx: std::collections::HashMap::new(),
        })
    });

pub static MEGA_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MEGA_DUPLICATE_SEQUENCE_WARNING_DONE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[derive(Clone, Debug, Default)]
pub struct Mega; // original: Mega (muscle/src/mega.h)

#[derive(Clone, Debug, Default)]
pub struct MegaScoringSnapshot {
    pub weights: Vec<f32>,
    pub log_probs_vec: Vec<Vec<f32>>,
    pub log_prob_mx_vec: Vec<Vec<Vec<f32>>>,
    pub log_odds_mx_vec: Vec<Vec<Vec<f32>>>,
}

#[doc(hidden)]
pub fn mega_reset_duplicate_sequence_warning_for_tests() {
    MEGA_DUPLICATE_SEQUENCE_WARNING_DONE.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// Return the global sequence index for a label, panicking if not found.
#[track_caller]
pub fn mega_get_gsi_by_label(label: &str) -> uint {
    let mega = MEGA_STATE.lock().unwrap();
    *mega
        .label_to_idx
        .get(label)
        .unwrap_or_else(|| panic!("Mega::GetGSIByLabel({label})"))
}

/// Return the label of the profile at the given global sequence index.
#[track_caller]
pub fn mega_get_label_by_gsi(gsi: uint) -> String {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = gsi as usize;
    assert!(idx < mega.labels.len());
    mega.labels[idx].clone()
}

/// Return a clone of the Mega profile at the given global sequence index.
#[track_caller]
pub fn mega_get_profile_by_gsi(gsi: uint) -> Vec<Vec<byte>> {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = gsi as usize;
    assert!(idx < mega.profiles.len());
    mega.profiles[idx].clone()
}

/// Look up a Mega profile by label, panicking if missing.
#[track_caller]
pub fn mega_get_profile_by_label(label: &str) -> Vec<Vec<byte>> {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = *mega
        .label_to_idx
        .get(label)
        .unwrap_or_else(|| panic!("Mega::GetProfileByLabel({label})")) as usize;
    assert!(idx < mega.profiles.len());
    mega.profiles[idx].clone()
}

#[track_caller]
pub fn mega_with_profiles_by_label<R, F>(label_x: &str, label_y: &str, f: F) -> R
where
    F: FnOnce(&[Vec<byte>], &[Vec<byte>]) -> R,
{
    let (profile_x_addr, profile_y_addr) = {
        let mega = MEGA_STATE.lock().unwrap();
        let idx_x = *mega
            .label_to_idx
            .get(label_x)
            .unwrap_or_else(|| panic!("Mega::GetProfileByLabel({label_x})"))
            as usize;
        let idx_y = *mega
            .label_to_idx
            .get(label_y)
            .unwrap_or_else(|| panic!("Mega::GetProfileByLabel({label_y})"))
            as usize;
        assert!(idx_x < mega.profiles.len());
        assert!(idx_y < mega.profiles.len());
        (
            &mega.profiles[idx_x] as *const Vec<Vec<byte>> as usize,
            &mega.profiles[idx_y] as *const Vec<Vec<byte>> as usize,
        )
    };
    // SAFETY: Mega profile storage is process-global and treated as immutable
    // after `mega_from_file` publishes a fully parsed state. This avoids cloning
    // large profiles for each posterior pair while preserving the C++ global
    // profile-table access shape. Callers must not reload Mega concurrently.
    unsafe {
        f(
            &*(profile_x_addr as *const Vec<Vec<byte>>),
            &*(profile_y_addr as *const Vec<Vec<byte>>),
        )
    }
}

/// Look up a Mega profile by ungapped sequence, optionally panicking when missing.
#[track_caller]
pub fn mega_get_profile_by_seq(seq: &str, fail_on_error: bool) -> Option<Vec<Vec<byte>>> {
    let mega = MEGA_STATE.lock().unwrap();
    let Some(idx) = mega.seq_to_idx.get(seq) else {
        if fail_on_error {
            let prefix: String = seq.chars().take(16).collect();
            panic!("Mega::GetProfileBySeq({prefix}...)");
        }
        return None;
    };
    let idx = *idx as usize;
    assert!(idx < mega.profiles.len());
    Some(mega.profiles[idx].clone())
}

#[track_caller]
pub fn mega_get_scoring_snapshot() -> MegaScoringSnapshot {
    let mega = MEGA_STATE.lock().unwrap();
    MegaScoringSnapshot {
        weights: mega.weights.clone(),
        log_probs_vec: mega.log_probs_vec.clone(),
        log_prob_mx_vec: mega.log_prob_mx_vec.clone(),
        log_odds_mx_vec: mega.log_odds_mx_vec.clone(),
    }
}

/// Consume and return the next line from the Mega input buffer.
#[track_caller]
pub fn mega_get_next_line() -> String {
    let mut mega = MEGA_STATE.lock().unwrap();
    let idx = mega.next_line_nr as usize;
    assert!(idx < mega.lines.len());
    mega.next_line_nr += 1;
    mega.lines[idx].clone()
}

/// Consume the next Mega line, split on tabs, and assert the expected field count.
#[track_caller]
pub fn mega_get_next_fields(expected_nr_fields: uint) -> Vec<String> {
    let line = mega_get_next_line();
    let fields = split(&line, '\t');
    if expected_nr_fields != uint::MAX && fields.len() != expected_nr_fields as usize {
        panic!(
            "Expected {} fields got {} in '{}'",
            expected_nr_fields,
            fields.len(),
            line
        );
    }
    fields
}

/// Assert that a square matrix is symmetric to within a 1% relative tolerance.
#[track_caller]
pub fn mega_assert_symmetrical(mx: &[Vec<f32>]) {
    let n = mx.len();
    for i in 0..n {
        assert_eq!(mx[i].len(), n);
        for j in 0..i {
            let x = mx[i][j] as f64;
            let y = mx[j][i] as f64;
            if x != y {
                let max = x.abs().max(y.abs());
                let diff = (x.abs() - y.abs()).abs();
                assert!(diff < max * 0.01);
            }
        }
    }
}

/// Compute per-letter marginal frequencies (row sums) from a joint frequency matrix.
#[track_caller]
pub fn mega_calc_marginal_freqs(freqs_mx: &[Vec<f32>]) -> Vec<f32> {
    let mut marginal_freqs = Vec::new();
    mega_assert_symmetrical(freqs_mx);
    let n = freqs_mx.len();
    let mut sum_marginal_freqs = 0.0_f32;
    for row in freqs_mx.iter().take(n) {
        let mut marginal_freq = 0.0_f32;
        for value in row.iter().take(n) {
            marginal_freq += *value;
        }
        marginal_freqs.push(marginal_freq);
        sum_marginal_freqs += marginal_freq;
    }
    if sum_marginal_freqs != 1.0 {
        let max = sum_marginal_freqs.abs().max(1.0);
        let diff = (sum_marginal_freqs.abs() - 1.0_f32).abs();
        assert!(diff < max * 0.01);
    }
    marginal_freqs
}

/// Load the global Mega state (features, weights, profiles, log-odds matrices) from a file.
#[track_caller]
pub fn mega_from_file(file_name: &str) {
    if file_name.is_empty() {
        die("Missing mega filename");
    }
    {
        let mega = MEGA_STATE.lock().unwrap();
        assert!(mega.feature_names.is_empty());
        assert_eq!(mega.feature_count, 0);
        assert!(mega.profiles.is_empty());
    }

    let mut local = MegaState {
        loaded: true,
        file_name: file_name.to_string(),
        lines: std::fs::read_to_string(file_name)
            .unwrap_or_else(|e| panic!("OpenStdioFile({file_name}): {e}"))
            .lines()
            .map(|line| line.to_string())
            .collect(),
        ..MegaState::default()
    };

    let get_next_fields = |mega: &mut MegaState, expected_nr_fields: uint| -> Vec<String> {
        let idx = mega.next_line_nr as usize;
        assert!(idx < mega.lines.len());
        mega.next_line_nr += 1;
        let line = mega.lines[idx].clone();
        let fields = split(&line, '\t');
        if expected_nr_fields != uint::MAX && fields.len() != expected_nr_fields as usize {
            panic!(
                "Expected {} fields got {} in '{}'",
                expected_nr_fields,
                fields.len(),
                line
            );
        }
        fields
    };

    let flds = get_next_fields(&mut local, 5);
    assert_eq!(flds[0], "mega");
    let feature_count = str_to_uint_l1278(&flds[1], false);
    let profile_count = str_to_uint_l1278(&flds[2], false);
    local.feature_count = feature_count;
    local.gap_open = str_to_float_l1204(&flds[3], false) as f32;
    local.gap_ext = str_to_float_l1204(&flds[4], false) as f32;
    local
        .log_probs_vec
        .resize(feature_count as usize, Vec::new());
    local
        .log_prob_mx_vec
        .resize(feature_count as usize, Vec::new());
    local
        .log_odds_mx_vec
        .resize(feature_count as usize, Vec::new());

    for feature_idx in 0..feature_count {
        let flds = get_next_fields(&mut local, 4);
        assert_eq!(str_to_uint_l1278(&flds[0], false), feature_idx);
        let feature_name = flds[1].clone();
        let alpha_size = str_to_uint_l1278(&flds[2], false);
        let weight = str_to_float_l1204(&flds[3], false) as f32;
        local.feature_names.push(feature_name);
        local.alpha_sizes.push(alpha_size);
        local.weights.push(weight);

        let flds = get_next_fields(&mut local, alpha_size + 1);
        assert_eq!(flds[0], "freqs");
        let mut log_probs = Vec::new();
        for letter in 0..alpha_size as usize {
            let mut prob = str_to_float_l1204(&flds[letter + 1], false) as f32;
            if prob < 1e-6 {
                prob = 1e-6;
            }
            log_probs.push(prob.ln());
        }
        local.log_probs_vec[feature_idx as usize] = log_probs;

        let mut log_prob_mx = vec![vec![0.0_f32; alpha_size as usize]; alpha_size as usize];
        for letter1 in 0..alpha_size {
            let flds = get_next_fields(&mut local, letter1 + 2);
            assert_eq!(str_to_uint_l1278(&flds[0], false), letter1);
            for letter2 in 0..=letter1 {
                let mut prob = str_to_float_l1204(&flds[letter2 as usize + 1], false) as f32;
                if prob < 1e-6 {
                    prob = 1e-6;
                }
                let log_prob = prob.ln();
                log_prob_mx[letter1 as usize][letter2 as usize] = log_prob;
                log_prob_mx[letter2 as usize][letter1 as usize] = log_prob;
            }
        }
        local.log_prob_mx_vec[feature_idx as usize] = log_prob_mx;

        let flds = get_next_fields(&mut local, 1);
        assert_eq!(flds[0], "logoddsmx");
        let mut log_odds_mx = vec![vec![0.0_f32; alpha_size as usize]; alpha_size as usize];
        for letter1 in 0..alpha_size {
            let flds = get_next_fields(&mut local, letter1 + 3);
            assert_eq!(str_to_uint_l1278(&flds[0], false), letter1);
            let letter_str = &flds[1];
            assert_eq!(letter_str.len(), 1);
            assert!(letter_str.as_bytes()[0].is_ascii_alphabetic());
            for letter2 in 0..=letter1 {
                let score = str_to_float_l1204(&flds[letter2 as usize + 2], false) as f32;
                log_odds_mx[letter1 as usize][letter2 as usize] = score;
                log_odds_mx[letter2 as usize][letter1 as usize] = score;
            }
        }
        local.log_odds_mx_vec[feature_idx as usize] = log_odds_mx;
    }

    local.profiles.resize(profile_count as usize, Vec::new());
    local.seqs.resize(profile_count as usize, String::new());

    for profile_idx in 0..profile_count {
        let flds = get_next_fields(&mut local, 4);
        assert_eq!(flds[0], "chain");
        assert_eq!(str_to_uint_l1278(&flds[1], false), profile_idx);
        let label = flds[2].clone();
        if local.label_to_idx.contains_key(&label) {
            die(&format!("Duplicate label in mega file >{label}"));
        }
        local.label_to_idx.insert(label.clone(), profile_idx);
        local.labels.push(label);
        let l = str_to_uint_l1278(&flds[3], false);
        let mut profile = vec![Vec::<byte>::new(); l as usize];
        let mut s = String::new();
        for pos in 0..l {
            let flds = get_next_fields(&mut local, 3);
            assert_eq!(str_to_uint_l1278(&flds[0], false), profile_idx);
            assert_eq!(str_to_uint_l1278(&flds[1], false), pos);
            let syms = flds[2].as_bytes();
            assert_eq!(syms.len(), feature_count as usize);
            for feature_idx in 0..feature_count {
                let sym = syms[feature_idx as usize];
                if feature_idx == 0 {
                    let mut letter = match (sym as char).to_ascii_uppercase() {
                        'A' => 0,
                        'C' => 1,
                        'D' => 2,
                        'E' => 3,
                        'F' => 4,
                        'G' => 5,
                        'H' => 6,
                        'I' => 7,
                        'K' => 8,
                        'L' => 9,
                        'M' => 10,
                        'N' => 11,
                        'P' => 12,
                        'Q' => 13,
                        'R' => 14,
                        'S' => 15,
                        'T' => 16,
                        'V' => 17,
                        'W' => 18,
                        'Y' => 19,
                        _ => uint::MAX,
                    };
                    if letter >= 20 {
                        letter = 0;
                    }
                    profile[pos as usize].push(letter as byte);
                    s.push(sym as char);
                } else {
                    let letter = uint::from(sym - b'A');
                    assert!(letter < 16);
                    profile[pos as usize].push(letter as byte);
                }
            }
        }
        local.profiles[profile_idx as usize] = profile;
        local.seqs[profile_idx as usize] = s.clone();
        if local.seq_to_idx.contains_key(&s)
            && !MEGA_DUPLICATE_SEQUENCE_WARNING_DONE
                .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            warning("Duplicate sequences found\n");
        }
        local.seq_to_idx.insert(s, profile_idx);
    }
    local.lines.clear();
    *MEGA_STATE.lock().unwrap() = local;
    MEGA_LOADED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Return the weighted log-probability insertion score at one profile position.
#[track_caller]
pub fn mega_get_ins_score(profile: &[Vec<byte>], pos: uint) -> f32 {
    assert!((pos as usize) < profile.len());
    let prof_col = &profile[pos as usize];
    let mega = MEGA_STATE.lock().unwrap();
    let mut score = 0.0_f32;
    for i in 0..mega.feature_count as usize {
        let log_probs = &mega.log_probs_vec[i];
        let letter = prof_col[i] as usize;
        score += log_probs[letter] * mega.weights[i];
    }
    score
}

#[inline(always)]
pub fn mega_scoring_get_ins_score(
    scoring: &MegaScoringSnapshot,
    profile: &[Vec<byte>],
    pos: uint,
) -> f32 {
    assert!((pos as usize) < profile.len());
    let prof_col = &profile[pos as usize];
    let mut score = 0.0_f32;
    for i in 0..scoring.weights.len() {
        let log_probs = &scoring.log_probs_vec[i];
        let letter = prof_col[i] as usize;
        score += log_probs[letter] * scoring.weights[i];
    }
    score
}

/// Return the name of the feature at the given index.
#[track_caller]
pub fn mega_get_feature_name(feature_index: uint) -> String {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = feature_index as usize;
    assert!(idx < mega.feature_names.len());
    mega.feature_names[idx].clone()
}

/// Return the alphabet size of the given feature.
#[track_caller]
pub fn mega_get_alpha_size(feature_index: uint) -> uint {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = feature_index as usize;
    assert!(idx < mega.alpha_sizes.len());
    mega.alpha_sizes[idx]
}

/// Return the scoring weight of the given feature.
#[track_caller]
pub fn mega_get_weight(feature_index: uint) -> f32 {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = feature_index as usize;
    assert!(idx < mega.weights.len());
    mega.weights[idx]
}

/// Return the label of the profile at the given index.
#[track_caller]
pub fn mega_get_label(profile_idx: uint) -> String {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = profile_idx as usize;
    assert!(idx < mega.profiles.len());
    mega.labels[idx].clone()
}

/// Return a clone of the Mega profile at the given index.
#[track_caller]
pub fn mega_get_profile(profile_idx: uint) -> Vec<Vec<byte>> {
    let mega = MEGA_STATE.lock().unwrap();
    let idx = profile_idx as usize;
    assert!(idx < mega.profiles.len());
    mega.profiles[idx].clone()
}

/// Weighted log-odds match score between one position of two Mega profiles.
#[track_caller]
pub fn mega_get_match_score_log_odds(
    profile_x: &[Vec<byte>],
    pos_x: uint,
    profile_y: &[Vec<byte>],
    pos_y: uint,
) -> f32 {
    assert!((pos_x as usize) < profile_x.len());
    assert!((pos_y as usize) < profile_y.len());
    let prof_col_x = &profile_x[pos_x as usize];
    let prof_col_y = &profile_y[pos_y as usize];
    let mega = MEGA_STATE.lock().unwrap();
    let mut score = 0.0_f32;
    for i in 0..mega.feature_count as usize {
        let subst_mx = &mega.log_odds_mx_vec[i];
        let letter_x = prof_col_x[i] as usize;
        let letter_y = prof_col_y[i] as usize;
        let letter_pair_score = subst_mx[letter_x][letter_y];
        score += letter_pair_score * mega.weights[i];
    }
    score
}

/// Weighted log-probability match score between one position of two Mega profiles.
#[track_caller]
pub fn mega_get_match_score(
    profile_x: &[Vec<byte>],
    pos_x: uint,
    profile_y: &[Vec<byte>],
    pos_y: uint,
) -> f32 {
    assert!((pos_x as usize) < profile_x.len());
    assert!((pos_y as usize) < profile_y.len());
    let prof_col_x = &profile_x[pos_x as usize];
    let prof_col_y = &profile_y[pos_y as usize];
    let mega = MEGA_STATE.lock().unwrap();
    let mut score = 0.0_f32;
    for i in 0..mega.feature_count as usize {
        let subst_mx = &mega.log_prob_mx_vec[i];
        let letter_x = prof_col_x[i] as usize;
        let letter_y = prof_col_y[i] as usize;
        let letter_pair_score = subst_mx[letter_x][letter_y];
        score += letter_pair_score * mega.weights[i];
    }
    score
}

#[inline(always)]
pub fn mega_scoring_get_match_score(
    scoring: &MegaScoringSnapshot,
    profile_x: &[Vec<byte>],
    pos_x: uint,
    profile_y: &[Vec<byte>],
    pos_y: uint,
) -> f32 {
    assert!((pos_x as usize) < profile_x.len());
    assert!((pos_y as usize) < profile_y.len());
    let prof_col_x = &profile_x[pos_x as usize];
    let prof_col_y = &profile_y[pos_y as usize];
    let mut score = 0.0_f32;
    for i in 0..scoring.weights.len() {
        let subst_mx = &scoring.log_prob_mx_vec[i];
        let letter_x = prof_col_x[i] as usize;
        let letter_y = prof_col_y[i] as usize;
        let letter_pair_score = subst_mx[letter_x][letter_y];
        score += letter_pair_score * scoring.weights[i];
    }
    score
}

/// Format a labelled float vector as a debug log string.
#[track_caller]
pub fn mega_log_vec(name: &str, vec_: &[f32]) -> String {
    let n = vec_.len();
    let mut out = format!("\n{name}/{n}");
    for (i, value) in vec_.iter().enumerate() {
        if i % 10 == 0 {
            out.push_str("\n  ");
        } else {
            out.push(' ');
        }
        out.push_str(&format!("[{i:2}]={value:.2}"));
    }
    out.push('\n');
    log(&out);
    out
}

/// Format a labelled float matrix as a debug log string.
#[track_caller]
pub fn mega_log_mx(name: &str, mx: &[Vec<f32>]) -> String {
    let n = mx.len();
    let mut out = String::new();
    out.push_str(&format!("\n{name}/{n}\n"));
    out.push_str("     ");
    for j in 0..n {
        out.push_str(&format!(" {j:7}"));
    }
    out.push('\n');
    for i in 0..n {
        out.push_str(&format!("[{i:2}] "));
        let row = &mx[i];
        assert_eq!(row.len(), n);
        for value in row.iter().take(n) {
            out.push_str(&format!(" {value:7.2}"));
        }
        out.push('\n');
    }
    log(&out);
    out
}

/// Dump the name, weight, log-probs and log-prob matrix of one feature for logging.
#[track_caller]
pub fn mega_log_feature_params(idx: uint) -> String {
    let (name, weight, probs, mx) = {
        let mega = MEGA_STATE.lock().unwrap();
        let idx = idx as usize;
        assert!(idx < mega.feature_names.len());
        assert!(idx < mega.log_prob_mx_vec.len());
        assert!(idx < mega.log_probs_vec.len());
        (
            mega.feature_names[idx].clone(),
            mega.weights[idx],
            mega.log_probs_vec[idx].clone(),
            mega.log_prob_mx_vec[idx].clone(),
        )
    };
    let mut out = String::new();
    out.push('\n');
    let weight_s = if weight == 0.0 {
        "0".to_string()
    } else if !weight.is_finite() {
        weight.to_string()
    } else {
        let weight64 = f64::from(weight);
        let exp = weight64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{weight64:.2e}");
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
            format!("{weight64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    out.push_str(&format!("Feature {name}, weight {weight_s}\n"));
    log(&out);
    out.push_str(&mega_log_vec(&name, &probs));
    out.push_str(&mega_log_mx(&name, &mx));
    out
}

/// Return the index of the AA feature, panicking if absent.
#[track_caller]
pub fn mega_get_aa_feature_idx() -> uint {
    let mega = MEGA_STATE.lock().unwrap();
    for feature_idx in 0..mega.feature_names.len() {
        if mega.feature_names[feature_idx] == "AA" {
            return feature_idx as uint;
        }
    }
    panic!("Mega::GetAAFeatureIdx(), not found");
}

/// Initialise the global Mega state from an MSA using only the AA feature and BLOSUM62.
#[track_caller]
pub fn mega_from_msa_aa_only(aln: &MultiSequence, gap_open: f32, gap_ext: f32) {
    let mut mega = MEGA_STATE.lock().unwrap();
    mega.file_name = "FromMSA_AAOnly()".to_string();
    mega.lines.clear();

    mega.feature_names.clear();
    mega.feature_names.push("AA".to_string());

    mega.weights.clear();
    mega.weights.push(1.0);

    mega.alpha_sizes.clear();
    mega.alpha_sizes.push(20);
    mega.feature_count = 1;

    mega.label_to_idx.clear();
    mega.seq_to_idx.clear();
    mega.labels.clear();
    mega.seqs.clear();
    mega.profiles.clear();
    let seq_count = aln.seqs.len();
    mega.profiles.resize(seq_count, Vec::new());

    for seq_idx in 0..seq_count {
        let label = aln.seqs[seq_idx].label.clone();
        mega.labels.push(label.clone());
        let mut ungapped_seq = String::new();
        for &c in &aln.seqs[seq_idx].char_vec {
            if c != '-' && c != '.' {
                ungapped_seq.push(c);
            }
        }
        mega.seqs.push(ungapped_seq.clone());
        mega.label_to_idx.insert(label, seq_idx as uint);
        mega.seq_to_idx
            .insert(ungapped_seq.clone(), seq_idx as uint);

        let profile = &mut mega.profiles[seq_idx];
        for c in ungapped_seq.bytes() {
            let mut letter = match (c as char).to_ascii_uppercase() {
                'A' => 0,
                'C' => 1,
                'D' => 2,
                'E' => 3,
                'F' => 4,
                'G' => 5,
                'H' => 6,
                'I' => 7,
                'K' => 8,
                'L' => 9,
                'M' => 10,
                'N' => 11,
                'P' => 12,
                'Q' => 13,
                'R' => 14,
                'S' => 15,
                'T' => 16,
                'V' => 17,
                'W' => 18,
                'Y' => 19,
                _ => uint::MAX,
            };
            if letter >= 20 {
                letter = 0;
            }
            let mut col = Vec::new();
            col.push(letter as byte);
            profile.push(col);
        }
    }

    mega.log_probs_vec.clear();
    mega.log_prob_mx_vec.clear();
    mega.log_odds_mx_vec.clear();
    mega.log_odds_mx_vec.resize(1, Vec::new());
    mega.log_odds_mx_vec[0] = get_blosum62_log_odds_letter_mx();
    mega.gap_open = gap_open;
    mega.gap_ext = gap_ext;
    mega.loaded = true;
    MEGA_LOADED.store(true, std::sync::atomic::Ordering::Relaxed);
}
