// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// `qscore2` subcommand: score a single test MSA against a reference and return
/// the formatted `Q`/`TC` line.
#[track_caller]
pub fn cmd_qscore2(test_file_name: &str, ref_file_name: &str, max_gap_fract: f64) -> String {
    let name = get_base_name(test_file_name);

    let test = msa_from_fasta_file_l95(test_file_name);
    let ref_msa = msa_from_fasta_file_preserve_case(ref_file_name);

    let mut qs = QScorer {
        max_gap_fract,
        ..QScorer::default()
    };
    q_scorer_run_l337(&mut qs, &name, &test, &ref_msa);
    format!("{}: Q={:.4}, TC={:.4}\n", name, qs.q, qs.tc)
}

/// `qscoredir` subcommand: score every named test/ref MSA pair in two
/// directories and emit per-name plus aggregate statistics.
#[track_caller]
pub fn cmd_qscoredir(
    names_file_name: &str,
    test_dir: &str,
    ref_dir: &str,
    output_file_name: &str,
    max_gap_fract: f64,
) -> String {
    let mut test_dir = test_dir.to_string();
    let mut ref_dir = ref_dir.to_string();
    dirize(&mut test_dir);
    dirize(&mut ref_dir);

    let names = read_strings_from_file(names_file_name);
    let name_count = names.len() as uint;
    let mut sum_q = 0.0_f32;
    let mut sum_tc = 0.0_f32;
    let mut n = 0_u32;
    let mut m = 0_u32;
    let mut out = String::new();
    let mut avg_q = 0.0_f32;
    let mut avg_tc = 0.0_f32;

    for (i, name) in names.iter().enumerate() {
        let _ = progress_step(
            i as uint,
            name_count,
            &format!("{test_dir}  Q {avg_q:.2} TC {avg_tc:.2}"),
        );
        let test_file_name = format!("{test_dir}{name}");
        let ref_file_name = format!("{ref_dir}{name}");
        if !stdio_file_exists(&test_file_name) {
            log(&warning(&format!("Not found {test_file_name}")));
            continue;
        }
        n += 1;

        let test = msa_from_fasta_file_l95(&test_file_name);
        let ref_msa = msa_from_fasta_file_preserve_case(&ref_file_name);

        let mut qs = QScorer {
            max_gap_fract,
            ..QScorer::default()
        };
        let ok = q_scorer_run_l346(&mut qs, name, &test, &ref_msa, false);
        if ok {
            m += 1;
            out.push_str(&format!("set={name}\tq={:.4}\ttc={:.4}\n", qs.q, qs.tc));
        } else {
            out.push_str(&format!("set={name}\tNOMATCH\n"));
        }

        sum_q += qs.q;
        sum_tc += qs.tc;
        avg_q = sum_q / n as f32;
        avg_tc = sum_tc / n as f32;
    }

    avg_q = 0.0;
    avg_tc = 0.0;
    if m > 0 {
        avg_q = sum_q / m as f32;
        avg_tc = sum_tc / m as f32;
    }
    out.push_str(&format!(
        "testdir={test_dir}\tn={name_count}\tN={n}\tM={m}\tavgq={avg_q:.4}\tavgtc={avg_tc:.4}\n"
    ));
    if !output_file_name.is_empty() {
        std::fs::write(output_file_name, &out).expect("failed to write qscoredir output");
    }
    out
}
