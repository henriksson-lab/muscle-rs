// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Fast Smith-Waterman of a MASM against a single sequence with affine
/// gaps; returns score, endpoints and traceback string.
#[track_caller]
pub fn sw_fast_masm_seq(
    mem: &mut XDPMem,
    a: &MASM,
    b: &Sequence,
    open: f32,
    ext: f32,
) -> (f32, uint, uint, uint, uint, String) {
    let smx = masm_make_s_mx_sequence(a, b);
    sw_fast_s_mx(mem, &smx, -open, -ext)
}

/// CLI entry point: build a MASM from an MSA, save it, then align every
/// query sequence in `fasta_file_name` against the MASM.
#[track_caller]
pub fn cmd_swmasm_seq<FLoadMega>(
    aln_file_name: &str,
    mega_file_name: &str,
    fasta_file_name: &str,
    output_file_name: &str,
    mut load_mega: FLoadMega,
) -> String
where
    FLoadMega: FnMut(&str),
{
    load_mega(mega_file_name);

    let mut aln = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut aln, aln_file_name, false);

    let gap_open = 4.0_f32;
    let gap_ext = 0.5_f32;

    let mut m = MASM::default();
    masm_from_msa(&mut m, &aln, "FomMSA", gap_open, gap_ext);
    masm_to_file_l150(&m, output_file_name);

    let mut query = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut query, fasta_file_name, true);

    let mut mem = XDPMem::default();
    let mut out = String::new();
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
    for q in &query.seqs {
        let (score, loi, loj, _leni, _lenj, path) =
            sw_fast_masm_seq(&mut mem, &m, q, gap_open, gap_ext);
        let label = q.label.chars().take(16).collect::<String>();
        out.push_str(&format!(
            "{:>10}  {:>16}  {:7}  {:7}  {}\n\n",
            format_g3(score),
            label,
            loi,
            loj,
            path
        ));
    }
    out
}
