// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: build a Profile3 from an MSA and report its self-score.
#[track_caller]
pub fn cmd_msaselfscore3(
    input_file_name: &str,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
) -> (Profile3, f32, String) {
    let mut msa = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa, input_file_name, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&msa);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let seq_count = msa.seqs.len() as uint;
    let w = 1.0_f32 / seq_count as f32;
    let seq_weights = vec![w; seq_count as usize];

    let mut prof = Profile3::default();
    profile3_from_msa(&mut prof, &msa, subst_mx_letter, gap_open, &seq_weights);
    profile3_validate(&prof);
    let self_score = profile3_get_self_score(&prof);
    let score_s = if self_score == 0.0 {
        "0".to_string()
    } else if !self_score.is_finite() {
        self_score.to_string()
    } else {
        let score64 = f64::from(self_score);
        let exp = score64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 5 {
            let raw = format!("{score64:.4e}");
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
            format!("{score64:.decimals$}")
        };
        if !s.contains('e') && !s.contains('E') {
            s = s.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        if s == "-0" {
            s = "0".to_string();
        }
        s
    };
    let log = format!(
        "MSASelfScore3={}, MSA={}\n",
        score_s,
        base_name(input_file_name)
    );
    (prof, self_score, log)
}
