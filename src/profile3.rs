// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Profile3 {
    pub pps: Vec<ProfPos3>,
} // original: Profile3 (muscle/src/profile3.h)

/// Drop all per-column entries from the profile.
#[track_caller]
pub fn profile3_clear(prof: &mut Profile3) {
    if prof.pps.is_empty() {
        return;
    }
    prof.pps.clear();
}

/// Return the profile position at the given column.
#[track_caller]
pub fn profile3_get_pp(prof: &Profile3, col_index: uint) -> &ProfPos3 {
    assert!((col_index as usize) < prof.pps.len());
    &prof.pps[col_index as usize]
}

/// Set the gap-open score on the given column from the base gap-open penalty.
#[track_caller]
pub fn profile3_set_gap_open_score(prof: &mut Profile3, gap_open: f32, col_index: uint) {
    assert!((col_index as usize) < prof.pps.len());
    let pp = &mut prof.pps[col_index as usize];
    if col_index == 0 {
        pp.gap_open_score = pp.f_occ * gap_open / 2.0;
    } else {
        let gap_open_freq = pp.lg;
        pp.gap_open_score = gap_open * (1.0 - gap_open_freq) / 2.0;
    }
}

/// Set the gap-close score on the given column from the base gap-open penalty.
#[track_caller]
pub fn profile3_set_gap_close_score(prof: &mut Profile3, gap_open: f32, col_index: uint) {
    let col_count = prof.pps.len() as uint;
    assert!(col_index < col_count);
    if col_index + 1 == col_count {
        let pp = &mut prof.pps[col_index as usize];
        pp.gap_close_score = gap_open * pp.f_occ / 2.0;
    } else {
        let gap_close_freq = prof.pps[col_index as usize + 1].gl;
        let pp = &mut prof.pps[col_index as usize];
        pp.gap_close_score = gap_open * (1.0 - gap_close_freq) / 2.0;
    }
}

/// Build a profile from a single sequence by wrapping it as a singleton MSA.
#[track_caller]
pub fn profile3_from_seq(
    prof: &mut Profile3,
    seq: &Sequence,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
) {
    let mut msa = MultiSequence::default();
    msa.seqs.push(seq.clone());
    msa.owners.push(false);
    let seq_weights = vec![1.0_f32];
    profile3_from_msa(prof, &msa, subst_mx_letter, gap_open, &seq_weights);
}

/// Build the profile from an MSA. `seq_weights` must sum to 1 (used for the
/// per-column ProfPos3 gap open/close probabilities).
#[track_caller]
pub fn profile3_from_msa(
    prof: &mut Profile3,
    msa: &MultiSequence,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
    seq_weights: &[f32],
) {
    let col_count = multi_sequence_get_col_count(msa);
    let seq_count = msa.seqs.len();
    assert_eq!(seq_weights.len(), seq_count);
    let mut sum_weights = 0.0_f32;
    for w in seq_weights.iter().take(seq_count) {
        sum_weights += *w;
    }
    assert!(sum_weights > 0.9 && sum_weights < 1.1);

    prof.pps.clear();
    prof.pps.reserve(col_count as usize);
    for col_index in 0..col_count {
        let mut pp = ProfPos3::default();
        prof_pos3_set_freqs(&mut pp, msa, col_index, seq_weights);
        prof_pos3_set_aa_scores(&mut pp, subst_mx_letter);
        prof.pps.push(pp);
    }
    for col_index in 0..col_count {
        profile3_set_gap_open_score(prof, gap_open, col_index);
        profile3_set_gap_close_score(prof, gap_open, col_index);
    }
}

/// Format a small float for the LogMe table, blanking near-zero values.
#[track_caller]
pub fn log_f(f: f32) -> String {
    if f > -0.00001 && f < 0.00001 {
        "       ".to_string()
    } else {
        format!("  {f:5.3}")
    }
}

/// Render a human-readable log of the profile (gap stats, frequencies, scores).
#[track_caller]
pub fn profile3_log_me(prof: &Profile3, msa: Option<&MultiSequence>) -> String {
    let state = ALPHA_STATE.lock().unwrap().clone();
    let mut out = String::new();
    out.push_str("  Pos  Occ     LL     LG     GL     GG       Open    Close\n");
    out.push_str("  ---  ---     --     --     --     --     ------  -------\n");
    let col_count = prof.pps.len();
    for col_index in 0..col_count {
        let pp = &prof.pps[col_index];
        out.push_str(&format!("{col_index:5}"));
        out.push_str(&log_f(pp.f_occ));
        out.push_str(&log_f(pp.ll));
        out.push_str(&log_f(pp.lg));
        out.push_str(&log_f(pp.gl));
        out.push_str(&log_f(pp.gg));
        out.push_str(&format!("  {:7.3}", -pp.gap_open_score));
        out.push_str(&format!("  {:7.3}", -pp.gap_close_score));
        if let Some(msa) = msa {
            out.push_str("  ");
            for seq in &msa.seqs {
                out.push(seq.char_vec[col_index]);
            }
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str("  Pos");
    for n in 0..state.alpha_size as usize {
        out.push_str(&format!("     {}", state.letter_to_char[n] as char));
    }
    out.push('\n');
    out.push_str("  ---");
    for _ in 0..state.alpha_size as usize {
        out.push_str(" -----");
    }
    out.push('\n');

    for col_index in 0..col_count {
        let pp = &prof.pps[col_index];
        out.push_str(&format!("{col_index:5}"));
        let mut sum_freqs = 0.0_f32;
        for letter in 0..state.alpha_size as usize {
            let f = pp.freqs[letter];
            sum_freqs += f;
            if f == 0.0 {
                out.push_str("      ");
            } else {
                out.push_str(&format!(" {f:5.3}"));
            }
        }
        out.push_str(&format!("  (Sum={sum_freqs:4.2})"));
        if let Some(msa) = msa {
            out.push_str("  ");
            for seq in &msa.seqs {
                out.push(seq.char_vec[col_index]);
            }
        }
        for k in 0..state.alpha_size as usize {
            let i = pp.sort_order[k] as usize;
            let freq = pp.freqs[i];
            if freq > 0.0 {
                out.push_str(&format!(" {}({freq:.3})", state.letter_to_char[i] as char));
            }
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str("Scores\n");
    out.push_str("  Col");
    for n in 0..state.alpha_size as usize {
        out.push_str(&format!("     {}", state.letter_to_char[n] as char));
    }
    out.push('\n');
    for col_index in 0..col_count {
        let pp = &prof.pps[col_index];
        out.push_str(&format!("{col_index:5}"));
        for letter in 0..state.alpha_size as usize {
            let f = pp.aa_scores[letter];
            out.push_str(&format!(" {f:5.2}"));
        }
        out.push('\n');
    }
    out
}

/// Recompute amino-acid match scores for every column.
#[track_caller]
pub fn profile3_set_aa_scores(prof: &mut Profile3, subst_mx_letter: &[[f32; 20]; 20]) {
    let col_count = prof.pps.len();
    for col in 0..col_count {
        prof_pos3_set_aa_scores(&mut prof.pps[col], subst_mx_letter);
    }
}

/// Recompute both AA and gap scores for every column.
#[track_caller]
pub fn profile3_set_scores(prof: &mut Profile3, subst_mx_letter: &[[f32; 20]; 20], gap_open: f32) {
    profile3_set_aa_scores(prof, subst_mx_letter);
    profile3_set_gap_scores(prof, gap_open);
}

/// Recompute gap open and close scores for every column.
#[track_caller]
pub fn profile3_set_gap_scores(prof: &mut Profile3, gap_open: f32) {
    let col_count = prof.pps.len() as uint;
    for col_index in 0..col_count {
        profile3_set_gap_open_score(prof, gap_open, col_index);
        profile3_set_gap_close_score(prof, gap_open, col_index);
    }
}

/// Assert internal consistency of the per-column LL/LG/GL/GG and fOcc values.
#[track_caller]
pub fn profile3_validate(prof: &Profile3) {
    let col_count = prof.pps.len();
    for col_index in 0..col_count {
        let pp = &prof.pps[col_index];
        if !myfeq(pp.f_occ as f64, (pp.ll + pp.gl) as f64) {
            panic!("Col {col_index}, fOcc != LL + GL");
        }

        let s1 = pp.ll + pp.lg + pp.gl + pp.gg;
        assert!(myfeq(s1 as f64, 1.0));

        if col_index > 0 {
            let pp_prev = &prof.pps[col_index - 1];
            let s2 = pp_prev.ll + pp_prev.gl;
            let s3 = pp.ll + pp.lg;
            if !myfeq(s2 as f64, s3 as f64) {
                panic!("Col {col_index}, LL + LG != Prev.LL + Prev.GL");
            }
        }
        if col_index + 1 < col_count {
            let pp_next = &prof.pps[col_index + 1];
            let s4 = pp.ll + pp.gl;
            let s5 = pp_next.ll + pp_next.lg;
            if !myfeq(s4 as f64, s5 as f64) {
                panic!("Col {col_index}, LL + GL != Next.LL + Next.LG");
            }
        }
    }
}

/// Write the profile TSV dump to `file_name` (no-op when empty).
#[track_caller]
pub fn profile3_to_tsv_l249(prof: &Profile3, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    std::fs::write(file_name, profile3_to_tsv_l256(prof)).unwrap();
}

/// Render the profile as TSV text (column count followed by per-column rows).
#[track_caller]
pub fn profile3_to_tsv_l256(prof: &Profile3) -> String {
    let mut out = String::new();
    let col_count = prof.pps.len();
    out.push_str(&format!("{col_count}\n"));
    for i in 0..col_count {
        out.push_str(&i.to_string());
        profile3_write_pp_tsv(&mut out, &prof.pps[i]);
    }
    out
}

fn profile3_format_g5(value: f32) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if !value.is_finite() {
        return value.to_string();
    }

    const PRECISION: i32 = 5;
    let exp = value.abs().log10().floor() as i32;
    let mut s = if exp < -4 || exp >= PRECISION {
        let raw = format!("{:.*e}", (PRECISION - 1) as usize, value);
        let (mantissa, exponent) = raw.split_once('e').unwrap();
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        let exp_value = exponent.parse::<i32>().unwrap();
        let sign = if exp_value >= 0 { '+' } else { '-' };
        format!("{mantissa}e{sign}{:02}", exp_value.abs())
    } else {
        let decimals = (PRECISION - exp - 1).max(0) as usize;
        format!("{value:.decimals$}")
    };

    if !s.contains('e') && !s.contains('E') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

fn profile3_write_pp_tsv(out: &mut String, pp: &ProfPos3) {
    for value in [
        pp.ll,
        pp.lg,
        pp.gl,
        pp.gg,
        pp.f_occ,
        pp.gap_open_score,
        pp.gap_close_score,
    ] {
        out.push('\t');
        out.push_str(&profile3_format_g5(value));
    }
    for i in 0..20 {
        for value in [pp.freqs[i], pp.aa_scores[i]] {
            out.push('\t');
            out.push_str(&profile3_format_g5(value));
        }
    }
    out.push('\n');
}

/// Sum of self-similarity scores (profile against itself) over all columns.
#[track_caller]
pub fn profile3_get_self_score(prof: &Profile3) -> f32 {
    let mut sum = 0.0_f32;
    let col_count = prof.pps.len();
    for col in 0..col_count {
        let pp = profile3_get_pp(prof, col as uint);
        let score = score_prof_pos2(pp, pp);
        sum += score;
    }
    sum
}

/// Compare two profiles cell-by-cell; returns the number of differing entries
/// and a log of each difference.
#[track_caller]
pub fn profile3_log_diffs(prof: &Profile3, prof2: &Profile3) -> (uint, String) {
    let col_count = prof.pps.len();
    let col_count2 = prof2.pps.len();
    let mut out = String::new();
    if col_count2 != col_count {
        out.push_str(&format!("Lengths differ {col_count}, {col_count2}\n"));
        return (1, out);
    }

    let state = ALPHA_STATE.lock().unwrap().clone();
    let format_g4 = |d: f32| -> String {
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
    let mut diff_count = 0_u32;
    for col_index in 0..col_count {
        let pp = profile3_get_pp(prof, col_index as uint);
        let pp2 = profile3_get_pp(prof2, col_index as uint);
        for (name, f, f2) in [
            ("LL", pp.ll, pp2.ll),
            ("LG", pp.lg, pp2.lg),
            ("GL", pp.gl, pp2.gl),
            ("GG", pp.gg, pp2.gg),
            ("fOcc", pp.f_occ, pp2.f_occ),
            ("GapOpenScore", pp.gap_open_score, pp2.gap_open_score),
            ("GapCloseScore", pp.gap_close_score, pp2.gap_close_score),
        ] {
            if !myfeq(f as f64, f2 as f64) {
                diff_count += 1;
                out.push_str(&format!(
                    "Col {col_index} {name}: {} {}\n",
                    format_g4(f),
                    format_g4(f2)
                ));
            }
        }

        for i in 0..20 {
            let f = pp.freqs[i];
            let f2 = pp2.freqs[i];
            if !myfeq(f as f64, f2 as f64) {
                diff_count += 1;
                out.push_str(&format!(
                    "Col {col_index} Freqs[{}={}] {} {}\n",
                    i,
                    state.letter_to_char[i] as char,
                    format_g4(f),
                    format_g4(f2)
                ));
            }

            let f = pp.aa_scores[i];
            let f2 = pp2.aa_scores[i];
            if !myfeq(f as f64, f2 as f64) {
                diff_count += 1;
                out.push_str(&format!(
                    "Col {col_index} AAScores[{}={}] {} {}\n",
                    i,
                    state.letter_to_char[i] as char,
                    format_g4(f),
                    format_g4(f2)
                ));
            }
        }
    }
    (diff_count, out)
}

/// `build_prof3` subcommand: build a Profile3 from an input MFA and dump the
/// profile log and TSV. Returns (profile, log, tsv).
#[track_caller]
pub fn cmd_build_prof3(
    input_file_name: &str,
    output_file_name: &str,
    subst_mx_letter: &[[f32; 20]; 20],
    gap_open: f32,
) -> (Profile3, String, String) {
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
    let log = profile3_log_me(&prof, Some(&msa));
    profile3_validate(&prof);
    let tsv = profile3_to_tsv_l256(&prof);
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &tsv).expect("failed to write Profile3 TSV");
    }
    (prof, log, tsv)
}
