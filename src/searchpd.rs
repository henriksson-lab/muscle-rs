// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Run pair Viterbi alignment and return the protein distance between the aligned rows.
#[track_caller]
pub fn align_and_prot_dist<FViterbi, FDist>(
    seqi: &[byte],
    li: uint,
    seqj: &[byte],
    lj: uint,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> f64
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let pi = viterbi_fast_mem(seqi, li, seqj, lj);
    let (row_x, row_y) = make_aln_rows_l45(seqi, li, seqj, lj, &pi);
    let col_count = pi.path.len() as uint;
    assert_eq!(row_x.len() as uint, col_count);
    assert_eq!(row_y.len() as uint, col_count);
    get_prot_dist(&row_x, &row_y, col_count)
}

/// `searchpd` command: align each query against a database and report hits within MaxPD protein distance.
#[track_caller]
pub fn cmd_searchpd<FViterbi, FDist>(
    input_file_name: &str,
    db_file_name: &str,
    max_pd: f64,
    tsv_out_file_name: &str,
    mut viterbi_fast_mem: FViterbi,
    mut get_prot_dist: FDist,
) -> String
where
    FViterbi: FnMut(&[byte], uint, &[byte], uint) -> PathInfo,
    FDist: FnMut(&str, &str, uint) -> f64,
{
    let mut query = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut query, input_file_name, true);

    let mut db = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut db, db_file_name, true);

    let is_nucleo = multi_sequence_guess_is_nucleo(&query);
    set_alpha_l209(if is_nucleo {
        ALPHA::ALPHA_Nucleo
    } else {
        ALPHA::ALPHA_Amino
    });

    let format_g3 = |d: f64| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 3 {
            let raw = format!("{d:.2e}");
            let (mantissa, exponent) = raw.split_once('e').unwrap();
            let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
            let exp_value = exponent.parse::<i32>().unwrap();
            let sign = if exp_value >= 0 { '+' } else { '-' };
            format!("{mantissa}e{sign}{:02}", exp_value.abs())
        } else {
            let decimals = (2 - exp).max(0) as usize;
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

    let mut out = String::new();
    for q in &query.seqs {
        let seq_q = sequence_get_seq_as_string(q).into_bytes();
        let lq = seq_q.len() as uint;
        for t in &db.seqs {
            let seq_t = sequence_get_seq_as_string(t).into_bytes();
            let lt = seq_t.len() as uint;
            let d = align_and_prot_dist(
                &seq_q,
                lq,
                &seq_t,
                lt,
                |seqi, li, seqj, lj| viterbi_fast_mem(seqi, li, seqj, lj),
                |row_x, row_y, col_count| get_prot_dist(row_x, row_y, col_count),
            );
            if d <= max_pd {
                out.push_str(&format!("{}\t{}\t{}\n", q.label, t.label, format_g3(d)));
            }
        }
    }
    if !tsv_out_file_name.is_empty() {
        std::fs::write(tsv_out_file_name, &out).expect("failed to write searchpd TSV");
    }
    out
}
