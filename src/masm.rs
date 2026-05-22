// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sequence {
    pub label: String,
    pub char_vec: Vec<char>,
} // original: Sequence (muscle/src/masm.h)

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MASM {
    pub aln: Option<Box<MultiSequence>>,
    pub label: String,
    pub col_count: uint,
    pub seq_count: uint,
    pub feature_count: uint,
    pub feature_names: Vec<String>,
    pub alpha_sizes: Vec<uint>,
    pub cols: Vec<MASMCol>,
    pub ungapped_seqs: Vec<String>,
    pub feature_aln_vec: Vec<Vec<Vec<byte>>>,
    pub gap_open: f32,
    pub gap_ext: f32,
    pub aa_feature_idx: uint,
} // original: MASM (muscle/src/masm.h)

/// Score a MASM column against a profile column by summing per-feature scores.
#[track_caller]
pub fn score_pp(ppa: &MASMCol, prof_col_b: &[byte]) -> f32 {
    let feature_count = MEGA_STATE.lock().unwrap().feature_count as usize;
    let mut total_score = 0.0_f32;
    for feature_idx in 0..feature_count {
        let scores_a = &ppa.scores_vec[feature_idx];
        let letter_b = prof_col_b[feature_idx];
        if letter_b != byte::MAX {
            total_score += scores_a[letter_b as usize];
        }
    }
    total_score
}

/// Build the substitution score matrix between a MASM and a per-position feature profile.
#[track_caller]
pub fn masm_make_s_mx(masm: &MASM, prof_b: &[Vec<byte>]) -> Mx {
    let la = masm.col_count as usize;
    let lb = prof_b.len();
    let mut smx = Mx {
        row_count: masm.col_count,
        col_count: lb as uint,
        data: vec![vec![0.0; lb]; la],
        ..Mx::default()
    };
    for pos_a in 0..la {
        let col_a = &masm.cols[pos_a];
        for (pos_b, col_b) in prof_b.iter().enumerate() {
            smx.data[pos_a][pos_b] = score_pp(col_a, col_b);
        }
    }
    smx
}

/// Count letters and the three kinds of gaps (open/ext/close) at a MASM column.
#[track_caller]
pub fn masm_get_counts(masm: &MASM, col_index: uint) -> (uint, uint, uint, uint) {
    let mut letter_count = 0;
    let mut gap_open_count = 0;
    let mut gap_ext_count = 0;
    let mut gap_close_count = 0;
    let aln = masm.aln.as_ref().expect("MASM::GetCounts, m_Aln is null");
    let col = col_index as usize;
    assert!(col_index < masm.col_count);
    for seq_index in 0..masm.seq_count as usize {
        let seq = &aln.seqs[seq_index].char_vec;
        assert_eq!(seq.len(), masm.col_count as usize);
        let gap_prev = if col_index == 0 {
            false
        } else {
            seq[col - 1] == '-' || seq[col - 1] == '.'
        };
        let gap = seq[col] == '-' || seq[col] == '.';
        let gap_next = if col_index + 1 == masm.col_count {
            false
        } else {
            seq[col + 1] == '-' || seq[col + 1] == '.'
        };
        if gap {
            if gap_prev {
                gap_ext_count += 1;
            } else if gap_next {
                gap_open_count += 1;
            } else {
                gap_close_count += 1;
            }
        } else {
            letter_count += 1;
        }
    }
    (letter_count, gap_open_count, gap_ext_count, gap_close_count)
}

/// Return per-letter frequencies at a MASM column for the given feature.
#[track_caller]
pub fn masm_get_freqs(masm: &MASM, col_index: uint, feature_idx: uint) -> Vec<f32> {
    let alpha_size = mega_get_alpha_size(feature_idx) as usize;
    let mut counts = vec![0_u32; alpha_size];
    let feature_idx_usize = feature_idx as usize;
    let col = col_index as usize;
    assert!(feature_idx_usize < masm.feature_aln_vec.len());
    for seq_idx in 0..masm.seq_count as usize {
        let letter = masm.feature_aln_vec[feature_idx_usize][seq_idx][col];
        if letter == byte::MAX {
            continue;
        }
        assert!((letter as usize) < alpha_size);
        counts[letter as usize] += 1;
    }
    let mut freqs = Vec::new();
    for n in counts {
        freqs.push(n as f32 / masm.seq_count as f32);
    }
    freqs
}

/// Return per-feature letter frequencies at a MASM column.
#[track_caller]
pub fn masm_get_freqs_vec(masm: &MASM, col_index: uint) -> Vec<Vec<f32>> {
    let feature_count = masm.feature_count;
    let mut freqs_vec = Vec::new();
    for feature_idx in 0..feature_count {
        freqs_vec.push(masm_get_freqs(masm, col_index, feature_idx));
    }
    freqs_vec
}

/// Build a MASM (multi-aligned scoring model) from an MSA and gap parameters.
#[track_caller]
pub fn masm_from_msa(
    masm: &mut MASM,
    aln: &MultiSequence,
    label: &str,
    gap_open: f32,
    gap_ext: f32,
) {
    assert!(gap_open >= 0.0);
    assert!(gap_ext >= 0.0);

    *masm = MASM::default();
    masm.label = label.to_string();
    masm.gap_open = gap_open;
    masm.gap_ext = gap_ext;
    masm.aln = Some(Box::new(aln.clone()));
    masm.col_count = multi_sequence_get_col_count(aln);
    masm.seq_count = aln.seqs.len() as uint;
    {
        let mega = MEGA_STATE.lock().unwrap();
        masm.feature_count = mega.feature_count;
        masm.feature_names = mega.feature_names.clone();
        masm.alpha_sizes = mega.alpha_sizes.clone();
    }
    masm.aa_feature_idx = mega_get_aa_feature_idx();

    masm_set_ungapped_seqs(masm);
    masm_set_feature_aln_vec(masm);
    for col_index in 0..masm.col_count {
        let (letter_count, gap_open_count, gap_ext_count, gap_close_count) =
            masm_get_counts(masm, col_index);
        assert_eq!(
            letter_count + gap_open_count + gap_ext_count + gap_close_count,
            masm.seq_count
        );

        let mut col = MASMCol {
            masm: Some(Box::new(MASM {
                feature_count: masm.feature_count,
                feature_names: masm.feature_names.clone(),
                alpha_sizes: masm.alpha_sizes.clone(),
                aa_feature_idx: masm.aa_feature_idx,
                ..MASM::default()
            })),
            col_index,
            letter_freq: letter_count as f32 / masm.seq_count as f32,
            gap_open_freq: gap_open_count as f32 / masm.seq_count as f32,
            gap_ext_freq: gap_ext_count as f32 / masm.seq_count as f32,
            gap_close_freq: gap_close_count as f32 / masm.seq_count as f32,
            ..MASMCol::default()
        };
        let sum = col.gap_open_freq + col.gap_ext_freq + col.gap_close_freq + col.letter_freq;
        if sum != 1.0 {
            let max = sum.abs().max(1.0);
            let diff = (sum.abs() - 1.0_f32).abs();
            assert!(diff < max * 0.01);
        }

        col.gap_open = (1.0 - col.gap_open_freq) * gap_open / 2.0;
        col.gap_close = (1.0 - col.gap_close_freq) * gap_open / 2.0;
        col.gap_ext = (1.0 - col.gap_ext_freq) * gap_ext;
        col.freqs_vec = masm_get_freqs_vec(masm, col_index);
        masm_col_set_score_vec(&mut col);
        masm.cols.push(col);
    }
}

/// Serialize a MASM to the given file path (no-op if empty).
#[track_caller]
pub fn masm_to_file_l150(masm: &MASM, file_name: &str) {
    if file_name.is_empty() {
        return;
    }
    let mut file = TextFile::default();
    masm_to_file_l159(masm, &mut file);
    std::fs::write(file_name, &file.data)
        .unwrap_or_else(|e| panic!("CreateStdioFile({file_name}): {e}"));
}

/// Write a MASM header and all columns to an open text file.
#[track_caller]
pub fn masm_to_file_l159(masm: &MASM, file: &mut TextFile) {
    let format_g4 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let d64 = f64::from(d);
        let exp = d64.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d64:.3e}");
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
    text_file_put_format(
        file,
        &format!(
            "MASM\t{}\t{}\t{}\t{}\t{}\t{}\n",
            masm.seq_count,
            masm.col_count,
            masm.feature_count,
            format_g4(masm.gap_open),
            format_g4(masm.gap_ext),
            masm.label
        ),
    );
    for i in 0..masm.feature_count as usize {
        text_file_put_format(
            file,
            &format!(
                "feature\t{}\t{}\t{}\n",
                i, masm.feature_names[i], masm.alpha_sizes[i]
            ),
        );
    }
    for i in 0..masm.col_count as usize {
        masm_col_to_file(&masm.cols[i], file);
    }
}

/// Return a human-readable dump of a MASM for logging.
#[track_caller]
pub fn masm_log_me(masm: &MASM) -> String {
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
    let mut out = format!(
        "MASM {} seqs, {} cols, {} features, open {}, ext {}, label {}\n",
        masm.seq_count,
        masm.col_count,
        masm.feature_count,
        format_g3(masm.gap_open),
        format_g3(masm.gap_ext),
        masm.label
    );
    for i in 0..masm.feature_count as usize {
        out.push_str(&format!(
            "Feature {}  AS {}  {}\n",
            i, masm.alpha_sizes[i], masm.feature_names[i]
        ));
    }
    for i in 0..masm.col_count as usize {
        out.push_str(&masm_col_log_me(&masm.cols[i]));
    }
    out
}

/// Build per-feature aligned letter vectors for every sequence.
#[track_caller]
pub fn masm_set_feature_aln_vec(masm: &mut MASM) {
    masm.feature_aln_vec.clear();
    masm.feature_aln_vec
        .resize(masm.feature_count as usize, Vec::new());
    for feature_idx in 0..masm.feature_count {
        masm_set_feature_aln(masm, feature_idx);
    }
}

/// Cache an ungapped string for each sequence in the underlying MSA.
#[track_caller]
pub fn masm_set_ungapped_seqs(masm: &mut MASM) {
    masm.ungapped_seqs.clear();
    let aln = masm
        .aln
        .as_ref()
        .expect("MASM::SetUngappedSeqs, m_Aln is null");
    for seq_idx in 0..masm.seq_count as usize {
        let seq = &aln.seqs[seq_idx];
        assert_eq!(seq.char_vec.len(), masm.col_count as usize);
        let mut ungapped_seq = String::new();
        for col in 0..masm.col_count as usize {
            let c = seq.char_vec[col];
            if c != '-' && c != '.' {
                ungapped_seq.push(c);
            }
        }
        masm.ungapped_seqs.push(ungapped_seq);
    }
}

/// Fill in the aligned-letter vector for one feature by looking up Mega profiles.
#[track_caller]
pub fn masm_set_feature_aln(masm: &mut MASM, feature_idx: uint) {
    assert!((feature_idx as usize) < masm.feature_aln_vec.len());
    masm.feature_aln_vec[feature_idx as usize].clear();
    masm.feature_aln_vec[feature_idx as usize].resize(masm.seq_count as usize, Vec::new());
    assert_eq!(masm.ungapped_seqs.len(), masm.seq_count as usize);
    let aln = masm
        .aln
        .as_ref()
        .expect("MASM::SetFeatureAln, m_Aln is null");
    for seq_idx in 0..masm.seq_count as usize {
        let ungapped_seq = &masm.ungapped_seqs[seq_idx];
        let mega_profile = mega_get_profile_by_seq(ungapped_seq, true)
            .expect("Mega::GetProfileBySeq returned null");
        let seq = &aln.seqs[seq_idx];
        assert_eq!(seq.char_vec.len(), masm.col_count as usize);
        let mut pos = 0_usize;
        for col in 0..masm.col_count as usize {
            let c = seq.char_vec[col];
            if c == '-' || c == '.' {
                masm.feature_aln_vec[feature_idx as usize][seq_idx].push(byte::MAX);
            } else {
                let letter = mega_profile[pos][feature_idx as usize];
                masm.feature_aln_vec[feature_idx as usize][seq_idx].push(letter);
                pos += 1;
            }
        }
    }
}

/// Build the substitution score matrix between a MASM and an unaligned query sequence.
#[track_caller]
pub fn masm_make_s_mx_sequence(masm: &MASM, q: &Sequence) -> Mx {
    let lm = masm.col_count as usize;
    let lq = q.char_vec.len();
    let mut smx = Mx {
        row_count: masm.col_count,
        col_count: lq as uint,
        data: vec![vec![0.0; lq]; lm],
        ..Mx::default()
    };
    for col in 0..lm {
        let mc = &masm.cols[col];
        let scores = masm_col_get_aa_scores(mc);
        for pos_q in 0..lq {
            let letter = match q.char_vec[pos_q] {
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
                _ => byte::MAX,
            };
            let mut score = 0.0_f32;
            if letter < 20 {
                score = scores[letter as usize];
            }
            smx.data[col][pos_q] = score;
        }
    }
    smx
}

/// Load a MASM previously written by `masm_to_file_l150`.
#[track_caller]
pub fn masm_from_file(masm: &mut MASM, file_name: &str) {
    if file_name.is_empty() {
        panic!("Missing MASM input file");
    }
    *masm = MASM::default();
    let mut file = text_file_text_file_l5(file_name, false);
    let line = text_file_get_line_x(&mut file, uint::MAX);
    let fields = split(line.trim_end_matches(['\r', '\n']), '\t');
    assert_eq!(fields.len(), 7);
    assert_eq!(fields[0], "MASM");
    masm.seq_count = str_to_uint_l1278(&fields[1], false);
    masm.col_count = str_to_uint_l1278(&fields[2], false);
    masm.feature_count = str_to_uint_l1278(&fields[3], false);
    masm.gap_open = str_to_float_l1204(&fields[4], false) as f32;
    masm.gap_ext = str_to_float_l1204(&fields[5], false) as f32;
    masm.label = fields[6].clone();
    masm.aa_feature_idx = uint::MAX;
    for i in 0..masm.feature_count as usize {
        let line = text_file_get_line_x(&mut file, uint::MAX);
        let fields = split(line.trim_end_matches(['\r', '\n']), '\t');
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "feature");
        assert_eq!(str_to_uint_l1278(&fields[1], false), i as uint);
        let feature_name = fields[2].clone();
        if feature_name == "AA" {
            masm.aa_feature_idx = i as uint;
        }
        masm.feature_names.push(feature_name);
        masm.alpha_sizes.push(str_to_uint_l1278(&fields[3], false));
    }
    for i in 0..masm.col_count as usize {
        let mut mc = MASMCol {
            masm: Some(Box::new(MASM {
                feature_count: masm.feature_count,
                feature_names: masm.feature_names.clone(),
                alpha_sizes: masm.alpha_sizes.clone(),
                aa_feature_idx: masm.aa_feature_idx,
                ..MASM::default()
            })),
            ..MASMCol::default()
        };
        masm_col_from_file(&mut mc, &mut file);
        assert_eq!(mc.col_index, i as uint);
        masm.cols.push(mc);
    }
}

/// Return the consensus amino-acid string built from per-column consensus characters.
#[track_caller]
pub fn masm_get_consensus_seq(masm: &MASM) -> String {
    let mut seq = String::new();
    for col in 0..masm.col_count as usize {
        seq.push(masm_col_get_consensus_aa_char(&masm.cols[col]));
    }
    seq
}
