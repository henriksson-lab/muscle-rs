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
    if let Some(log) = sw_tester_run_xab(
        &mut st,
        "Simple_MASM_Mega",
        "EVRDYIQ",
        "RQGEG",
        true,
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |lo_a, lo_b, path, _trace| {
            let rows_a = split("EVRDYIQ", '|');
            let ma = make_masm_rows(&rows_a, gap_open, gap_ext);
            let pb = make_mega_profile("RQGEG");
            let mut ps = PathScorerMASMMega::default();
            path_scorer_masm_mega_init(&mut ps, &ma, &pb);
            path_scorer_get_local_score(7, 5, lo_a, lo_b, path, |from, to, pa, pb| {
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
            })
        },
    ) {
        out.push_str(&log);
    }
    let ps = PathScorerAABLOSUM62 {
        gap_open,
        gap_ext,
        seq_a: "EVRDYIQ".to_string(),
        seq_b: "RQGEG".to_string(),
        base: PathScorer {
            la: 7,
            lb: 5,
            ..PathScorer::default()
        },
    };
    if let Some(log) = sw_tester_run_xab(
        &mut st,
        "Fast_Seqs_AA_BLOSUM62",
        "EVRDYIQ",
        "RQGEG",
        true,
        |s, lo_a, lo_b, path| {
            s_wer_fast_seqs_aa_blosum62_sw(s, gap_open, gap_ext, lo_a, lo_b, path)
        },
        |lo_a, lo_b, path, _trace| {
            path_scorer_get_local_score(7, 5, lo_a, lo_b, path, |from, to, pa, pb| {
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
        },
    ) {
        out.push_str(&log);
    }
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
    sw_tester_set_x_name(&mut st, "Fast_Seqs_AA_BLOSUM62");
    sw_tester_set_y_name(&mut st, "PS");
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
