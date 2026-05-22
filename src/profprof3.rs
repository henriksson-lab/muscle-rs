// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `profprof3` subcommand: build Profile3s from two MSAs, align them with
/// NWSmall3, then validate by also computing the joint profile both via the
/// merged MSA and the alignment path, and log the diffs.
#[track_caller]
pub fn cmd_profprof3<FNwSmall3, FAlignTwoProfsGivenPath>(
    input_file_name: &str,
    input_file_name2: &str,
    output_file_name: &str,
    output1_file_name: &str,
    output2_file_name: &str,
    output3_file_name: &str,
    output4_file_name: &str,
    ap: &M3AlnParams,
    mut nw_small3: FNwSmall3,
    mut align_two_profs_given_path: FAlignTwoProfsGivenPath,
) -> (
    MultiSequence,
    Profile3,
    Profile3,
    Profile3,
    Profile3,
    uint,
    String,
)
where
    FNwSmall3: FnMut(&mut CacheMem3, &Profile3, &Profile3) -> (f32, String),
    FAlignTwoProfsGivenPath:
        FnMut(&Profile3, f32, &Profile3, f32, &[[f32; 20]; 20], f32, &str) -> Profile3,
{
    let subst_mx_letter = ap.subst_mx_letter;
    let gap_open = ap.gap_open;

    let mut msa1 = MultiSequence::default();
    let mut msa2 = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut msa1, input_file_name, false);
    multi_sequence_load_mfa_l8(&mut msa2, input_file_name2, false);
    let is_nucleo = multi_sequence_guess_is_nucleo(&msa1);
    let is_nucleo2 = multi_sequence_guess_is_nucleo(&msa2);
    assert_eq!(is_nucleo, is_nucleo2);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let seq_count1 = msa1.seqs.len() as uint;
    let seq_count2 = msa2.seqs.len() as uint;
    let w1 = 1.0_f32 / seq_count1 as f32;
    let w2 = 1.0_f32 / seq_count2 as f32;
    let seq_weights1 = vec![w1; seq_count1 as usize];
    let seq_weights2 = vec![w2; seq_count2 as usize];

    let seq_count12 = seq_count1 + seq_count2;
    let w12 = 1.0_f32 / seq_count12 as f32;
    let seq_weights12 = vec![w12; seq_count12 as usize];

    let mut prof1 = Profile3::default();
    let mut prof2 = Profile3::default();
    profile3_from_msa(&mut prof1, &msa1, &subst_mx_letter, gap_open, &seq_weights1);
    profile3_from_msa(&mut prof2, &msa2, &subst_mx_letter, gap_open, &seq_weights2);
    profile3_validate(&prof1);
    profile3_validate(&prof2);
    profile3_to_tsv_l249(&prof1, output1_file_name);
    profile3_to_tsv_l249(&prof2, output2_file_name);

    let mut cm = CacheMem3::default();
    let (score, path) = nw_small3(&mut cm, &prof1, &prof2);
    let mut log = String::new();
    let score_s = if score == 0.0 {
        "0".to_string()
    } else {
        let score64 = f64::from(score);
        let exp = score64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{score64:.3e}");
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
    log.push_str(&format!("Score={score_s}\n"));
    log.push_str(&format!("Path={path}\n"));

    let mut msa12 = MultiSequence::default();
    align_two_ms_as_given_path(&msa1, &msa2, &path, &mut msa12);
    multi_sequence_write_mfa(&msa12, output_file_name);

    let mut prof12_msa = Profile3::default();
    profile3_from_msa(
        &mut prof12_msa,
        &msa12,
        &subst_mx_letter,
        gap_open,
        &seq_weights12,
    );
    profile3_to_tsv_l249(&prof12_msa, output4_file_name);
    profile3_validate(&prof12_msa);

    let prof12_path =
        align_two_profs_given_path(&prof1, 0.5, &prof2, 0.5, &subst_mx_letter, gap_open, &path);
    profile3_to_tsv_l249(&prof12_path, output3_file_name);
    profile3_validate(&prof12_path);

    let (diff_count, diff_log) = profile3_log_diffs(&prof12_msa, &prof12_path);
    log.push_str(&diff_log);
    log.push_str(&format!("{diff_count} diffs\n"));
    (
        msa12,
        prof1,
        prof2,
        prof12_msa,
        prof12_path,
        diff_count,
        log,
    )
}
