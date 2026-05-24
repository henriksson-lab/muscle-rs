// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Regression reproducer for the SCOREDIFF bug between Enum_MASM_Mega and
/// Simple_MASM_Mega aligners on `VDA|KMY|STN` vs `MHS`.
#[track_caller]
pub fn bug_swtestmm_l13() -> String {
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
        "Enum_MASM_Mega",
        "VDA|KMY|STN",
        "MHS",
        true,
        |s, lo_a, lo_b, path| s_wer_enum_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |lo_a, lo_b, path, _trace| {
            let rows_a = split("VDA|KMY|STN", '|');
            let ma = make_masm_rows(&rows_a, gap_open, gap_ext);
            let pb = make_mega_profile("MHS");
            let mut ps = PathScorerMASMMega::default();
            path_scorer_masm_mega_init(&mut ps, &ma, &pb);
            path_scorer_get_local_score(3, 3, lo_a, lo_b, path, |from, to, pa, pb| {
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
    if let Some(log) = sw_tester_run_xab(
        &mut st,
        "Simple_MASM_Mega",
        "VDA|KMY|STN",
        "MHS",
        true,
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |lo_a, lo_b, path, _trace| {
            let rows_a = split("VDA|KMY|STN", '|');
            let ma = make_masm_rows(&rows_a, gap_open, gap_ext);
            let pb = make_mega_profile("MHS");
            let mut ps = PathScorerMASMMega::default();
            path_scorer_masm_mega_init(&mut ps, &ma, &pb);
            path_scorer_get_local_score(3, 3, lo_a, lo_b, path, |from, to, pa, pb| {
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
    out
}

/// Stress-test loop comparing Simple_MASM_Mega against MASM_Mega_Seqs on
/// random sequences.
#[track_caller]
pub fn test_seqs() -> String {
    let gap_open = -1.0;
    let gap_ext = -0.4;
    let mut st = SWTester {
        x: Some(SWer::default()),
        y: Some(SWer::default()),
        ..SWTester::default()
    };
    sw_tester_set_x_name(&mut st, "Simple_MASM_Mega");
    sw_tester_set_y_name(&mut st, "MASM_Mega_Seqs");
    let mut out = sw_tester_run_random_seqs_iters(
        &mut st,
        3,
        5,
        1000,
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |s, lo_a, lo_b, path| s_wer_masm_mega_seqs_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |_lo_a, _lo_b, _path| 0.0,
    );
    out.push_str(&sw_tester_stats(&st));
    out
}

/// Entry point for the `swtestmm` command: random MSA-vs-sequence stress
/// test comparing Enum_MASM_Mega and Simple_MASM_Mega.
#[track_caller]
pub fn cmd_swtestmm() -> String {
    let gap_open = -1.0;
    let gap_ext = -0.4;
    let mut st = SWTester {
        x: Some(SWer::default()),
        y: Some(SWer::default()),
        ..SWTester::default()
    };
    sw_tester_set_x_name(&mut st, "Enum_MASM_Mega");
    sw_tester_set_y_name(&mut st, "Simple_MASM_Mega");
    let mut out = sw_tester_run_random_msa_seq_iters(
        &mut st,
        1,
        5,
        3,
        7,
        1000,
        |s, lo_a, lo_b, path| s_wer_enum_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |_lo_a, _lo_b, _path| 0.0,
    );
    out.push_str(&sw_tester_stats(&st));
    out
}
