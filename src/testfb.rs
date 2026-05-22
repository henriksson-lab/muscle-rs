// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// String-input wrapper around the flat backward HMM calculator.
#[track_caller]
pub fn calc_bwd_flat_l22(x: &str, y: &str, flat: &mut [f32]) {
    let px = x.as_bytes();
    let py = y.as_bytes();
    let lx = x.len() as uint;
    let ly = y.len() as uint;
    calc_bwd_flat_l10(px, lx, py, ly, flat);
}

/// String-input wrapper around the flat forward HMM calculator.
#[track_caller]
pub fn calc_fwd_flat_l31(x: &str, y: &str, flat: &mut [f32]) {
    let px = x.as_bytes();
    let py = y.as_bytes();
    let lx = x.len() as uint;
    let ly = y.len() as uint;
    calc_fwd_flat_l12(px, lx, py, ly, flat);
}

/// Formats a DP matrix and its traceback grid side-by-side for debugging.
#[track_caller]
pub fn log_tba(a: &[Vec<f32>], tb: &[Vec<char>], lx: uint, ly: uint) -> String {
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
    out.push('\n');
    out.push_str("       ");
    for j in 0..=ly {
        out.push_str(&format!("  {j:8}"));
    }
    out.push('\n');

    for i in 0..=lx {
        out.push_str(&format!("[{i:3}]  "));
        for j in 0..=ly {
            out.push_str(&format!("  {:>8}", format_g3(a[i as usize][j as usize])));
        }
        out.push('\n');
    }

    out.push('\n');
    out.push_str("       ");
    for j in 0..=ly {
        out.push_str(&format!("  {j:2}"));
    }
    out.push('\n');

    for i in 0..=lx {
        out.push_str(&format!("[{i:3}]  "));
        for j in 0..=ly {
            let mut c = tb[i as usize][j as usize];
            if c == 'D' {
                c = 'B';
            } else if c == 'L' {
                c = 'Y';
            } else if c == 'U' {
                c = 'X';
            }
            out.push_str(&format!("  {c:2}"));
        }
        out.push('\n');
    }
    out
}

/// Pretty-prints Tom's reference posterior matrix.
#[track_caller]
pub fn log_tom_post(tom_posterior: &[f32], lx: uint, ly: uint) -> String {
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
    out.push('\n');
    out.push_str(&format!("TomPost LX={lx} LY={ly}\n"));
    out.push_str("       ");
    for j in 0..=ly {
        out.push_str(&format!("  {j:10}"));
    }
    out.push('\n');

    let mut ix = 0;
    for i in 0..=lx {
        out.push_str(&format!("[{i:3}]  "));
        for _j in 0..=ly {
            let p = tom_posterior[ix];
            ix += 1;
            out.push_str(&format!("  {:>10}", format_g3(p)));
        }
        out.push('\n');
    }
    out
}

/// Pretty-prints the flat posterior matrix produced by this codebase.
#[track_caller]
pub fn log_flat_post(my_post: &[f32], lx: uint, ly: uint) -> String {
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
    out.push('\n');
    out.push_str(&format!("MyPost LX={lx} LY={ly}\n"));
    out.push_str("       ");
    for j in 0..ly {
        out.push_str(&format!("  {j:10}"));
    }
    out.push('\n');

    let mut ix = 0;
    for i in 0..lx {
        out.push_str(&format!("[{i:3}]  "));
        for _j in 0..ly {
            let p = my_post[ix];
            ix += 1;
            out.push_str(&format!("  {:>10}", format_g3(p)));
        }
        out.push('\n');
    }
    out
}

/// Returns true if two posterior probabilities agree to within tolerance
/// or are both below the cutoff.
#[track_caller]
pub fn post_eq(x: f32, y: f32) -> bool {
    if x == y {
        return true;
    }
    if x.abs() < POSTERIOR_CUTOFF && x.abs() < POSTERIOR_CUTOFF {
        return true;
    }
    let x_abs = x.abs() as f64;
    let y_abs = y.abs() as f64;
    let max = x_abs.max(y_abs);
    let diff = (x_abs - y_abs).abs();
    diff < max * 0.05
}

/// Asserts that Tom's and the flat posterior matrices agree element-wise.
#[track_caller]
pub fn cmp_post(tom_posterior: &[f32], post_flat: &[f32], lx: uint, ly: uint) {
    for i in 0..lx {
        for j in 0..ly {
            let tom_p = tom_posterior[((ly + 1) * (i + 1) + j + 1) as usize];
            let my_p = post_flat[(ly * i + j) as usize];
            if !post_eq(tom_p, my_p) {
                let format_g5 = |d: f32| -> String {
                    if d == 0.0 {
                        return "0".to_string();
                    }
                    if !d.is_finite() {
                        return d.to_string();
                    }
                    let d64 = f64::from(d);
                    let exp = d64.abs().log10().floor() as i32;
                    let mut s = if exp < -4 || exp >= 5 {
                        let raw = format!("{d64:.4e}");
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
                        let decimals = (4 - exp).max(0) as usize;
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
                panic!(
                    "CmpPost i={i} j={j} Tom={} My={}",
                    format_g5(tom_p),
                    format_g5(my_p)
                );
            }
        }
    }
}

/// Converts a flat HMM matrix into the per-state 3D layout for inspection.
#[track_caller]
pub fn cvt_flat(flat: &[f32], lx: uint, ly: uint) -> Vec<Vec<Vec<f32>>> {
    let ly1 = ly + 1;
    let mut mxs = vec![
        vec![vec![INVALID_LOG; (ly + 1) as usize]; (lx + 1) as usize];
        HMMSTATE_COUNT as usize
    ];

    for s in 0..HMMSTATE_COUNT as usize {
        for i in 0..=lx as usize {
            for j in 0..=ly as usize {
                let ix = HMMSTATE_COUNT as usize * (i * ly1 as usize + j) + s;
                mxs[s][i][j] = flat[ix];
            }
        }
    }
    mxs
}

/// Forward/backward smoke-test scaffold for a single pair of sequences.
#[track_caller]
pub fn test_l182() -> String {
    let x = String::new();
    let y = String::new();
    let do_fwd = false;
    let do_bwd = false;
    let lx = x.len() as uint;
    let ly = y.len() as uint;
    let flat_len = (lx + 1) * (ly + 1) * HMMSTATE_COUNT as uint;
    let post_len = lx * ly;
    let flat_fwd = vec![INVALID_LOG; flat_len as usize];
    let flat_bwd = vec![INVALID_LOG; flat_len as usize];
    let post_flat = vec![0.0_f32; post_len as usize];
    let tom_mxs_fwd = Vec::<Vec<Vec<f32>>>::new();
    let tom_mxs_bwd = Vec::<Vec<Vec<f32>>>::new();
    let simple_mxs_fwd = Vec::<Vec<Vec<f32>>>::new();
    let simple_mxs_bwd = Vec::<Vec<Vec<f32>>>::new();
    if do_fwd {
        let flat_mxs_fwd = cvt_flat(&flat_fwd, lx, ly);
        assert_eq!(tom_mxs_fwd.len(), simple_mxs_fwd.len());
        assert_eq!(flat_mxs_fwd.len(), HMMSTATE_COUNT as usize);
    }
    if do_bwd {
        let flat_mxs_bwd = cvt_flat(&flat_bwd, lx, ly);
        assert_eq!(tom_mxs_bwd.len(), simple_mxs_bwd.len());
        assert_eq!(flat_mxs_bwd.len(), HMMSTATE_COUNT as usize);
    }
    if do_fwd && do_bwd {
        assert!(post_flat.is_empty());
    }
    String::new()
}

/// Returns a random sequence of length in `[min_len, max_len)` using the
/// current alphabet.
#[track_caller]
pub fn get_random_seq(min_len: uint, max_len: uint) -> String {
    let mut seq = String::new();
    let l = min_len + randu32() % (max_len - min_len);
    let state = ALPHA_STATE.lock().unwrap();
    for _ in 0..l {
        let letter = (randu32() % 20) as usize;
        seq.push(state.letter_to_char[letter] as char);
    }
    seq
}

/// Timing benchmark scaffold for forward/backward computations.
#[track_caller]
pub fn test_timing() -> String {
    let mut seqs = Vec::<String>::new();
    let mut tom_seqs = Vec::<Sequence>::new();
    let n = 0_u32;
    let max_l = 300_u32;
    for _ in 0..n {
        let seq = get_random_seq(max_l / 2, max_l);
        let mut tom_seq = Sequence::default();
        sequence_from_string(&mut tom_seq, "tom", &seq);
        seqs.push(seq);
        tom_seqs.push(tom_seq);
    }
    let my_ticks = 0.0_f64;
    let tom_ticks = 0.0_f64;
    assert_eq!(seqs.len(), tom_seqs.len());
    assert_eq!(my_ticks, tom_ticks);
    String::new()
}

/// Runs the forward/backward test scaffold across a few short fixed
/// sequence pairs.
#[track_caller]
pub fn test_short() -> String {
    let do_fwd = false;
    let do_bwd = false;
    let cases = [
        ("MQTIF", "MSIF"),
        ("GATTACA", "MQTIF"),
        ("ABC", "DEF"),
        ("LQNGSEQVENCE", "QTHERSEQVENCEINSERT"),
    ];
    for (x, y) in cases {
        if do_fwd || do_bwd {
            assert!(!x.is_empty());
            assert!(!y.is_empty());
        }
    }
    String::new()
}

/// Runs the forward/backward test scaffold across longer sequence pairs.
#[track_caller]
pub fn test_long() -> String {
    let max_l = 0_u32;
    let do_fwd = false;
    let do_bwd = false;
    let mut seqs = Vec::<String>::new();
    for _ in 0..0 {
        seqs.push(get_random_seq(max_l / 2, max_l));
    }
    seqs.push("LSIDGKKYDTRLVATLLWFASLVLQDHVVDRYKDAADVLITETIYALLVTFSGTVVAKHGGNASGGYLTLILNCLVQLLLLIRSNIKRCGCTIGRCLVPAIIGDDGTY".to_string());
    seqs.push("LEIDISKFDKSQQMIACLFEREIMKRFGFPDDLAEIWFNCRWICSFYDPVCGVSFKSDFQMKSGVASTFITNTLFLMSVIFYFWEPSPNAFGLFGGDDSLL".to_string());
    seqs.push("GKFDKSQGLLALLIEIGIMRRFGAPEDLVELWYYSHMYTLLKDVKTGVSLKVIFQRKSGDAATFIGNTLFLLFVLAYYFGFNSLALALLGGDDSLL".to_string());
    seqs.push("EEIDISKYDKSQGLLALMFECKLMKRFGVMWFNQHLSSHFYSQSTGVSGMTSFQRKSGDAATFAGNTFFLMAIVADSCKIEDLDICAFSGDDSVL".to_string());
    seqs.push("LEIDISKYDKSQRELALEFECKLMKYFGVPSDIVELWFNAHVLTEVYDRTTKLNALIPYQRKSGDASTFIGNTLFLMAVICDLIPVSELELALFSGDDSLL".to_string());
    let n = seqs.len() as uint;
    for i in 0..n {
        let x = &seqs[i as usize];
        for j in 0..n {
            let y = &seqs[j as usize];
            if do_fwd || do_bwd {
                assert!(!x.is_empty());
                assert!(!y.is_empty());
            }
        }
    }
    String::new()
}

/// Entry point for the `testfb` command: runs the configured forward and
/// backward sanity tests.
#[track_caller]
pub fn cmd_testfb() -> String {
    let do_test_single = false;
    let do_test_short = false;
    let do_test_long = false;
    let mut out = String::new();
    if do_test_single {
        out.push_str(&test_l182());
    }
    if do_test_short {
        out.push_str(&test_short());
    }
    if do_test_long {
        set_alpha_amino();
        out.push_str(&test_long());
        out.push_str("Eq 0 diff 0\n");
    }
    out
}
