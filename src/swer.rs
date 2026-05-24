// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct SWer {
    pub la: uint,
    pub lb: uint,
    pub a: String,
    pub rows_a: Vec<String>,
    pub b: String,
} // original: SWer (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerEnumSeqsAABLOSUM62; // original: SWer_Enum_Seqs_AA_BLOSUM62 (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerFastSeqsAABLOSUM62; // original: SWer_Fast_Seqs_AA_BLOSUM62 (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerSimpleSeqsAABLOSUM62; // original: SWer_Simple_Seqs_AA_BLOSUM62 (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerMASMMegaSeqs; // original: SWer_MASM_Mega_Seqs (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerSimpleMASMMega; // original: SWer_Simple_MASM_Mega (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerMASMMega; // original: SWer_MASM_Mega (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerEnumMASMMega; // original: SWer_Enum_MASM_Mega (muscle/src/swer.h)

#[derive(Clone, Debug, Default)]
pub struct SWerPS; // original: SWer_PS (muscle/src/swer.h)

/// Pretty-print a DP matrix, rendering `f32::MAX` cells as `*`.
#[track_caller]
pub fn log_m(msg: &str, m: &[Vec<f32>]) -> String {
    let la = m.len();
    assert!(la > 0);
    let lb = m[0].len();
    let mut out = String::new();
    out.push_str(&format!("LogM({msg})\n"));
    out.push_str("      ");
    for j in 0..lb {
        out.push_str(&format!(" {j:10}"));
    }
    out.push('\n');
    for i in 0..la {
        out.push_str(&format!("{i:3} | "));
        for j in 0..lb {
            let x = m[i][j];
            if x == f32::MAX {
                out.push_str(&format!(" {:>10}", "*"));
            } else {
                out.push_str(&format!(" {:>10}", float_to_str_l1385(f64::from(x))));
            }
        }
        out.push('\n');
    }
    out
}

/// Compare two forward DP matrices cell-by-cell; panics on disagreement.
#[track_caller]
pub fn cmp_fwd_m(m1: &[Vec<f32>], m2: &[Vec<f32>]) {
    let la = m1.len();
    assert!(la > 0);
    assert_eq!(m2.len(), la);
    let lb = m1[0].len();
    assert_eq!(m2[0].len(), lb);
    for i in 0..la {
        for j in 0..lb {
            let a = m1[i][j];
            let b = m2[i][j];
            if (a - b).abs() > 1e-6 && !(a.is_nan() && b.is_nan()) {
                let _m1_log = log_m("M1", m1);
                let _m2_log = log_m("M2", m2);
                panic!("CmpFwdM");
            }
        }
    }
}

/// Build a single-row MASM from `seq` using the supplied gap penalties.
#[track_caller]
pub fn make_masm_seq(seq: &str, gap_open: f32, gap_ext: f32) -> MASM {
    let mut aln = MultiSequence::default();
    multi_sequence_from_strings(&mut aln, &["LABEL".to_string()], &[seq.to_string()]);
    let mut m = MASM::default();
    mega_from_msa_aa_only(&aln, gap_open, gap_ext);
    masm_from_msa(&mut m, &aln, "MSA", -gap_open, -gap_ext);
    m
}

/// Build a multi-row MASM from aligned `rows`; all rows must share length.
#[track_caller]
pub fn make_masm_rows(rows: &[String], gap_open: f32, gap_ext: f32) -> MASM {
    let seq_count = rows.len();
    let mut col_count = uint::MAX;
    for i in 0..seq_count {
        let row = &rows[i];
        if i == 0 {
            col_count = row.len() as uint;
        } else {
            assert_eq!(row.len() as uint, col_count);
        }
    }
    let labels = vec!["Row".to_string(); seq_count];
    let mut aln = MultiSequence::default();
    multi_sequence_from_strings(&mut aln, &labels, rows);
    let mut m = MASM::default();
    mega_from_msa_aa_only(&aln, gap_open, gap_ext);
    masm_from_msa(&mut m, &aln, "Rows", -gap_open, -gap_ext);
    m
}

/// Encode `seq` as a one-column mega profile of amino-acid letter indices.
#[track_caller]
pub fn make_mega_profile(seq: &str) -> Vec<Vec<byte>> {
    let mut prof = Vec::new();
    for c in seq.bytes() {
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
        prof.push(col);
    }
    prof
}

/// Count residues consumed from sequence A by `path` (M and D states).
#[track_caller]
pub fn s_wer_get_na(path: &str) -> uint {
    let mut n = 0;
    for c in path.chars() {
        if c == 'M' || c == 'D' {
            n += 1;
        }
    }
    n
}

/// Count residues consumed from sequence B by `path` (M and I states).
#[track_caller]
pub fn s_wer_get_nb(path: &str) -> uint {
    let mut n = 0;
    for c in path.chars() {
        if c == 'M' || c == 'I' {
            n += 1;
        }
    }
    n
}

/// Generic SWer driver: install `a`/`b` (and the `|`-split rows of `a`) on
/// `s`, invoke the caller's SW closure, and sanity-check the returned path.
#[track_caller]
pub fn s_wer_run<FSW>(
    s: &mut SWer,
    a: &str,
    b: &str,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
    mut sw: FSW,
) -> f32
where
    FSW: FnMut(&mut SWer, &mut uint, &mut uint, &mut String) -> f32,
{
    s.rows_a.clear();
    s.a = a.to_string();
    s.b = b.to_string();
    s.rows_a = split(a, '|');
    s.la = s.rows_a[0].len() as uint;
    s.lb = s.b.len() as uint;
    let score = sw(s, lo_a, lo_b, path);
    if score <= 0.0 {
        return 0.0;
    }
    let na = s_wer_get_na(path);
    let nb = s_wer_get_nb(path);
    let hi_a = *lo_a + na - 1;
    let hi_b = *lo_b + nb - 1;
    assert!(hi_a < s.la);
    assert!(hi_b < s.lb);
    score
}

/// Enumeration callback: scores `(pos_a, pos_b, path)` and keeps the best.
#[track_caller]
pub fn on_path_l158<F>(
    state: &mut TestSwMmBruteState,
    pos_a: uint,
    pos_b: uint,
    path: &str,
    mut get_local_score: F,
) where
    F: FnMut(uint, uint, &str) -> f32,
{
    let score = get_local_score(pos_a, pos_b, path);
    if score > state.best_score {
        state.best_score = score;
        state.best_path = path.to_string();
        state.best_pos_a = pos_a;
        state.best_pos_b = pos_b;
    }
}

/// Install the open/extend gap penalties on the BLOSUM62 enum SWer.
#[track_caller]
pub fn s_wer_enum_seqs_aa_blosum62_set_gaps(
    gap_open: &mut f32,
    gap_ext: &mut f32,
    open: f32,
    ext: f32,
) {
    *gap_open = open;
    *gap_ext = ext;
}

/// Brute-force enumerated SW between two AA sequences under BLOSUM62.
#[track_caller]
pub fn s_wer_enum_seqs_aa_blosum62_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);
    let ps = PathScorerAABLOSUM62 {
        gap_open,
        gap_ext,
        seq_a: s.a.clone(),
        seq_b: s.b.clone(),
        base: PathScorer {
            la: s.la,
            lb: s.lb,
            ..PathScorer::default()
        },
    };
    let (score, loi, loj, aln_path) = sw_enum_dp(s.la, s.lb, |pos_a, pos_b, p| {
        path_scorer_get_local_score(s.la, s.lb, pos_a, pos_b, p, |from, to, pa, pb| {
            path_scorer_get_score(
                from,
                to,
                pa,
                pb,
                |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps, pa, pb),
                |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps, pa, pb),
            )
        })
    });
    *lo_a = loi;
    *lo_b = loj;
    *path = aln_path;
    score
}

/// Fast (bit-packed) SW between two AA sequences under BLOSUM62.
#[track_caller]
pub fn s_wer_fast_seqs_aa_blosum62_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);
    let mut mem = XDPMem::default();
    let (score, loi, loj, _leni, _lenj, aln_path) =
        sw_fast_strings_blosum62(&mut mem, &s.a, &s.b, gap_open, gap_ext);
    *lo_a = loi;
    *lo_b = loj;
    *path = aln_path;
    score
}

/// Reference SW between two AA sequences under BLOSUM62 via `sw_simple`.
#[track_caller]
pub fn s_wer_simple_seqs_aa_blosum62_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);
    let ps = PathScorerAABLOSUM62 {
        gap_open,
        gap_ext,
        seq_a: s.a.clone(),
        seq_b: s.b.clone(),
        base: PathScorer {
            la: s.a.len() as uint,
            lb: s.b.len() as uint,
            ..PathScorer::default()
        },
    };
    sw_simple(
        ps.base.la,
        ps.base.lb,
        lo_a,
        lo_b,
        path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps, pa, pb),
    )
}

/// SW between a sequence wrapped as a MASM and a single-column mega profile.
#[track_caller]
pub fn s_wer_masm_mega_seqs_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);

    let ma = make_masm_seq(&s.a, gap_open, gap_ext);
    let pb = make_mega_profile(&s.b);
    let mut mem = XDPMem::default();
    let (score, loi, loj, _leni, _lenj, aln_path) =
        sw_fast_masm_mega_prof(&mut mem, &ma, &pb, gap_open, gap_ext);
    *lo_a = loi;
    *lo_b = loj;
    *path = aln_path;
    score
}

/// Reference SW between a multi-row MASM and a mega profile, cross-checked
/// against the enum and `sw_simple_fwd_m` implementations.
#[track_caller]
pub fn s_wer_simple_masm_mega_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);

    let pb = make_mega_profile(&s.b);
    let ma = make_masm_rows(&s.rows_a, gap_open, gap_ext);
    let mut ps = PathScorerMASMMega::default();
    path_scorer_masm_mega_init(&mut ps, &ma, &pb);
    ps.base.la = s.rows_a[0].len() as uint;
    ps.base.lb = s.b.len() as uint;

    let score = sw_simple(
        ps.base.la,
        ps.base.lb,
        lo_a,
        lo_b,
        path,
        |pa, pb| path_scorer_masm_mega_get_match_score(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_mm(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_md(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_mi(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_dm(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_dd(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_im(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_ii(&ps, pa, pb),
    );

    let mut enum_fwd_m = Vec::new();
    let mut path2 = String::new();
    let (score2, _enum_lo_a, _enum_lo_b, enum_path) = sw_enum_dp_fwd_m(
        ps.base.la,
        ps.base.lb,
        &mut enum_fwd_m,
        |pos_a, pos_b, p| {
            path_scorer_get_local_score(
                ps.base.la,
                ps.base.lb,
                pos_a,
                pos_b,
                p,
                |from, to, pa, pb| {
                    path_scorer_get_score(
                        from,
                        to,
                        pa,
                        pb,
                        |pa, pb| path_scorer_masm_mega_get_match_score(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_mm(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_md(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_mi(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_dm(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_dd(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_im(&ps, pa, pb),
                        |pa, pb| path_scorer_masm_mega_get_score_ii(&ps, pa, pb),
                    )
                },
            )
        },
    );
    let mut simple_fwd_m = Vec::new();
    let score3 = sw_simple_fwd_m(
        ps.base.la,
        ps.base.lb,
        lo_a,
        lo_b,
        &mut path2,
        &mut simple_fwd_m,
        |pa, pb| path_scorer_masm_mega_get_match_score(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_mm(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_md(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_mi(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_dm(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_dd(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_im(&ps, pa, pb),
        |pa, pb| path_scorer_masm_mega_get_score_ii(&ps, pa, pb),
    );
    assert!((score - score2).abs() < 1e-3);
    assert!((score - score3).abs() < 1e-3);
    let _ = (enum_path, path2);
    score
}

/// Fast SW between a multi-row MASM and a mega profile.
#[track_caller]
pub fn s_wer_masm_mega_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);

    let ma = make_masm_rows(&s.rows_a, gap_open, gap_ext);
    let pb = make_mega_profile(&s.b);
    let mut mem = XDPMem::default();
    let (score, loi, loj, _leni, _lenj, aln_path) =
        sw_fast_masm_mega_prof(&mut mem, &ma, &pb, gap_open, gap_ext);
    *lo_a = loi;
    *lo_b = loj;
    *path = aln_path;
    score
}

/// Brute-force enumerated SW between a multi-row MASM and a mega profile.
#[track_caller]
pub fn s_wer_enum_masm_mega_sw(
    s: &mut SWer,
    gap_open: f32,
    gap_ext: f32,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    assert!(gap_open != f32::MAX && gap_open < 0.0);
    assert!(gap_ext != f32::MAX && gap_ext < 0.0);

    let rows_a = split(&s.a, '|');
    let ma = make_masm_rows(&rows_a, gap_open, gap_ext);
    let pb = make_mega_profile(&s.b);
    let mut ps = PathScorerMASMMega::default();
    path_scorer_masm_mega_init(&mut ps, &ma, &pb);
    ps.base.la = s.la;
    ps.base.lb = s.lb;
    let (score, loi, loj, aln_path) = sw_enum_dp(ps.base.la, ps.base.lb, |pos_a, pos_b, p| {
        path_scorer_get_local_score(
            ps.base.la,
            ps.base.lb,
            pos_a,
            pos_b,
            p,
            |from, to, pa, pb| {
                path_scorer_get_score(
                    from,
                    to,
                    pa,
                    pb,
                    |pa, pb| path_scorer_masm_mega_get_match_score(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_mm(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_md(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_mi(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_dm(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_dd(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_im(&ps, pa, pb),
                    |pa, pb| path_scorer_masm_mega_get_score_ii(&ps, pa, pb),
                )
            },
        )
    });
    *lo_a = loi;
    *lo_b = loj;
    *path = aln_path;
    score
}

/// SW driven by a generic `PathScorerAABLOSUM62`, cross-checked against
/// `sw_simple2`.
#[track_caller]
pub fn s_wer_ps_sw(
    s: &mut SWer,
    ps: &mut PathScorerAABLOSUM62,
    lo_a: &mut uint,
    lo_b: &mut uint,
    path: &mut String,
) -> f32 {
    ps.seq_a = s.a.clone();
    ps.seq_b = s.b.clone();
    ps.base.la = s.la;
    ps.base.lb = s.lb;

    let mut mem = XDPMem::default();
    let score = swps(
        &mut mem,
        ps.base.la,
        ps.base.lb,
        lo_a,
        lo_b,
        path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(ps, pa, pb),
    );

    let mut mem2 = XDPMem::default();
    let mut lo_a2 = uint::MAX;
    let mut lo_b2 = uint::MAX;
    let score2 = sw_simple2(
        &mut mem2,
        ps.base.la,
        ps.base.lb,
        &mut lo_a2,
        &mut lo_b2,
        path,
        |pa, pb| path_scorer_aa_blosum62_get_match_score(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mm(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_md(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_mi(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dm(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_dd(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_im(ps, pa, pb),
        |pa, pb| path_scorer_aa_blosum62_get_score_ii(ps, pa, pb),
    );
    assert!((score - score2).abs() < 1e-3);
    score
}
