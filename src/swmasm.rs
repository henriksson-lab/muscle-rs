// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Fast Smith-Waterman of a MASM against a mega-profile column matrix `b`,
/// using the MASM's gap parameters.
#[track_caller]
pub fn sw_fast_masm(
    mem: &mut XDPMem,
    a: &MASM,
    b: &[Vec<byte>],
) -> (f32, uint, uint, uint, uint, String) {
    let open = a.gap_open;
    let ext = a.gap_ext;
    let smx = masm_make_s_mx(a, b);
    sw_fast_s_mx(mem, &smx, -open, -ext)
}

/// CLI entry point: align one MASM against every mega-profile loaded from
/// `mega_file_name` and emit score table plus traceback output.
#[track_caller]
pub fn cmd_swmasm<FLoadMega>(
    masm_file_name: &str,
    mega_file_name: &str,
    output_file_name: &str,
    mut load_mega: FLoadMega,
) -> String
where
    FLoadMega: FnMut(&str),
{
    load_mega(mega_file_name);

    let mut m = MASM::default();
    masm_from_file(&mut m, masm_file_name);
    let label_m = m.label.clone();

    let query_profile_count = MEGA_STATE.lock().unwrap().profiles.len() as uint;
    let mut mem = XDPMem::default();
    let mut out = String::new();
    let mut tab = String::new();
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
    for i in 0..query_profile_count {
        let q = mega_get_profile(i);
        let label_q = mega_get_label(i);
        let (score, loi, loj, _leni, _lenj, path) = sw_fast_masm(&mut mem, &m, &q);
        if !path.is_empty() {
            out.push_str(&write_local_aln_masm(
                &label_m, &m, &label_q, &q, loi, loj, &path,
            ));
        }
        let score_s = format_g3(score);
        out.push_str(&format!("Score = {score_s}\n\n"));
        tab.push_str(&format!("{}\t{}\t{}\n", label_m, label_q, score_s));
    }
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &tab).expect("failed to write SWMASM output");
    }
    out
}
