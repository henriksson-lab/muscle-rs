// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Command implementation: print summary statistics for an MSA (seq counts, gap distributions, ...).
#[track_caller]
pub fn cmd_msastats(msa_file_name: &str, max_gap_fract: Option<f64>) -> String {
    let aln = msa_from_fasta_file_preserve_case(msa_file_name);

    let seq_count = aln.seqs.len() as uint;
    let col_count = multi_sequence_get_col_count(&aln);

    let mut out = String::new();
    out.push_str(&format!("{seq_count:10}  Sequences\n"));
    out.push_str(&format!("{col_count:10}  Columns\n"));

    let mut ls = Vec::<uint>::new();
    for seq_index in 0..seq_count {
        let l = msa_get_ungapped_seq_length(&aln, seq_index);
        ls.push(l);
    }
    let mut q = get_quarts(&ls);

    out.push_str(&format!("{:10.1}  Mean seq length", q.avg));
    out.push_str(&format!(
        "  min {}, median {}, max {}\n",
        q.min, q.med, q.max
    ));

    let mut gap_pcts = Vec::<uint>::new();
    let mut gap0 = 0_u32;
    let mut gap_ok = 0_u32;
    let max_gap_pct = max_gap_fract.unwrap_or(0.5) * 100.0;
    let mut lower_col_count = 0_u32;
    let mut upper_col_count = 0_u32;
    let mut mixed_col_count = 0_u32;
    let mut dot_count = 0_u32;
    let mut dash_count = 0_u32;
    for col in 0..col_count {
        let (nu, nl, ng, n_dots, n_dashes) = msa_get_upper_lower_gap_count(&aln, col);
        dot_count += n_dots;
        dash_count += n_dashes;
        if nu > 0 && nl == 0 {
            upper_col_count += 1;
        } else if nu == 0 && nl > 0 {
            lower_col_count += 1;
        } else if nu > 0 && nl > 0 {
            mixed_col_count += 1;
        }
        let gap_pct = (100 * ng) / seq_count;
        if ng == 0 {
            gap0 += 1;
        }
        if f64::from(gap_pct) <= max_gap_pct {
            gap_ok += 1;
        }
        gap_pcts.push(gap_pct);
    }
    q = get_quarts(&gap_pcts);

    let dash_gap_count = dash_count + dot_count;
    let dash_pct = if dash_gap_count == 0 {
        assert_eq!(dash_count, 0);
        0.0
    } else {
        100.0 * f64::from(dash_count) / f64::from(dash_gap_count)
    };
    let gap0_pct = if col_count == 0 {
        assert_eq!(gap0, 0);
        0.0
    } else {
        100.0 * f64::from(gap0) / f64::from(col_count)
    };
    let gap_ok_pct = if col_count == 0 {
        assert_eq!(gap_ok, 0);
        0.0
    } else {
        100.0 * f64::from(gap_ok) / f64::from(col_count)
    };

    out.push_str(&format!("{:10.1}  Mean col gap pct,", q.avg));
    out.push_str(&format!(
        " min {}, median {}, max {}\n",
        q.min, q.med, q.max
    ));
    out.push_str(&format!(
        "{gap0:10}  Cols with no gaps ({gap0_pct:.1}% of cols)\n"
    ));
    out.push_str(&format!(
        "{gap_ok:10}  Cols with <{max_gap_pct:.1}% gaps ({gap_ok_pct:.1}% of cols)\n"
    ));
    out.push_str(&format!(
        "{upper_col_count:10}  Upper-case ({lower_col_count} lower, {mixed_col_count} mixed)\n"
    ));
    out.push_str(&format!("{dash_pct:9.1}%  Dash gaps\n"));
    out.push('\n');
    out
}
