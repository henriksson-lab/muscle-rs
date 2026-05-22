// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Regression case that compares the MASM-mega and BLOSUM62 SW
/// implementations on a small problematic input.
#[track_caller]
pub fn bug_l13() -> String {
    let gap_open = -1.0;
    let gap_ext = -0.4;
    let mut st = SWTester {
        x: Some(SWer::default()),
        y: Some(SWer::default()),
        ..SWTester::default()
    };
    let mut out = String::new();
    if let Some(log) = sw_tester_run_ab(
        &mut st,
        "EVRDYIQ",
        "RQGEG",
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |s, lo_a, lo_b, path| {
            s_wer_fast_seqs_aa_blosum62_sw(s, gap_open, gap_ext, lo_a, lo_b, path)
        },
        |_lo_a, _lo_b, _path| 0.0,
    ) {
        out.push_str(&log);
    }
    out.push_str(&sw_tester_stats(&st));
    out
}

/// Test harness that cross-checks the fast and path-scorer SW
/// implementations on a synthetic sequence pair.
#[track_caller]
pub fn cmd_swtest() -> String {
    let gap_open = -1.0;
    let gap_ext = -0.4;
    let mut st = SWTester {
        x: Some(SWer::default()),
        y: Some(SWer::default()),
        ..SWTester::default()
    };
    let ps_x = PathScorerAABLOSUM62 {
        gap_open,
        gap_ext,
        seq_a: "SEQV".to_string(),
        seq_b: "EQ".to_string(),
        base: PathScorer {
            la: 4,
            lb: 2,
            ..PathScorer::default()
        },
    };
    let mut ps_y = PathScorerAABLOSUM62 {
        gap_open,
        gap_ext,
        ..PathScorerAABLOSUM62::default()
    };
    let mut out = String::new();
    if let Some(log) = sw_tester_run_ab(
        &mut st,
        "SEQV",
        "EQ",
        |s, lo_a, lo_b, path| {
            s_wer_fast_seqs_aa_blosum62_sw(s, gap_open, gap_ext, lo_a, lo_b, path)
        },
        |s, lo_a, lo_b, path| s_wer_ps_sw(s, &mut ps_y, lo_a, lo_b, path),
        |lo_a, lo_b, path| {
            path_scorer_get_local_score(4, 2, lo_a, lo_b, path, |from, to, pa, pb| {
                path_scorer_get_score(
                    from,
                    to,
                    pa,
                    pb,
                    |pa, pb| path_scorer_aa_blosum62_get_match_score(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_mm(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_md(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_mi(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_dm(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_dd(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_im(&ps_x, pa, pb),
                    |pa, pb| path_scorer_aa_blosum62_get_score_ii(&ps_x, pa, pb),
                )
            })
        },
    ) {
        out.push_str(&log);
    }
    out.push_str(&sw_tester_stats(&st));
    out
}
