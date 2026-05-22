// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Writes the pair-HMM parameter report to the named file (no-op if empty).
#[track_caller]
pub fn pair_hmm_write_params_report_l4(file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    std::fs::write(file_name, pair_hmm_write_params_report_l13()).unwrap();
}

/// Renders the current pair-HMM probabilities and scores as a C-style report string.
#[track_caller]
pub fn pair_hmm_write_params_report_l13() -> String {
    let fmt5 = |x: f32| -> String {
        if x == 0.0 {
            return "0".to_string();
        }
        if !x.is_finite() {
            return x.to_string();
        }
        let x64 = f64::from(x);
        let exp = x64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 5 {
            let raw = format!("{x64:.4e}");
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
            format!("{x64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let start_score = *PAIR_HMM_START_SCORE.lock().unwrap();
    let trans_score = *PAIR_HMM_TRANS_SCORE.lock().unwrap();
    let ins_score = *PAIR_HMM_INS_SCORE.lock().unwrap();
    let match_score = PAIR_HMM_MATCH_SCORE.read().unwrap();

    let mut init_probs = Vec::new();
    let mut init_scores = Vec::new();
    for score in start_score.iter().take(HMMSTATE_COUNT as usize) {
        init_probs.push(score.exp());
        init_scores.push(*score);
    }

    let init_prob_m = init_probs[HMMSTATE_M as usize];
    let init_prob_ix = init_probs[HMMSTATE_IX as usize];
    let init_prob_iy = init_probs[HMMSTATE_IY as usize];
    let init_prob_jx = init_probs[HMMSTATE_JX as usize];
    let init_prob_jy = init_probs[HMMSTATE_JY as usize];
    let init_sum = init_prob_m + init_prob_ix + init_prob_iy + init_prob_jx + init_prob_jy;
    assert!(myfeq(init_prob_ix as f64, init_prob_iy as f64));
    assert!(myfeq(init_prob_jx as f64, init_prob_jy as f64));
    assert!(myfeq(init_sum as f64, 1.0));

    let mut out = String::new();
    out.push('\n');
    out.push_str("// Probs\n");
    out.push_str(&format!(
        "const float InitProb_IM = {};\n",
        fmt5(init_prob_m)
    ));
    out.push_str(&format!(
        "const float InitProb_IS = {};\n",
        fmt5(init_prob_ix)
    ));
    out.push_str(&format!(
        "const float InitProb_IL = {};\n",
        fmt5(init_prob_jx)
    ));

    let mut trans_probs = vec![vec![0.0_f32; HMMSTATE_COUNT as usize]; HMMSTATE_COUNT as usize];
    let mut trans_scores = vec![vec![0.0_f32; HMMSTATE_COUNT as usize]; HMMSTATE_COUNT as usize];
    for i in 0..HMMSTATE_COUNT as usize {
        let mut sum = 0.0_f32;
        for j in 0..HMMSTATE_COUNT as usize {
            let score = trans_score[i][j];
            let prob = score.exp();
            sum += prob;
            trans_probs[i][j] = prob;
            trans_scores[i][j] = score;
        }
        assert!(myfeq(sum as f64, 1.0));
    }

    assert_eq!(trans_probs[HMMSTATE_IX as usize][HMMSTATE_IY as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_IX as usize][HMMSTATE_JY as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_JX as usize][HMMSTATE_IY as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_JX as usize][HMMSTATE_JY as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_IY as usize][HMMSTATE_IX as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_IY as usize][HMMSTATE_JX as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_JY as usize][HMMSTATE_IX as usize], 0.0);
    assert_eq!(trans_probs[HMMSTATE_JY as usize][HMMSTATE_JX as usize], 0.0);
    assert!(myfeq(
        trans_probs[HMMSTATE_M as usize][HMMSTATE_IX as usize] as f64,
        trans_probs[HMMSTATE_M as usize][HMMSTATE_IY as usize] as f64
    ));
    assert!(myfeq(
        trans_probs[HMMSTATE_M as usize][HMMSTATE_JX as usize] as f64,
        trans_probs[HMMSTATE_M as usize][HMMSTATE_JY as usize] as f64
    ));
    assert!(myfeq(
        trans_probs[HMMSTATE_IX as usize][HMMSTATE_M as usize] as f64,
        trans_probs[HMMSTATE_IY as usize][HMMSTATE_M as usize] as f64
    ));
    assert!(myfeq(
        trans_probs[HMMSTATE_JX as usize][HMMSTATE_M as usize] as f64,
        trans_probs[HMMSTATE_JY as usize][HMMSTATE_M as usize] as f64
    ));

    let trans_prob_m_m = trans_probs[HMMSTATE_M as usize][HMMSTATE_M as usize];
    let trans_score_m_m = trans_scores[HMMSTATE_M as usize][HMMSTATE_M as usize];
    let trans_prob_m_is = trans_probs[HMMSTATE_M as usize][HMMSTATE_IX as usize];
    let trans_score_m_is = trans_scores[HMMSTATE_M as usize][HMMSTATE_IX as usize];
    let trans_prob_m_il = trans_probs[HMMSTATE_M as usize][HMMSTATE_JX as usize];
    let trans_score_m_il = trans_scores[HMMSTATE_M as usize][HMMSTATE_JX as usize];
    let trans_prob_is_is = trans_probs[HMMSTATE_IX as usize][HMMSTATE_IX as usize];
    let trans_score_is_is = trans_scores[HMMSTATE_IX as usize][HMMSTATE_IX as usize];
    let trans_prob_il_il = trans_probs[HMMSTATE_JX as usize][HMMSTATE_JX as usize];
    let trans_score_il_il = trans_scores[HMMSTATE_JX as usize][HMMSTATE_JX as usize];
    let trans_prob_is_m = trans_probs[HMMSTATE_IX as usize][HMMSTATE_M as usize];
    let trans_score_is_m = trans_scores[HMMSTATE_IX as usize][HMMSTATE_M as usize];
    let trans_prob_il_m = trans_probs[HMMSTATE_JX as usize][HMMSTATE_M as usize];
    let trans_score_il_m = trans_scores[HMMSTATE_JX as usize][HMMSTATE_M as usize];

    assert!(myfeq(
        (init_prob_m + 2.0 * init_prob_ix + 2.0 * init_prob_jx) as f64,
        1.0
    ));
    assert!(myfeq((trans_prob_is_is + trans_prob_is_m) as f64, 1.0));
    assert!(myfeq((trans_prob_il_il + trans_prob_il_m) as f64, 1.0));
    assert!(myfeq(
        (trans_prob_m_m + 2.0 * trans_prob_m_is + 2.0 * trans_prob_m_il) as f64,
        1.0
    ));

    out.push('\n');
    out.push_str(&format!(
        "const float TransProb_M_M   = {};\n",
        fmt5(trans_prob_m_m)
    ));
    out.push_str(&format!(
        "const float TransProb_M_IS  = {};\n",
        fmt5(trans_prob_m_is)
    ));
    out.push_str(&format!(
        "const float TransProb_M_IL  = {};\n",
        fmt5(trans_prob_m_il)
    ));
    out.push_str(&format!(
        "const float TransProb_IS_IS = {};\n",
        fmt5(trans_prob_is_is)
    ));
    out.push_str(&format!(
        "const float TransProb_IL_IL = {};\n",
        fmt5(trans_prob_il_il)
    ));
    out.push_str(&format!(
        "const float TransProb_IS_M  = {};\n",
        fmt5(trans_prob_is_m)
    ));
    out.push_str(&format!(
        "const float TransProb_IL_M  = {};\n",
        fmt5(trans_prob_il_m)
    ));

    let a = b"ACDEFGHIKLMNPQRSTVWY";
    assert_eq!(a.len(), 20);
    let mut ins_probs = vec![0.0_f32; 20];
    let mut ins_scores = vec![0.0_f32; 20];
    let mut sum = 0.0_f32;
    for i in 0..20 {
        let letter = a[i] as usize;
        let score = ins_score[letter];
        let prob = score.exp();
        ins_probs[i] = prob;
        ins_scores[i] = score;
        sum += prob;
    }
    assert!(myfeq(sum as f64, 1.0));

    let mut emit_probs = vec![vec![0.0_f32; 20]; 20];
    let mut emit_scores = vec![vec![0.0_f32; 20]; 20];
    sum = 0.0;
    for i in 0..20 {
        let letter_a = a[i] as usize;
        for j in 0..20 {
            let letter_b = a[j] as usize;
            let score = match_score[letter_a][letter_b];
            let prob = score.exp();
            emit_probs[i][j] = prob;
            emit_scores[i][j] = score;
            sum += prob;
        }
    }
    assert!(myfeq(sum as f64, 1.0));

    out.push('\n');
    out.push_str("const float InsProbs[20] =\n");
    out.push_str("\t{\n");
    for i in 0..20 {
        out.push_str(&format!("\t{},\t// {}\n", fmt5(ins_probs[i]), a[i] as char));
    }
    out.push_str("\t};\n");

    out.push('\n');
    out.push_str("const float EmitProbs[20][20] =\n");
    out.push_str("\t{\n");
    out.push_str("//\t      ");
    for c in a {
        out.push_str(&format!("        {}", *c as char));
    }
    out.push('\n');
    for i in 0..20 {
        out.push_str(&format!("/* {} */ {{ ", a[i] as char));
        for j in 0..20 {
            out.push_str(&format!(" {}", fmt5(emit_probs[i][j])));
        }
        out.push_str(&format!(" }} // {}\n", a[i] as char));
    }
    out.push_str("\t};\n");

    out.push('\n');
    out.push_str("// Scores\n");
    out.push_str(&format!(
        "const float InitScore_IM = {};\n",
        fmt5(init_scores[HMMSTATE_M as usize])
    ));
    out.push_str(&format!(
        "const float InitScore_IS = {};\n",
        fmt5(init_scores[HMMSTATE_IX as usize])
    ));
    out.push_str(&format!(
        "const float InitScore_IL = {};\n",
        fmt5(init_scores[HMMSTATE_JX as usize])
    ));

    out.push('\n');
    out.push_str(&format!(
        "const float TransScore_M_M   = {};\n",
        fmt5(trans_score_m_m)
    ));
    out.push_str(&format!(
        "const float TransScore_M_IS  = {};\n",
        fmt5(trans_score_m_is)
    ));
    out.push_str(&format!(
        "const float TransScore_M_IL  = {};\n",
        fmt5(trans_score_m_il)
    ));
    out.push_str(&format!(
        "const float TransScore_IS_IS = {};\n",
        fmt5(trans_score_is_is)
    ));
    out.push_str(&format!(
        "const float TransScore_IL_IL = {};\n",
        fmt5(trans_score_il_il)
    ));
    out.push_str(&format!(
        "const float TransScore_IS_M  = {};\n",
        fmt5(trans_score_is_m)
    ));
    out.push_str(&format!(
        "const float TransScore_IL_M  = {};\n",
        fmt5(trans_score_il_m)
    ));

    out.push('\n');
    out.push_str("const float InsScores[20] =\n");
    out.push_str("\t{\n");
    for i in 0..20 {
        out.push_str(&format!(
            "\t{},\t// {}\n",
            fmt5(ins_scores[i]),
            a[i] as char
        ));
    }
    out.push_str("\t};\n");

    out.push('\n');
    out.push_str("const float EmitScores[20][20] =\n");
    out.push_str("\t{\n");
    out.push_str("//\t      ");
    for c in a {
        out.push_str(&format!("        {}", *c as char));
    }
    out.push('\n');
    for i in 0..20 {
        out.push_str(&format!("/* {} */ {{ ", a[i] as char));
        for j in 0..20 {
            out.push_str(&format!(" {}", fmt5(emit_scores[i][j])));
        }
        out.push_str(&format!(" }} // {}\n", a[i] as char));
    }
    out.push_str("\t};\n");
    out
}

/// Implements the `hmmdump` command: writes default HMM params, scores, and reports to `out_dir`.
#[track_caller]
pub fn cmd_hmmdump<FUpdate>(
    out_dir: &str,
    nucleo: bool,
    mut cmd_line_update: FUpdate,
) -> Vec<String>
where
    FUpdate: FnMut(&mut HMMParams),
{
    let mut out_dir = out_dir.to_string();
    dirize(&mut out_dir);

    set_alpha_l209(ALPHA::ALPHA_Amino);
    init_probcons();

    let params_report = format!("{out_dir}params_report.txt");
    pair_hmm_write_params_report_l4(&params_report);

    let mut hp = hmm_params_from_defaults(nucleo);
    let hmm = format!("{out_dir}hmm.tsv");
    hmm_params_to_file(&hp, &hmm);

    cmd_line_update(&mut hp);
    hmm_params_to_pair_hmm(&hp);
    let params_report2 = format!("{out_dir}params_report2.txt");
    pair_hmm_write_params_report_l4(&params_report2);

    let hmm2 = format!("{out_dir}hmm2.tsv");
    hmm_params_to_file(&hp, &hmm2);
    hp = hmm_params_from_file(&hmm2);
    let hmm3 = format!("{out_dir}hmm3.tsv");
    hmm_params_to_file(&hp, &hmm3);

    let sa = hmm_params_to_single_affine_probs(&hp);
    let sa_hmm = format!("{out_dir}sa.hmm");
    hmm_params_to_file(&sa, &sa_hmm);

    vec![params_report, hmm, params_report2, hmm2, hmm3, sa_hmm]
}
