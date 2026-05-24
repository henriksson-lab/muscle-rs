// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Run the benchmark with the given parameter values and update top-score state, returning (Q, TC, log).
#[track_caller]
pub fn sweeper_get_score_l20<FRunBench>(
    s: &Sweeper,
    param_values: &[f32],
    subst_mx_letter: &[[f32; 20]; 20],
    ap: &mut M3AlnParams,
    top_score: &mut f64,
    top_q: &mut f64,
    top_tc: &mut f64,
    mut run_bench: FRunBench,
) -> (f64, f64, String)
where
    FRunBench: FnMut(&M3AlnParams) -> (f64, f64),
{
    assert_eq!(param_values.len() as uint, s.param_count);
    let mut gap_open = f32::MAX;
    let mut center = f32::MAX;
    let mut out = format!("{}  ", get_progress_prefix_c_str());
    let format_g4 = |d: f32| -> String {
        if d == 0.0 {
            return "0".to_string();
        }
        if !d.is_finite() {
            return d.to_string();
        }
        let exp = d.abs().log10().floor() as i32;
        let mut s = if exp < -4 || exp >= 4 {
            let raw = format!("{d:.3e}");
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
            let decimals = (3 - exp).max(0) as usize;
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
    for param_index in 0..s.param_count {
        let name = &s.param_names[param_index as usize];
        let value = param_values[param_index as usize];
        if name == "gapopen" {
            gap_open = value;
            out.push_str(&format!("gapopen={:>8}", format_g4(value)));
        } else if name == "center" {
            center = value;
            out.push_str(&format!(" center={:>8}", format_g4(value)));
        } else {
            die(&format!("SweeperGetScore bad param '{name}'"));
        }
    }

    m3_aln_params_update_mx(ap, subst_mx_letter, gap_open, center);
    let (q, tc) = run_bench(ap);
    out.push_str(&format!(
        "  Q={:+6.4}({:6.4}) TC={:+6.4}({:6.4})",
        q - *top_q,
        *top_q,
        tc - *top_tc,
        *top_tc
    ));
    if s.grid_counter != uint::MAX {
        let pct = 100.0 * f64::from(s.grid_counter) / f64::from(s.grid_count);
        out.push_str(&format!(" ({pct:.2}%)"));
    }
    if s.spatter_iter != uint::MAX {
        out.push_str(&format!(
            " {}[{}/{}]",
            s.spatter_iter,
            s.spatter_try + 1,
            s.spatter_tries_per_iter
        ));
    }
    let score = tc;
    if score > *top_score {
        *top_score = score;
        *top_q = q;
        *top_tc = tc;
        out.push_str(&format!(" TC={tc:.5} <<\n"));
    } else {
        out.push_str("      \r");
    }
    (q, tc, out)
}

/// Parse a "name,delta/name,delta/..." spatter spec into parallel name and delta vectors.
pub fn parse_spatter_spec(spec: &str) -> (Vec<String>, Vec<f32>) {
    let mut names = Vec::new();
    let mut deltas = Vec::new();

    let fields = split(spec, '/');
    for spec1 in fields {
        let fields2 = split(&spec1, ',');
        if fields2.len() != 2 {
            panic!("Bad spec1='{}'", spec1);
        }

        let name = fields2[0].clone();
        let delta = str_to_float_l1209(&fields2[1], false) as f32;
        names.push(name);
        deltas.push(delta);
    }

    (names, deltas)
}

/// `spatter` command: warm up on a sample bench via grid search, then optimize parameters with spatter search.
#[track_caller]
pub fn cmd_spatter<FRunCase>(
    input_file_name: &str,
    ref_dir: &str,
    warmup_pct: uint,
    grid_spec: &str,
    spatter_spec: &str,
    output1_file_name: &str,
    fev_file_name: &str,
    blosum_pct: Option<uint>,
    subst_mx_file_name: Option<&str>,
    max_iters: uint,
    max_fail_iters: uint,
    tries_per_iter: uint,
    shrink: f32,
    mut run_case: FRunCase,
) -> (Sweeper, Sweeper, Bench, Bench, String, String, String)
where
    FRunCase: FnMut(&M3AlnParams, &str, &MultiSequence, &MultiSequence, bool) -> (f64, f64),
{
    with_quiet(true, || {
        cmd_spatter_quiet(
            input_file_name,
            ref_dir,
            warmup_pct,
            grid_spec,
            spatter_spec,
            output1_file_name,
            fev_file_name,
            blosum_pct,
            subst_mx_file_name,
            max_iters,
            max_fail_iters,
            tries_per_iter,
            shrink,
            &mut run_case,
        )
    })
}

#[track_caller]
fn cmd_spatter_quiet<FRunCase>(
    input_file_name: &str,
    ref_dir: &str,
    warmup_pct: uint,
    grid_spec: &str,
    spatter_spec: &str,
    output1_file_name: &str,
    fev_file_name: &str,
    blosum_pct: Option<uint>,
    subst_mx_file_name: Option<&str>,
    max_iters: uint,
    max_fail_iters: uint,
    tries_per_iter: uint,
    shrink: f32,
    mut run_case: FRunCase,
) -> (Sweeper, Sweeper, Bench, Bench, String, String, String)
where
    FRunCase: FnMut(&M3AlnParams, &str, &MultiSequence, &MultiSequence, bool) -> (f64, f64),
{
    let mut ref_dir = ref_dir.to_string();
    dirize(&mut ref_dir);
    let mut bench = Bench::default();
    bench_load(&mut bench, input_file_name, &ref_dir);

    assert!(warmup_pct > 0 && warmup_pct <= 100);
    let subst_mx_letter = if let Some(pct) = blosum_pct {
        get_subst_mx_letter_blosum(pct)
    } else if let Some(file_name) = subst_mx_file_name {
        read_subst_mx_letter_from_file(file_name)
    } else {
        get_subst_mx_letter_blosum(62)
    };

    let mut warmup_bench = Bench::default();
    bench_from_sample(&mut warmup_bench, &bench, warmup_pct);

    let mut ap = M3AlnParams::default();
    let _ = m3_aln_params_set_from_cmd_line(
        &mut ap,
        false,
        false,
        None,
        None,
        None,
        Some(62),
        Some(0),
        None,
        None,
        None,
        None,
    );

    assert!(max_iters > 0);
    assert!(max_fail_iters > 0);
    assert!(tries_per_iter > 0);
    assert!(shrink > 0.0 && shrink < 1.0);

    let (names, goods, los, his, sizes) = parse_grid_spec(grid_spec);
    let (names2, start_max_deltas) = parse_spatter_spec(spatter_spec);
    assert_eq!(names2.len(), names.len());
    for i in 0..names2.len() {
        assert_eq!(names2[i], names[i]);
    }

    let mut top_score = 0.0_f64;
    let mut top_q = 0.0_f64;
    let mut top_tc = 0.0_f64;
    let mut log = String::new();
    let mut output1_fev = String::new();

    let mut s1 = Sweeper {
        grid_counter: uint::MAX,
        grid_count: uint::MAX,
        spatter_iter: uint::MAX,
        ..Sweeper::default()
    };
    sweeper_set_param_names(&mut s1, &names);
    if !output1_file_name.is_empty() {
        sweeper_set_fev(&mut s1, output1_file_name);
    }
    if !goods.is_empty() {
        let (q, tc, score_log) = sweeper_get_score_l20(
            &s1,
            &goods,
            &subst_mx_letter,
            &mut ap,
            &mut top_score,
            &mut top_q,
            &mut top_tc,
            |ap| {
                bench_run(&mut warmup_bench, ap, false, &mut run_case);
                (warmup_bench.mean_q, warmup_bench.mean_tc)
            },
        );
        log.push_str(&score_log);
        output1_fev.push_str(&sweeper_run1(&mut s1, &goods, q, tc));
    }
    s1.grid_noise_fract = 0.1;
    let grid_fev = {
        let warmup_bench_ptr = &mut warmup_bench;
        let run_case_ref = &mut run_case;
        let log_ref = &mut log;
        sweeper_explore_grid(&mut s1, &los, &his, &sizes, |s, values| {
            let (q, tc, score_log) = sweeper_get_score_l20(
                s,
                values,
                &subst_mx_letter,
                &mut ap,
                &mut top_score,
                &mut top_q,
                &mut top_tc,
                |ap| {
                    bench_run(warmup_bench_ptr, ap, false, &mut *run_case_ref);
                    (warmup_bench_ptr.mean_q, warmup_bench_ptr.mean_tc)
                },
            );
            log_ref.push_str(&score_log);
            (q, tc)
        })
    };
    output1_fev.push_str(&grid_fev);

    let warmup_indexes = sweeper_get_distinct_top_indexes(&s1, 8, 0.05, 1.0);
    assert!(!warmup_indexes.is_empty());
    let mut start_value_vec = Vec::<Vec<f32>>::new();
    for index in &warmup_indexes {
        start_value_vec.push(s1.param_values_vec[*index as usize].clone());
    }

    log.push_str("\nWarmup done\n");
    top_score = 0.0;
    let mut s2 = Sweeper {
        grid_counter: uint::MAX,
        grid_count: uint::MAX,
        spatter_iter: uint::MAX,
        ..Sweeper::default()
    };
    sweeper_set_param_names(&mut s2, &names);
    if !fev_file_name.is_empty() {
        sweeper_set_fev(&mut s2, fev_file_name);
    }
    let fev = {
        let bench_ptr = &mut bench;
        let run_case_ref = &mut run_case;
        let log_ref = &mut log;
        sweeper_explore_spatter(
            &mut s2,
            &start_value_vec,
            &start_max_deltas,
            tries_per_iter,
            max_iters,
            max_fail_iters,
            shrink,
            |s, values| {
                let (q, tc, score_log) = sweeper_get_score_l20(
                    s,
                    values,
                    &subst_mx_letter,
                    &mut ap,
                    &mut top_score,
                    &mut top_q,
                    &mut top_tc,
                    |ap| {
                        bench_run(bench_ptr, ap, false, &mut *run_case_ref);
                        (bench_ptr.mean_q, bench_ptr.mean_tc)
                    },
                );
                log_ref.push_str(&score_log);
                (q, tc)
            },
        )
    };
    log.push_str(&sweeper_log_top(&s2, 10));

    (s1, s2, bench, warmup_bench, log, output1_fev, fev)
}
