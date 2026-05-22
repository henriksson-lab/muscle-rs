// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct ProfPos3 {
    pub all_gaps: bool,
    pub sort_order: [byte; 21],
    pub freqs: [f32; 20],
    pub ll: f32,
    pub lg: f32,
    pub gl: f32,
    pub gg: f32,
    pub f_occ: f32,
    pub aa_scores: [f32; 20],
    pub gap_open_score: f32,
    pub gap_close_score: f32,
} // original: ProfPos3 (muscle/src/profpos3.h)

/// Return a sort order of letter indexes by descending count.
#[track_caller]
pub fn sort_counts(counts: &[f32]) -> [byte; 21] {
    let alpha_size = ALPHA_STATE.lock().unwrap().alpha_size as usize;
    assert!(counts.len() >= alpha_size);
    let mut order = [0_u8; 21];
    for (i, item) in order.iter_mut().enumerate().take(20) {
        *item = i as byte;
    }
    let mut any = true;
    while any {
        any = false;
        for n in 0..alpha_size.saturating_sub(1) {
            let i1 = order[n] as usize;
            let i2 = order[n + 1] as usize;
            if counts[i1] < counts[i2] {
                order[n + 1] = i1 as byte;
                order[n] = i2 as byte;
                any = true;
            }
        }
    }
    order
}

/// Initialise the pseudo-column dimer counts placed before the first column.
#[track_caller]
pub fn prof_pos3_set_start_dimers(pp: &mut ProfPos3) {
    pp.ll = 1.0;
    pp.lg = 0.0;
    pp.gl = 0.0;
    pp.gg = 0.0;
}

/// Compute the column occupancy `f_occ = LL + GL`.
#[track_caller]
pub fn prof_pos3_set_occ(pp: &mut ProfPos3) {
    pp.f_occ = pp.ll + pp.gl;
}

/// Set alignment scores from the column frequencies and a substitution matrix.
#[track_caller]
pub fn prof_pos3_set_aa_scores(pp: &mut ProfPos3, subst_mx_letter: &[[f32; 20]; 20]) {
    let alpha_size = ALPHA_STATE.lock().unwrap().alpha_size as usize;
    pp.sort_order = sort_counts(&pp.freqs);
    for (i, row) in subst_mx_letter.iter().enumerate().take(alpha_size) {
        let mut sum = 0.0_f32;
        for (j, score) in row.iter().enumerate().take(alpha_size) {
            let freq = pp.freqs[j];
            sum += freq * *score;
        }
        pp.aa_scores[i] = sum;
    }
}

/// Compute letter frequencies and LL/LG/GL/GG dimer counts for one MSA column.
/// `seq_weights` must sum to 1.
#[track_caller]
pub fn prof_pos3_set_freqs(
    pp: &mut ProfPos3,
    msa: &MultiSequence,
    col_index: uint,
    seq_weights: &[f32],
) {
    let seq_count = msa.seqs.len();
    assert_eq!(seq_weights.len(), seq_count);
    let col_count = multi_sequence_get_col_count(msa);
    assert!(col_index < col_count);
    let alpha_state = ALPHA_STATE.lock().unwrap().clone();

    pp.all_gaps = true;
    pp.freqs = [0.0; 20];
    pp.ll = 0.0;
    pp.lg = 0.0;
    pp.gl = 0.0;
    pp.gg = 0.0;
    pp.f_occ = 0.0;

    for (seq_index, seq) in msa.seqs.iter().enumerate() {
        let w = seq_weights[seq_index];
        let c = seq.char_vec[col_index as usize];
        let letter_here = !matches!(c, '-' | '.');
        let letter_prev =
            col_index == 0 || !matches!(seq.char_vec[col_index as usize - 1], '-' | '.');
        let _letter_next = col_index == col_count - 1
            || !matches!(seq.char_vec[col_index as usize + 1], '-' | '.');
        if letter_here {
            pp.all_gaps = false;
            pp.f_occ += w;
            let letter = alpha_state.char_to_letter[c as usize] as usize;
            if letter < alpha_state.alpha_size as usize {
                pp.freqs[letter] += w;
            }
            if letter_prev {
                pp.ll += w;
            } else {
                pp.gl += w;
            }
        } else if letter_prev {
            pp.lg += w;
        } else {
            pp.gg += w;
        }
    }
}

/// Variant of `prof_pos3_set_freqs` that takes pre-computed letter and dimer
/// counts (used when the column statistics were collected elsewhere).
#[track_caller]
pub fn prof_pos3_set_freqs2(
    pp: &mut ProfPos3,
    seq_count: uint,
    ll_count: uint,
    lg_count: uint,
    gl_count: uint,
    gg_count: uint,
    letter_counts: &[uint],
) {
    let alpha_size = ALPHA_STATE.lock().unwrap().alpha_size as usize;
    assert_eq!(letter_counts.len(), alpha_size);
    assert_eq!(ll_count + lg_count + gl_count + gg_count, seq_count);
    let letter_count = ll_count + gl_count;
    let mut letter_sum = 0;
    for count in letter_counts.iter().take(alpha_size) {
        letter_sum += *count;
    }
    assert!(letter_sum <= seq_count);
    assert_eq!(letter_sum, letter_count);

    let n = seq_count as f32;
    pp.ll = ll_count as f32 / n;
    pp.lg = lg_count as f32 / n;
    pp.gl = gl_count as f32 / n;
    pp.gg = gg_count as f32 / n;
    pp.f_occ = pp.ll + pp.gl;
    pp.freqs = [0.0; 20];

    let mut sum_freq = 0.0_f32;
    for (letter, count) in letter_counts.iter().enumerate().take(alpha_size) {
        let freq = *count as f32 / n;
        pp.freqs[letter] = freq;
        sum_freq += freq;
    }
    assert!((sum_freq - pp.f_occ).abs() <= 1e-6);
}

/// Human-readable log dump of a profile position (dimers, freqs, AA scores).
#[track_caller]
pub fn prof_pos3_log_me(pp: &ProfPos3) -> String {
    let state = ALPHA_STATE.lock().unwrap();
    let mut s = String::new();
    for (name, value) in [
        ("LL", pp.ll),
        ("LG", pp.lg),
        ("GL", pp.gl),
        ("GG", pp.gg),
        ("fOcc", pp.f_occ),
    ] {
        s.push(' ');
        s.push_str(name);
        s.push('=');
        s.push_str(&format!("{value:.3}"));
    }
    s.push('\n');

    s.push_str(" Freqs: ");
    let mut zero_found = false;
    for i in 0..20 {
        let letter = pp.sort_order[i] as usize;
        let freq = pp.freqs[letter];
        let c = state.letter_to_char[letter];
        if freq == 0.0 {
            zero_found = true;
        }
        if zero_found {
            assert_eq!(freq, 0.0);
            continue;
        }
        s.push(' ');
        s.push(c as char);
        s.push('=');
        let mut formatted = format!("{freq:.3}");
        if formatted.contains('.') {
            while formatted.ends_with('0') {
                formatted.pop();
            }
            if formatted.ends_with('.') {
                formatted.pop();
            }
        }
        s.push_str(&formatted);
    }
    s.push('\n');

    s.push_str("Scores: ");
    for i in 0..20 {
        if i == 9 {
            s.push_str("\n   ");
        }
        let letter = pp.sort_order[i] as usize;
        let score = pp.aa_scores[letter];
        let c = state.letter_to_char[letter];
        s.push(' ');
        s.push(c as char);
        s.push('=');
        let mut formatted = format!("{score:.3}");
        if formatted.contains('.') {
            while formatted.ends_with('0') {
                formatted.pop();
            }
            if formatted.ends_with('.') {
                formatted.pop();
            }
        }
        s.push_str(&formatted);
    }
    s.push('\n');
    s
}

/// Render a single profile position as a tab-separated row.
#[track_caller]
pub fn prof_pos3_to_tsv(pp: &ProfPos3) -> String {
    let mut s = String::new();
    for value in [
        pp.ll,
        pp.lg,
        pp.gl,
        pp.gg,
        pp.f_occ,
        pp.gap_open_score,
        pp.gap_close_score,
    ] {
        s.push('\t');
        let mut formatted = format!("{value:.5}");
        if formatted.contains('.') {
            while formatted.ends_with('0') {
                formatted.pop();
            }
            if formatted.ends_with('.') {
                formatted.pop();
            }
        }
        s.push_str(&formatted);
    }
    for i in 0..20 {
        for value in [pp.freqs[i], pp.aa_scores[i]] {
            s.push('\t');
            let mut formatted = format!("{value:.5}");
            if formatted.contains('.') {
                while formatted.ends_with('0') {
                    formatted.pop();
                }
                if formatted.ends_with('.') {
                    formatted.pop();
                }
            }
            s.push_str(&formatted);
        }
    }
    s.push('\n');
    s
}
