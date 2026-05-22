// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

pub const HMMTRANS_N: usize = 10;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum HMMTRANS {
    #[default]
    HMMTRANS_START_M = 0,
    HMMTRANS_START_IS = 1,
    HMMTRANS_START_IL = 2,
    HMMTRANS_M_M = 3,
    HMMTRANS_M_IS = 4,
    HMMTRANS_M_IL = 5,
    HMMTRANS_IS_IS = 6,
    HMMTRANS_IS_M = 7,
    HMMTRANS_IL_IL = 8,
    HMMTRANS_IL_M = 9,
} // original: HMMTRANS (muscle/src/hmmparams.h)

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HMMParams {
    pub logs: bool,
    pub line_nr: uint,
    pub var: f32,
    pub trans: Vec<f32>,
    pub emits: Vec<Vec<f32>>,
    pub lines: Vec<String>,
    pub alpha: String,
} // original: HMMParams (muscle/src/hmmparams.h)

/// Process-global `-anchor_letter` override consumed by `hmm_params_to_pair_hmm`.
pub(crate) static HMM_PARAMS_ANCHOR_LETTER: std::sync::Mutex<Option<u8>> =
    std::sync::Mutex::new(None);

/// Sets the optional anchor letter used to force one match score to -0.1.
#[track_caller]
pub fn set_hmm_params_anchor_letter(anchor_letter: Option<&str>) {
    let value = anchor_letter.map(|s| {
        assert_eq!(s.len(), 1);
        s.as_bytes()[0]
    });
    *HMM_PARAMS_ANCHOR_LETTER.lock().unwrap() = value;
}

/// Compatibility wrapper for the C++ `anchor_letter` option name.
#[track_caller]
pub fn set_anchor_letter(anchor_letter: Option<&str>) {
    set_hmm_params_anchor_letter(anchor_letter);
}

/// Returns the textual name (e.g. `"START_M"`) for a transition variant.
pub fn hmmtrans_to_str(t: HMMTRANS) -> &'static str {
    match t {
        HMMTRANS::HMMTRANS_START_M => "START_M",
        HMMTRANS::HMMTRANS_START_IS => "START_IS",
        HMMTRANS::HMMTRANS_START_IL => "START_IL",
        HMMTRANS::HMMTRANS_M_M => "M_M",
        HMMTRANS::HMMTRANS_M_IS => "M_IS",
        HMMTRANS::HMMTRANS_M_IL => "M_IL",
        HMMTRANS::HMMTRANS_IS_IS => "IS_IS",
        HMMTRANS::HMMTRANS_IS_M => "IS_M",
        HMMTRANS::HMMTRANS_IL_IL => "IL_IL",
        HMMTRANS::HMMTRANS_IL_M => "IL_M",
    }
}

/// Returns `params` as probabilities, converting from log scores if needed.
#[track_caller]
pub fn hmm_params_get_probs(params: &HMMParams) -> HMMParams {
    let probs = if params.logs {
        hmm_params_scores_to_probs(params)
    } else {
        params.clone()
    };
    hmm_params_assert_probs_valid(&probs);
    probs
}

/// Converts `params` to either probability or score representation.
#[track_caller]
pub fn hmm_params_from_params(params: &HMMParams, as_probs: bool) -> HMMParams {
    if as_probs {
        hmm_params_get_probs(params)
    } else {
        hmm_params_get_scores(params)
    }
}

/// Returns `params` as log scores, converting from probabilities if needed.
#[track_caller]
pub fn hmm_params_get_scores(params: &HMMParams) -> HMMParams {
    if params.logs {
        params.clone()
    } else {
        hmm_params_assert_probs_valid(params);
        hmm_params_probs_to_scores(params)
    }
}

/// Collapses long/short-gap probabilities to a single affine-gap pair.
#[track_caller]
pub fn hmm_params_to_single_affine_probs(params: &HMMParams) -> HMMParams {
    let mut params = hmm_params_get_probs(params);
    let t = &mut params.trans;

    let si =
        (t[HMMTRANS::HMMTRANS_START_IS as usize] + t[HMMTRANS::HMMTRANS_START_IL as usize]) / 2.0;
    let mi = (t[HMMTRANS::HMMTRANS_M_IS as usize] + t[HMMTRANS::HMMTRANS_M_IL as usize]) / 2.0;
    let im = (t[HMMTRANS::HMMTRANS_IS_M as usize] + t[HMMTRANS::HMMTRANS_IL_M as usize]) / 2.0;
    let ii = (t[HMMTRANS::HMMTRANS_IS_IS as usize] + t[HMMTRANS::HMMTRANS_IL_IL as usize]) / 2.0;

    t[HMMTRANS::HMMTRANS_START_IS as usize] = si;
    t[HMMTRANS::HMMTRANS_START_IL as usize] = si;
    t[HMMTRANS::HMMTRANS_M_IS as usize] = mi;
    t[HMMTRANS::HMMTRANS_M_IL as usize] = mi;
    t[HMMTRANS::HMMTRANS_IS_M as usize] = im;
    t[HMMTRANS::HMMTRANS_IL_M as usize] = im;
    t[HMMTRANS::HMMTRANS_IS_IS as usize] = ii;
    t[HMMTRANS::HMMTRANS_IL_IL as usize] = ii;

    hmm_params_assert_probs_valid(&params);
    params
}

/// Exponentiates log-score parameters into probability parameters.
#[track_caller]
pub fn hmm_params_scores_to_probs(scores: &HMMParams) -> HMMParams {
    let alpha_size = scores.alpha.len();
    assert!(alpha_size == 4 || alpha_size == 20);

    let mut probs = HMMParams {
        logs: false,
        var: DEFAULT_PERTURB_VAR,
        alpha: scores.alpha.clone(),
        trans: vec![0.0; HMMTRANS_N],
        emits: vec![vec![0.0; alpha_size]; alpha_size],
        ..HMMParams::default()
    };

    for i in 0..HMMTRANS_N {
        probs.trans[i] = scores.trans[i].exp();
    }
    for i in 0..alpha_size {
        for j in 0..alpha_size {
            probs.emits[i][j] = scores.emits[i][j].exp();
        }
    }

    hmm_params_assert_probs_valid(&probs);
    probs
}

/// Takes natural logs of probability parameters to produce log-score parameters.
#[track_caller]
pub fn hmm_params_probs_to_scores(probs: &HMMParams) -> HMMParams {
    hmm_params_assert_probs_valid(probs);
    let alpha_size = probs.alpha.len();

    let mut scores = HMMParams {
        logs: true,
        var: DEFAULT_PERTURB_VAR,
        alpha: probs.alpha.clone(),
        trans: vec![0.0; HMMTRANS_N],
        emits: vec![vec![f32::MAX; alpha_size]; alpha_size],
        ..HMMParams::default()
    };

    for i in 0..HMMTRANS_N {
        scores.trans[i] = probs.trans[i].ln();
    }
    for i in 0..alpha_size {
        for j in 0..alpha_size {
            scores.emits[i][j] = probs.emits[i][j].ln();
        }
    }
    scores
}

/// Panics unless `params` holds a valid probability distribution (sums to 1, symmetric emits).
#[track_caller]
pub fn hmm_params_assert_probs_valid(params: &HMMParams) {
    assert!(!params.logs);
    assert_eq!(params.trans.len(), HMMTRANS_N);
    let alpha_size = params.alpha.len();
    assert!(alpha_size == 4 || alpha_size == 20);
    assert_eq!(params.emits.len(), alpha_size);
    for i in 0..alpha_size {
        assert_eq!(params.emits[i].len(), alpha_size);
    }

    let sum_start = params.trans[HMMTRANS::HMMTRANS_START_M as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_START_IS as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_START_IL as usize];
    assert!(myfeq(sum_start as f64, 1.0));

    let sum_is = params.trans[HMMTRANS::HMMTRANS_IS_M as usize]
        + params.trans[HMMTRANS::HMMTRANS_IS_IS as usize];
    assert!(myfeq(sum_is as f64, 1.0));

    let sum_il = params.trans[HMMTRANS::HMMTRANS_IL_M as usize]
        + params.trans[HMMTRANS::HMMTRANS_IL_IL as usize];
    assert!(myfeq(sum_il as f64, 1.0));

    let sum_m = params.trans[HMMTRANS::HMMTRANS_M_M as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IS as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IL as usize];
    assert!(myfeq(sum_m as f64, 1.0));

    let mut sum_emit = 0.0;
    for i in 0..alpha_size {
        for j in 0..alpha_size {
            sum_emit += params.emits[i][j];
        }
    }
    assert!(myfeq(sum_emit as f64, 1.0));

    for i in 0..alpha_size {
        for j in 0..i {
            assert!(myfeq(params.emits[i][j] as f64, params.emits[j][i] as f64));
        }
    }
}

/// Serializes HMM probabilities to a tab-separated text file (returns the content).
#[track_caller]
pub fn hmm_params_to_file(params: &HMMParams, file_name: &str) -> Option<String> {
    if file_name.is_empty() {
        return None;
    }

    hmm_params_assert_probs_valid(params);
    let alpha_size = params.alpha.len();
    let mut out = String::new();
    if params.alpha == AMINO_ALPHA {
        out.push_str("HMM\taa\n");
    } else if params.alpha == NT_ALPHA {
        out.push_str("HMM\tnt\n");
    } else {
        panic!("HMMParams::ToFile alpha='{}'", params.alpha);
    }

    let format_g5 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 5 {
            let raw = format!("{d64:.4e}");
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
            let decimals = (4 - exp).max(0) as usize;
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

    for t in [
        HMMTRANS::HMMTRANS_START_M,
        HMMTRANS::HMMTRANS_START_IS,
        HMMTRANS::HMMTRANS_START_IL,
        HMMTRANS::HMMTRANS_M_M,
        HMMTRANS::HMMTRANS_M_IS,
        HMMTRANS::HMMTRANS_M_IL,
        HMMTRANS::HMMTRANS_IS_IS,
        HMMTRANS::HMMTRANS_IS_M,
        HMMTRANS::HMMTRANS_IL_IL,
        HMMTRANS::HMMTRANS_IL_M,
    ] {
        let p = format_g5(params.trans[t as usize]);
        out.push_str(&format!("T.{}\t{}\n", hmmtrans_to_str(t), p));
    }

    let alpha = params.alpha.as_bytes();
    for i in 0..alpha_size {
        let a = alpha[i] as char;
        for j in 0..=i {
            let b = alpha[j] as char;
            let p = format_g5(params.emits[i][j]);
            out.push_str(&format!("E.{a}{b}\t{p}\n"));
        }
    }

    std::fs::write(file_name, out.as_bytes()).expect("failed to write HMM params file");
    Some(out)
}

/// Reads the next `name\tprob` line during HMM-file parsing.
#[track_caller]
pub fn hmm_params_get_next_prob(params: &mut HMMParams, name: &str) -> f32 {
    let line = hmm_params_get_next_line(params)
        .unwrap_or_else(|| panic!("GetNextProb({name}) end-of-data"));
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() != 2 {
        panic!("GetNextProb({name}) expected 2 fields got '{line}'");
    }
    if fields[0] != name {
        panic!("ReadGetNextProbTrans({name}) got '{line}'");
    }
    fields[1].parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "Invalid floating point value in HMM parameter line '{}'",
            line
        )
    })
}

/// Consumes and returns the next buffered line, or `None` at end of input.
#[track_caller]
pub fn hmm_params_get_next_line(params: &mut HMMParams) -> Option<String> {
    if (params.line_nr as usize) >= params.lines.len() {
        return None;
    }
    let line = params.lines[params.line_nr as usize].clone();
    params.line_nr += 1;
    Some(line)
}

/// Loads HMM parameters by reading and parsing the named text file.
#[track_caller]
pub fn hmm_params_from_file(file_name: &str) -> HMMParams {
    let text = std::fs::read_to_string(file_name)
        .unwrap_or_else(|e| panic!("Cannot open {file_name}, errno={:?} {e}", e.raw_os_error()));
    let lines = text.lines().map(|s| s.to_string()).collect::<Vec<_>>();
    hmm_params_from_strings(&lines)
}

/// Parses HMM parameters from a list of text lines.
#[track_caller]
pub fn hmm_params_from_strings(lines: &[String]) -> HMMParams {
    let mut params = HMMParams {
        lines: lines.to_vec(),
        line_nr: 0,
        var: DEFAULT_PERTURB_VAR,
        ..HMMParams::default()
    };

    let alpha_line = hmm_params_get_next_line(&mut params).expect("Invalid HMM file");
    let fields: Vec<&str> = alpha_line.split('\t').collect();
    if fields.len() != 2 || fields[0] != "HMM" {
        panic!("Invalid HMM file");
    }
    params.alpha = match fields[1] {
        "aa" => AMINO_ALPHA.to_string(),
        "nt" => NT_ALPHA.to_string(),
        other => panic!("Invalid HMM alphabet '{other}'"),
    };

    params.trans.clear();
    params.trans.resize(HMMTRANS_N, f32::MAX);
    for t in [
        HMMTRANS::HMMTRANS_START_M,
        HMMTRANS::HMMTRANS_START_IS,
        HMMTRANS::HMMTRANS_START_IL,
        HMMTRANS::HMMTRANS_M_M,
        HMMTRANS::HMMTRANS_M_IS,
        HMMTRANS::HMMTRANS_M_IL,
        HMMTRANS::HMMTRANS_IS_IS,
        HMMTRANS::HMMTRANS_IS_M,
        HMMTRANS::HMMTRANS_IL_IL,
        HMMTRANS::HMMTRANS_IL_M,
    ] {
        let name = format!("T.{}", hmmtrans_to_str(t));
        params.trans[t as usize] = hmm_params_get_next_prob(&mut params, &name);
    }

    let alpha_size = params.alpha.len();
    params.emits.clear();
    params.emits.resize(alpha_size, vec![f32::MAX; alpha_size]);

    let alpha_bytes = params.alpha.as_bytes().to_vec();
    for i in 0..alpha_size {
        let a = alpha_bytes[i] as char;
        for j in 0..=i {
            let b = alpha_bytes[j] as char;
            let name = format!("E.{a}{b}");
            let p = hmm_params_get_next_prob(&mut params, &name);
            params.emits[i][j] = p;
            params.emits[j][i] = p;
        }
    }

    hmm_params_normalize(&mut params);
    hmm_params_assert_probs_valid(&params);
    params
}

/// Loads the built-in default HMM parameters (amino or nucleotide).
#[track_caller]
pub fn hmm_params_from_defaults(nucleo: bool) -> HMMParams {
    let lines = hmm_params_get_default_hmm_params(nucleo);
    hmm_params_from_strings(&lines)
}

/// Overrides selected transitions from command-line options, then renormalizes.
#[track_caller]
pub fn hmm_params_cmd_line_update(
    params: &mut HMMParams,
    s_is: Option<f32>,
    s_il: Option<f32>,
    m_is: Option<f32>,
    m_il: Option<f32>,
    is_is: Option<f32>,
    il_il: Option<f32>,
) {
    if s_is.is_none()
        && s_il.is_none()
        && m_is.is_none()
        && m_il.is_none()
        && is_is.is_none()
        && il_il.is_none()
    {
        return;
    }
    assert!(!params.logs);
    if let Some(value) = s_is {
        params.trans[HMMTRANS::HMMTRANS_START_IS as usize] = value;
    }
    if let Some(value) = s_il {
        params.trans[HMMTRANS::HMMTRANS_START_IL as usize] = value;
    }
    if let Some(value) = m_is {
        params.trans[HMMTRANS::HMMTRANS_M_IS as usize] = value;
    }
    if let Some(value) = m_il {
        params.trans[HMMTRANS::HMMTRANS_M_IL as usize] = value;
    }
    if let Some(value) = is_is {
        params.trans[HMMTRANS::HMMTRANS_IS_IS as usize] = value;
    }
    if let Some(value) = il_il {
        params.trans[HMMTRANS::HMMTRANS_IL_IL as usize] = value;
    }
    hmm_params_normalize(params);
}

/// Installs `params` into the global pair-HMM score tables used by Viterbi/forward.
#[track_caller]
pub fn hmm_params_to_pair_hmm(params: &HMMParams) {
    let scores = hmm_params_get_scores(params);
    let probs = hmm_params_get_probs(params);
    let alpha_size = params.alpha.len();

    let mut insert_scores = Vec::new();
    let mut sum_inserts = 0.0;
    for i in 0..alpha_size {
        let mut marginal_prob = 0.0;
        for j in 0..alpha_size {
            marginal_prob += probs.emits[i][j];
        }
        insert_scores.push(marginal_prob.ln());
        sum_inserts += marginal_prob;
    }
    assert!(myfeq(sum_inserts as f64, 1.0));

    {
        let mut start_score = PAIR_HMM_START_SCORE.lock().unwrap();
        start_score[HMMSTATE_M as usize] = scores.trans[HMMTRANS::HMMTRANS_START_M as usize];
        start_score[HMMSTATE_IX as usize] = scores.trans[HMMTRANS::HMMTRANS_START_IS as usize];
        start_score[HMMSTATE_IY as usize] = scores.trans[HMMTRANS::HMMTRANS_START_IS as usize];
        start_score[HMMSTATE_JX as usize] = scores.trans[HMMTRANS::HMMTRANS_START_IL as usize];
        start_score[HMMSTATE_JY as usize] = scores.trans[HMMTRANS::HMMTRANS_START_IL as usize];
    }

    {
        let mut trans_score = PAIR_HMM_TRANS_SCORE.lock().unwrap();
        for i in 0..HMMSTATE_COUNT as usize {
            for j in 0..HMMSTATE_COUNT as usize {
                trans_score[i][j] = LOG_ZERO;
            }
        }
        trans_score[HMMSTATE_M as usize][HMMSTATE_M as usize] =
            scores.trans[HMMTRANS::HMMTRANS_M_M as usize];
        trans_score[HMMSTATE_M as usize][HMMSTATE_IX as usize] =
            scores.trans[HMMTRANS::HMMTRANS_M_IS as usize];
        trans_score[HMMSTATE_M as usize][HMMSTATE_IY as usize] =
            scores.trans[HMMTRANS::HMMTRANS_M_IS as usize];
        trans_score[HMMSTATE_M as usize][HMMSTATE_JX as usize] =
            scores.trans[HMMTRANS::HMMTRANS_M_IL as usize];
        trans_score[HMMSTATE_M as usize][HMMSTATE_JY as usize] =
            scores.trans[HMMTRANS::HMMTRANS_M_IL as usize];
        trans_score[HMMSTATE_IX as usize][HMMSTATE_IX as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IS_IS as usize];
        trans_score[HMMSTATE_IY as usize][HMMSTATE_IY as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IS_IS as usize];
        trans_score[HMMSTATE_JX as usize][HMMSTATE_JX as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IL_IL as usize];
        trans_score[HMMSTATE_JY as usize][HMMSTATE_JY as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IL_IL as usize];
        trans_score[HMMSTATE_IX as usize][HMMSTATE_M as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IS_M as usize];
        trans_score[HMMSTATE_IY as usize][HMMSTATE_M as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IS_M as usize];
        trans_score[HMMSTATE_JX as usize][HMMSTATE_M as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IL_M as usize];
        trans_score[HMMSTATE_JY as usize][HMMSTATE_M as usize] =
            scores.trans[HMMTRANS::HMMTRANS_IL_M as usize];
    }

    let wildcard_insert_prob = 1.0 / alpha_size as f32;
    {
        let mut ins_score = PAIR_HMM_INS_SCORE.lock().unwrap();
        for i in 0..256 {
            ins_score[i] = wildcard_insert_prob.ln();
        }
        for i in 0..alpha_size {
            let a = params.alpha.as_bytes()[i];
            let p = insert_scores[i];
            ins_score[a.to_ascii_lowercase() as usize] = p;
            ins_score[a.to_ascii_uppercase() as usize] = p;
        }
    }

    {
        let mut match_score = PAIR_HMM_MATCH_SCORE.write().unwrap();
        let wild_score = (wildcard_insert_prob * wildcard_insert_prob).ln();
        for i in 0..256 {
            for j in 0..256 {
                match_score[i][j] = wild_score;
            }
        }
        for i in 0..alpha_size {
            let a = params.alpha.as_bytes()[i];
            let ia = a.to_ascii_lowercase();
            let i_a = a.to_ascii_uppercase();
            for j in 0..alpha_size {
                let b = params.alpha.as_bytes()[j];
                let ib = b.to_ascii_lowercase();
                let i_b = b.to_ascii_uppercase();
                let p = scores.emits[i][j];
                match_score[ia as usize][ib as usize] = p;
                match_score[ia as usize][i_b as usize] = p;
                match_score[i_a as usize][ib as usize] = p;
                match_score[i_a as usize][i_b as usize] = p;
            }
        }
        if let Some(anchor_letter) = *HMM_PARAMS_ANCHOR_LETTER.lock().unwrap() {
            match_score[anchor_letter as usize][anchor_letter as usize] = -0.1;
        }
    }

    if alpha_size == 4 {
        pair_hmm_fix_ut();
    }
}

/// Copies the `T` row/column scores to `U` so RNA and DNA share parameters.
#[track_caller]
pub fn pair_hmm_fix_ut() {
    {
        let mut ins_score = PAIR_HMM_INS_SCORE.lock().unwrap();
        ins_score[b'U' as usize] = ins_score[b'T' as usize];
        ins_score[b'u' as usize] = ins_score[b't' as usize];
    }

    let mut match_score = PAIR_HMM_MATCH_SCORE.write().unwrap();
    for i in 0..256 {
        let p = match_score[b'T' as usize][i];
        match_score[b'U' as usize][i] = p;
        match_score[b'u' as usize][i] = p;
        match_score[i][b'U' as usize] = p;
        match_score[i][b'u' as usize] = p;
    }
}

/// Renormalizes the symmetric emission matrix so its entries sum to 1.
#[track_caller]
pub fn hmm_params_normalize_emit(params: &mut HMMParams) {
    assert!(!params.logs);
    let alpha_size = params.alpha.len();
    assert!(alpha_size == 4 || alpha_size == 20);

    let mut sum = 0.0;
    for i in 0..alpha_size {
        for j in 0..=i {
            let p = params.emits[i][j];
            params.emits[i][j] = p;
            params.emits[j][i] = p;
            sum += p;
            if i != j {
                sum += p;
            }
        }
    }

    for i in 0..alpha_size {
        for j in 0..alpha_size {
            params.emits[i][j] /= sum;
        }
    }
}

/// Renormalizes the START transition probabilities to sum to 1.
#[track_caller]
pub fn hmm_params_normalize_start(params: &mut HMMParams) {
    let sum = params.trans[HMMTRANS::HMMTRANS_START_M as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_START_IS as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_START_IL as usize];
    params.trans[HMMTRANS::HMMTRANS_START_M as usize] /= sum;
    params.trans[HMMTRANS::HMMTRANS_START_IS as usize] /= sum;
    params.trans[HMMTRANS::HMMTRANS_START_IL as usize] /= sum;
}

/// Renormalizes M and IS transitions for the short-gap state.
#[track_caller]
pub fn hmm_params_normalize_short_gap(params: &mut HMMParams) {
    let sum_m = params.trans[HMMTRANS::HMMTRANS_M_M as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IS as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IL as usize];
    params.trans[HMMTRANS::HMMTRANS_M_M as usize] /= sum_m;
    params.trans[HMMTRANS::HMMTRANS_M_IS as usize] /= sum_m;
    params.trans[HMMTRANS::HMMTRANS_M_IL as usize] /= sum_m;

    let sum_is = params.trans[HMMTRANS::HMMTRANS_IS_IS as usize]
        + params.trans[HMMTRANS::HMMTRANS_IS_M as usize];
    params.trans[HMMTRANS::HMMTRANS_IS_IS as usize] /= sum_is;
    params.trans[HMMTRANS::HMMTRANS_IS_M as usize] /= sum_is;
}

/// Renormalizes M and IL transitions for the long-gap state.
#[track_caller]
pub fn hmm_params_normalize_long_gap(params: &mut HMMParams) {
    let sum_m = params.trans[HMMTRANS::HMMTRANS_M_M as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IS as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IL as usize];
    params.trans[HMMTRANS::HMMTRANS_M_M as usize] /= sum_m;
    params.trans[HMMTRANS::HMMTRANS_M_IS as usize] /= sum_m;
    params.trans[HMMTRANS::HMMTRANS_M_IL as usize] /= sum_m;

    let sum_il = params.trans[HMMTRANS::HMMTRANS_IL_IL as usize]
        + params.trans[HMMTRANS::HMMTRANS_IL_M as usize];
    params.trans[HMMTRANS::HMMTRANS_IL_IL as usize] /= sum_il;
    params.trans[HMMTRANS::HMMTRANS_IL_M as usize] /= sum_il;
}

/// Renormalizes the outgoing transitions from the match state.
#[track_caller]
pub fn hmm_params_normalize_match(params: &mut HMMParams) {
    let sum_m = params.trans[HMMTRANS::HMMTRANS_M_M as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IS as usize]
        + 2.0 * params.trans[HMMTRANS::HMMTRANS_M_IL as usize];
    params.trans[HMMTRANS::HMMTRANS_M_M as usize] /= sum_m;
    params.trans[HMMTRANS::HMMTRANS_M_IS as usize] /= sum_m;
    params.trans[HMMTRANS::HMMTRANS_M_IL as usize] /= sum_m;
}

/// Renormalizes all transition and emission distributions and re-validates.
#[track_caller]
pub fn hmm_params_normalize(params: &mut HMMParams) {
    hmm_params_normalize_start(params);
    hmm_params_normalize_short_gap(params);
    hmm_params_normalize_long_gap(params);
    hmm_params_normalize_emit(params);
    hmm_params_assert_probs_valid(params);
}
