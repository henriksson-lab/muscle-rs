// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug)]
pub struct ThreadData {
    pub input: Option<MultiSequence>,
    pub dist_mx: Vec<Vec<f32>>,
    pub pair_index: uint,
    pub idx1: uint,
    pub idx2: uint,
    pub open: f32,
    pub ext: f32,
    pub lock: Option<std::sync::Arc<std::sync::Mutex<()>>>,
    pub log: String,
} // original: ThreadData (muscle/src/swdistmx.cpp)

impl Default for ThreadData {
    fn default() -> Self {
        Self {
            input: None,
            dist_mx: Vec::new(),
            pair_index: uint::MAX,
            idx1: uint::MAX,
            idx2: uint::MAX,
            open: f32::MAX,
            ext: f32::MAX,
            lock: None,
            log: String::new(),
        }
    }
}

/// Worker loop that pulls pairs `(i,j)` from the shared queue, runs
/// Smith-Waterman BLOSUM62, and fills the distance matrix in `td`.
#[track_caller]
pub fn thread_body(thread_index: uint, td: &mut ThreadData) -> String {
    let _ = thread_index;
    let input = td.input.as_ref().expect("ThreadData Input != 0");
    let seq_count = input.seqs.len() as uint;
    let pair_count = (seq_count * (seq_count - 1)) / 2;
    let open = td.open;
    let ext = td.ext;
    let mut mem = XDPMem::default();
    let mut log = String::new();
    loop {
        let done = td.pair_index == pair_count;
        let idx1 = td.idx1;
        let idx2 = td.idx2;
        if !done {
            let _ = progress_step(td.pair_index, pair_count, "Aligning");
            td.pair_index += 1;
            assert!(idx1 < seq_count);
            assert!(idx2 < seq_count);
            assert_ne!(idx1, idx2);
            td.idx2 += 1;
            if td.idx2 == seq_count {
                td.idx1 += 1;
                if td.idx1 != seq_count {
                    td.idx2 = td.idx1 + 1;
                }
            }
        }
        if done {
            return log;
        }
        assert!(idx1 < seq_count);
        assert!(idx2 < seq_count);

        let seq_i = &input.seqs[idx1 as usize];
        let seq_j = &input.seqs[idx2 as usize];
        let li = seq_i.char_vec.len() as uint;
        let lj = seq_j.char_vec.len() as uint;
        let label_i = &seq_i.label;
        let label_j = &seq_j.label;

        let (sw_score, _loi, _loj, _leni, _lenj, path) =
            sw_fast_seqs_blosum62(&mut mem, seq_i, seq_j, open, ext);
        let mean_length = (li + lj) as f32 / 2.0;
        let norm_score = sw_score / mean_length;

        td.dist_mx[idx1 as usize][idx2 as usize] = norm_score;
        td.dist_mx[idx2 as usize][idx1 as usize] = norm_score;
        let line = format!(
            "{:>10.10}  {:>10.10}  {:10.1}  {:7.4}  {path}\n",
            label_i, label_j, sw_score, norm_score
        );
        crate::log(&line);
        log.push_str(&line);
    }
}

#[track_caller]
fn thread_body_shared(_thread_index: uint, shared_td: &std::sync::Mutex<ThreadData>) {
    let (input, seq_count, pair_count, open, ext) = {
        let td = shared_td.lock().unwrap();
        let input = td.input.as_ref().expect("ThreadData Input != 0").clone();
        let seq_count = input.seqs.len() as uint;
        let pair_count = (seq_count * (seq_count - 1)) / 2;
        (input, seq_count, pair_count, td.open, td.ext)
    };

    let mut mem = XDPMem::default();
    loop {
        let (idx1, idx2) = {
            let mut td = shared_td.lock().unwrap();
            let done = td.pair_index == pair_count;
            let idx1 = td.idx1;
            let idx2 = td.idx2;
            if !done {
                let _ = progress_step(td.pair_index, pair_count, "Aligning");
                td.pair_index += 1;
                assert!(idx1 < seq_count);
                assert!(idx2 < seq_count);
                assert_ne!(idx1, idx2);
                td.idx2 += 1;
                if td.idx2 == seq_count {
                    td.idx1 += 1;
                    if td.idx1 != seq_count {
                        td.idx2 = td.idx1 + 1;
                    }
                }
            }
            if done {
                return;
            }
            (idx1, idx2)
        };

        assert!(idx1 < seq_count);
        assert!(idx2 < seq_count);

        let seq_i = &input.seqs[idx1 as usize];
        let seq_j = &input.seqs[idx2 as usize];
        let li = seq_i.char_vec.len() as uint;
        let lj = seq_j.char_vec.len() as uint;
        let label_i = &seq_i.label;
        let label_j = &seq_j.label;

        let (sw_score, _loi, _loj, _leni, _lenj, path) =
            sw_fast_seqs_blosum62(&mut mem, seq_i, seq_j, open, ext);
        let mean_length = (li + lj) as f32 / 2.0;
        let norm_score = sw_score / mean_length;

        let mut td = shared_td.lock().unwrap();
        td.dist_mx[idx1 as usize][idx2 as usize] = norm_score;
        td.dist_mx[idx2 as usize][idx1 as usize] = norm_score;
        let line = format!(
            "{:>10.10}  {:>10.10}  {:10.1}  {:7.4}  {path}\n",
            label_i, label_j, sw_score, norm_score
        );
        crate::log(&line);
        td.log.push_str(&line);
    }
}

/// Build a guide tree from `input` by running all-pairs Smith-Waterman with
/// BLOSUM62 and clustering the resulting distances with UPGMA.
#[track_caller]
pub fn calc_guide_tree_sw_blosum62(input: &MultiSequence, t: &mut Tree) -> (Vec<Vec<f32>>, String) {
    let seq_count = input.seqs.len() as uint;
    let mut labels = Vec::<String>::new();
    for i in 0..seq_count {
        labels.push(input.seqs[i as usize].label.clone());
    }

    let mut td = ThreadData {
        input: Some(input.clone()),
        dist_mx: vec![vec![f32::MAX; seq_count as usize]; seq_count as usize],
        idx1: 0,
        idx2: 1,
        open: -11.0,
        ext: -1.0,
        pair_index: 0,
        lock: Some(std::sync::Arc::new(std::sync::Mutex::new(()))),
        log: String::new(),
    };

    let thread_count = get_requested_thread_count().max(1);
    let log = if thread_count == 1 {
        thread_body(0, &mut td)
    } else {
        let shared_td = std::sync::Mutex::new(td);
        std::thread::scope(|scope| {
            for thread_index in 0..thread_count {
                let shared_td_ref = &shared_td;
                scope.spawn(move || thread_body_shared(thread_index, shared_td_ref));
            }
        });
        td = shared_td.into_inner().unwrap();
        std::mem::take(&mut td.log)
    };
    assert_eq!(td.idx1, seq_count - 1);
    assert_eq!(td.idx2, seq_count);
    let _ = progress("Done.\n");

    let mut u = UPGMA5::default();
    upgma5_init(&mut u, &labels, &td.dist_mx);
    // C++ swdistmx.cpp:125 calls `U.ScaleDistMx()` (no arg) which uses the
    // default `InputIsSimilarity = true`. SW distances are similarity-like
    // (higher = more similar after length normalisation), so this must match.
    upgma5_scale_dist_mx(&mut u, true);
    upgma5_run_l75(&mut u, "avg", t);
    (td.dist_mx, log)
}

/// CLI entry point: load sequences, build the SW-BLOSUM62 distance matrix
/// and guide tree, optionally writing the tree to `guide_tree_out_file_name`.
#[track_caller]
pub fn cmd_swdistmx(
    input_file_name: &str,
    guide_tree_out_file_name: &str,
) -> (Tree, Vec<Vec<f32>>) {
    let mut input = MultiSequence::default();
    multi_sequence_load_mfa_l8(&mut input, input_file_name, true);
    set_global_input_ms(&input);
    let mut t = Tree::default();
    let (dist_mx, _log) = calc_guide_tree_sw_blosum62(&input, &mut t);
    if !guide_tree_out_file_name.is_empty() {
        tree_to_file_l13(&t, guide_tree_out_file_name);
    }
    (t, dist_mx)
}
