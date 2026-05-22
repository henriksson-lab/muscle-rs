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
    if let Some(log) = sw_tester_run_ab(
        &mut st,
        "VDA|KMY|STN",
        "MHS",
        |s, lo_a, lo_b, path| s_wer_enum_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |s, lo_a, lo_b, path| s_wer_simple_masm_mega_sw(s, gap_open, gap_ext, lo_a, lo_b, path),
        |_lo_a, _lo_b, _path| 0.0,
    ) {
        out.push_str(&log);
    }
    out.push_str(&sw_tester_stats(&st));
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
