// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Runs the benchmark suite against a reference directory, writing per-case
/// TC scores to a TSV and returning the populated Bench plus a summary log line.
#[track_caller]
pub fn cmd_bench<FRunCase>(
    input_file_name: &str,
    ref_dir: &str,
    tsv_out_file_name: &str,
    ap: &M3AlnParams,
    run_case: FRunCase,
) -> (Bench, String)
where
    FRunCase: FnMut(&M3AlnParams, &str, &MultiSequence, &MultiSequence, bool) -> (f64, f64),
{
    let mut bench = Bench::default();
    let mut ref_dir = ref_dir.to_string();
    dirize(&mut ref_dir);
    bench_load(&mut bench, input_file_name, &ref_dir);
    bench.show_progress = true;
    let msa_count = bench.inputs.len() as uint;

    bench_run(&mut bench, ap, false, run_case);
    bench_t_cs_to_file(&bench, tsv_out_file_name);

    let log = format!(
        "AvgQ={:.3} AvgTC={:.3} N={}\n",
        bench.mean_q, bench.mean_tc, msa_count
    );
    (bench, log)
}

/// Sweeps BLOSUM tables, parameter groups, and perturbation seeds, running the
/// benchmark for each combination and emitting a log and optional TSV.
#[track_caller]
pub fn cmd_bench_blosums<FRunCase>(
    input_file_name: &str,
    ref_dir: &str,
    tsv_out_file_name: &str,
    mut run_case: FRunCase,
) -> (Bench, String, Option<String>)
where
    FRunCase: FnMut(&M3AlnParams, &str, &MultiSequence, &MultiSequence, bool) -> (f64, f64),
{
    let mut bench = Bench::default();
    let mut ref_dir = ref_dir.to_string();
    dirize(&mut ref_dir);
    bench_load(&mut bench, input_file_name, &ref_dir);
    bench.show_progress = true;
    let msa_count = bench.inputs.len() as uint;

    let mut tsv = String::new();
    if !tsv_out_file_name.is_empty() {
        tsv.push_str("BLOSUM\tParamSet\tQ\tTC\tPerturbSeed\tDelta\n");
    }
    let mut log = String::new();
    let format_g3 = |x: f32| -> String {
        if x == 0.0 {
            return "0".to_string();
        }
        let ax = x.abs();
        let mut s = if (0.001..1000.0).contains(&ax) {
            format!("{x:.3}")
        } else {
            format!("{x:.3e}")
        };
        if let Some(dot) = s.find('.') {
            let exp = s.find('e').unwrap_or(s.len());
            let mut mant = s[..exp].to_string();
            while mant.ends_with('0') {
                mant.pop();
            }
            if mant.ends_with('.') {
                mant.pop();
            }
            s = if exp < s.len() {
                format!("{}{}", mant, &s[exp..])
            } else {
                mant
            };
            let _ = dot;
        }
        s
    };

    for perturb_seed in 0..6 {
        let delta = 0.05_f32 * perturb_seed as f32;
        for pct_id in [90_u32, 80, 70, 62] {
            for n in 0..4 {
                let mut ap = M3AlnParams::default();
                m3_aln_params_set_blosum(
                    &mut ap,
                    pct_id,
                    n,
                    f32::MAX,
                    f32::MAX,
                    perturb_seed,
                    delta,
                    delta,
                    delta,
                );
                bench_run(&mut bench, &ap, false, &mut run_case);

                let delta_s = format_g3(delta);
                log.push_str(&format!(
                    "BLOSUM{pct_id}:{n}  perturb={perturb_seed} delta={delta_s:>7} AvgQ={:.4} AvgTC={:.4} N={msa_count}\n",
                    bench.mean_q, bench.mean_tc
                ));
                if !tsv_out_file_name.is_empty() {
                    tsv.push_str(&format!(
                        "{pct_id}\t{n}\t{:.5}\t{:.5}\t{perturb_seed}\t{delta_s}\n",
                        bench.mean_q, bench.mean_tc
                    ));
                }
            }
        }
    }

    let tsv_written = if tsv_out_file_name.is_empty() {
        None
    } else {
        std::fs::write(tsv_out_file_name, &tsv).expect("failed to write BLOSUM bench TSV");
        Some(tsv)
    };
    (bench, log, tsv_written)
}
