// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct Bench {
    pub ap: Option<M3AlnParams>,
    pub ref_names: Vec<String>,
    pub refs: Vec<MultiSequence>,
    pub inputs: Vec<MultiSequence>,
    pub m3s: Vec<Muscle3>,
    pub qss: Vec<QScorer>,
    pub qs2s: Vec<QScorer2>,
    pub thread_count: uint,
    pub mean_q: f64,
    pub mean_tc: f64,
    pub final_score: f64,
    pub show_progress: bool,
    pub tcs: Vec<f64>,
} // original: Bench (muscle/src/bench.h)

/// Allocates per-thread Muscle3 and QScorer slots for the benchmark.
#[track_caller]
pub fn bench_alloc_threads(b: &mut Bench, q2: bool) {
    assert!(b.ap.is_some());
    if !b.m3s.is_empty() {
        assert_eq!(b.m3s.len(), b.thread_count as usize);
        if q2 {
            assert_eq!(b.qs2s.len(), b.thread_count as usize);
        } else {
            assert_eq!(b.qss.len(), b.thread_count as usize);
        }
        return;
    }
    assert!(b.ap.as_ref().unwrap().ready);
    for _ in 0..b.thread_count {
        b.m3s.push(Muscle3::default());
        if q2 {
            b.qs2s.push(QScorer2);
        } else {
            b.qss.push(QScorer::default());
        }
    }
}

/// Loads Q2-mode benchmark inputs: a list of reference names with separate input/ref directories.
#[track_caller]
pub fn bench_load_q2(b: &mut Bench, file_name: &str, fa_dir: &str, ref_dir: &str) {
    b.ref_names = read_strings_from_file(file_name);

    let ref_count = b.ref_names.len();
    for ref_index in 0..ref_count {
        let ref_name = &b.ref_names[ref_index];
        let ref_file_name = format!("{ref_dir}{ref_name}");

        let mut ref_msa = MultiSequence {
            dupe_labels_ok: true,
            ..MultiSequence::default()
        };
        multi_sequence_load_mfa_l8(&mut ref_msa, &ref_file_name, false);
        b.refs.push(ref_msa);
    }

    for ref_index in 0..ref_count {
        let ref_name = &b.ref_names[ref_index];
        let input_file_name = format!("{fa_dir}{ref_name}");
        let mut ms = MultiSequence::default();
        multi_sequence_load_mfa_l8(&mut ms, &input_file_name, true);
        b.inputs.push(ms);
    }
}

/// Loads benchmark MSAs (reference and ungapped input) from a list of names in `file_name`.
#[track_caller]
pub fn bench_load(b: &mut Bench, file_name: &str, ref_dir: &str) {
    b.ref_names = read_strings_from_file(file_name);

    let msa_count = b.ref_names.len();
    for msa_index in 0..msa_count {
        let ref_name = &b.ref_names[msa_index];
        let ref_file_name = format!("{ref_dir}{ref_name}");

        let mut ref_msa = MultiSequence::default();
        multi_sequence_load_mfa_l8(&mut ref_msa, &ref_file_name, false);
        b.refs.push(ref_msa);

        let mut ms = MultiSequence::default();
        multi_sequence_load_mfa_l8(&mut ms, &ref_file_name, true);
        b.inputs.push(ms);
    }
}

/// Copies reference names and MSAs from `b` into the empty `bench`.
#[track_caller]
pub fn bench_copy(bench: &mut Bench, b: &Bench) {
    assert!(bench.ref_names.is_empty());
    assert!(bench.refs.is_empty());
    assert!(bench.inputs.is_empty());
    let n = b.ref_names.len();
    for i in 0..n {
        bench.ref_names.push(b.ref_names[i].clone());
        bench.refs.push(b.refs[i].clone());
        bench.inputs.push(b.inputs[i].clone());
    }
}

/// Builds `bench` from a shuffled `pct`% sample of the MSAs in `b`.
#[track_caller]
pub fn bench_from_sample(bench: &mut Bench, b: &Bench, pct: uint) {
    assert!(bench.ref_names.is_empty());
    assert!(bench.refs.is_empty());
    assert!(bench.inputs.is_empty());
    let msa_count = b.ref_names.len() as uint;
    assert!(msa_count > 0);
    assert_eq!(b.refs.len() as uint, msa_count);
    assert_eq!(b.inputs.len() as uint, msa_count);
    let mut n = (msa_count * pct) / 100;
    if n == 0 {
        n = 1;
    }
    assert!(n <= msa_count);
    let mut order = Vec::new();
    for i in 0..n {
        order.push(i);
    }
    shuffle(&mut order);
    for k in order.iter().take(n as usize) {
        let k = *k as usize;
        bench.ref_names.push(b.ref_names[k].clone());
        bench.refs.push(b.refs[k].clone());
        bench.inputs.push(b.inputs[k].clone());
    }
}

/// Runs the benchmark for every loaded MSA, accumulating mean Q/TC and returning the final score.
#[track_caller]
pub fn bench_run<FRunCase>(
    bench: &mut Bench,
    ap: &M3AlnParams,
    q2: bool,
    mut run_case: FRunCase,
) -> f64
where
    FRunCase: FnMut(&M3AlnParams, &str, &MultiSequence, &MultiSequence, bool) -> (f64, f64),
{
    bench.ap = Some(ap.clone());
    bench.final_score = f64::MAX;
    if bench.thread_count == 0 {
        bench.thread_count = get_requested_thread_count();
    }
    bench_alloc_threads(bench, q2);
    bench.tcs.clear();

    let msa_count = bench.inputs.len();
    assert!(msa_count > 0);
    assert_eq!(bench.refs.len(), msa_count);
    bench.tcs.resize(msa_count, f64::MAX);

    let mut counter = 0usize;
    let mut sum_q = 0.0;
    let mut sum_tc = 0.0;
    for msa_index in 0..msa_count {
        let ref_name = bench.ref_names[msa_index].clone();
        let input = &bench.inputs[msa_index];
        let ref_msa = &bench.refs[msa_index];
        let (q, tc) = run_case(ap, &ref_name, input, ref_msa, q2);

        bench.tcs[msa_index] = tc;
        counter += 1;
        sum_q += q;
        sum_tc += tc;
        bench.mean_q = sum_q / counter as f64;
        bench.mean_tc = sum_tc / counter as f64;
    }

    bench.mean_q = sum_q / msa_count as f64;
    bench.mean_tc = sum_tc / msa_count as f64;
    bench.final_score = bench.mean_q + bench.mean_tc;
    bench.final_score
}

/// Writes per-MSA TC scores to a tab-separated file; returns the file contents.
#[track_caller]
pub fn bench_t_cs_to_file(bench: &Bench, file_name: &str) -> Option<String> {
    if file_name.is_empty() {
        return None;
    }
    let msa_count = bench.ref_names.len();
    assert_eq!(bench.tcs.len(), msa_count);
    let mut out = String::new();
    for i in 0..msa_count {
        let ref_name = &bench.ref_names[i];
        out.push_str(&format!("{ref_name}\t{:.4}\n", bench.tcs[i]));
    }
    std::fs::write(file_name, &out).expect("failed to write Bench TCs file");
    Some(out)
}
