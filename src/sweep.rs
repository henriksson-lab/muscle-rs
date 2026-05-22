// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Evaluate one sweep parameter vector: configure aln params, run the
/// benchmark, log Q/TC, and update the running best score.
#[track_caller]
pub fn sweeper_get_score_l12<FRunBench>(
    s: &Sweeper,
    param_values: &[f32],
    subst_mx_letter: &[[f32; 20]; 20],
    top_score: &mut f64,
    top_q: &mut f64,
    top_tc: &mut f64,
    tree_iters: Option<uint>,
    mut run_bench: FRunBench,
) -> (M3AlnParams, f64, f64, String)
where
    FRunBench: FnMut(&M3AlnParams) -> (f64, f64),
{
    assert_eq!(param_values.len() as uint, s.param_count);
    let mut gap_open = f32::MAX;
    let mut center = f32::MAX;
    let mut out = String::new();
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
    m3_aln_params_update_mx(&mut ap, subst_mx_letter, gap_open, center);
    if let Some(tree_iters) = tree_iters {
        ap.tree_iters = tree_iters;
    }

    let (q, tc) = run_bench(&ap);
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
    let score = tc;
    if score > *top_score {
        *top_score = score;
        *top_q = q;
        *top_tc = tc;
        out.push_str(" <<\n");
    } else {
        out.push_str("      \r");
    }
    (ap, q, tc, out)
}

/// Parse `name,good,lo,hi,size/...` grid-spec strings into parallel
/// vectors of names, anchors, bounds, and grid sizes.
pub fn parse_grid_spec(spec: &str) -> (Vec<String>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<uint>) {
    let mut names = Vec::new();
    let mut goods = Vec::new();
    let mut los = Vec::new();
    let mut his = Vec::new();
    let mut sizes = Vec::new();

    let fields = split(spec, '/');
    let mut do_goods = true;
    for (i, spec1) in fields.iter().enumerate() {
        let fields2 = split(spec1, ',');
        if fields2.len() != 5 {
            panic!("Bad spec1='{}'", spec1);
        }

        let name = fields2[0].clone();
        if i == 0 && fields2[1] == "-" {
            do_goods = false;
        }
        if do_goods {
            let good = str_to_float_l1209(&fields2[1], false) as f32;
            goods.push(good);
        }

        let lo = str_to_float_l1209(&fields2[2], false) as f32;
        let hi = str_to_float_l1209(&fields2[3], false) as f32;
        let size = str_to_uint_l1278(&fields2[4], false);
        assert!(size > 1);
        assert!(lo != hi);
        if lo < hi {
            los.push(lo);
            his.push(hi);
        } else {
            los.push(hi);
            his.push(lo);
        }

        names.push(name);
        sizes.push(size);
    }

    (names, goods, los, his, sizes)
}

/// CLI entry point for `sweep`: run a grid (and optional anchor) search
/// over alignment parameters against a benchmark.
#[track_caller]
pub fn cmd_sweep<FRunCase>(
    input_file_name: &str,
    ref_dir: &str,
    grid_spec: &str,
    fev_file_name: &str,
    blosum_pct: Option<uint>,
    subst_mx_file_name: Option<&str>,
    tree_iters: Option<uint>,
    mut run_case: FRunCase,
) -> (Sweeper, Bench, String, String)
where
    FRunCase: FnMut(&M3AlnParams, &str, &MultiSequence, &MultiSequence, bool) -> (f64, f64),
{
    let (names, goods, los, his, sizes) = parse_grid_spec(grid_spec);

    let subst_mx_letter = if let Some(pct) = blosum_pct {
        get_subst_mx_letter_blosum(pct)
    } else if let Some(file_name) = subst_mx_file_name {
        read_subst_mx_letter_from_file(file_name)
    } else {
        get_subst_mx_letter_blosum(62)
    };

    let mut ref_dir = ref_dir.to_string();
    dirize(&mut ref_dir);
    let mut bench = Bench::default();
    bench_load(&mut bench, input_file_name, &ref_dir);

    let mut s = Sweeper {
        grid_counter: uint::MAX,
        grid_count: uint::MAX,
        spatter_iter: uint::MAX,
        ..Sweeper::default()
    };
    sweeper_set_param_names(&mut s, &names);

    let mut top_score = 0.0_f64;
    let mut top_q = 0.0_f64;
    let mut top_tc = 0.0_f64;
    let mut log = String::new();
    let mut fev = String::new();

    if !goods.is_empty() {
        let (ap, q, tc, score_log) = sweeper_get_score_l12(
            &s,
            &goods,
            &subst_mx_letter,
            &mut top_score,
            &mut top_q,
            &mut top_tc,
            tree_iters,
            |ap| {
                bench_run(&mut bench, ap, false, &mut run_case);
                (bench.mean_q, bench.mean_tc)
            },
        );
        let _ = ap;
        log.push_str(&score_log);
        fev.push_str(&sweeper_run1(&mut s, &goods, q, tc));
    }

    let grid_fev = {
        let bench_ptr = &mut bench;
        let run_case_ref = &mut run_case;
        let log_ref = &mut log;
        sweeper_explore_grid(&mut s, &los, &his, &sizes, |s, values| {
            let (_ap, q, tc, score_log) = sweeper_get_score_l12(
                s,
                values,
                &subst_mx_letter,
                &mut top_score,
                &mut top_q,
                &mut top_tc,
                tree_iters,
                |ap| {
                    bench_run(bench_ptr, ap, false, &mut *run_case_ref);
                    (bench_ptr.mean_q, bench_ptr.mean_tc)
                },
            );
            log_ref.push_str(&score_log);
            (q, tc)
        })
    };
    fev.push_str(&grid_fev);
    log.push_str(&sweeper_log_top(&s, 10));

    if !fev_file_name.is_empty() {
        std::fs::write(fev_file_name, &fev).expect("failed to write sweep FEV");
    }
    (s, bench, log, fev)
}
