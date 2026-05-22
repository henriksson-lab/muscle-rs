// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

#[derive(Clone, Debug, Default)]
pub struct ThreadData {
    pub input: Option<MultiSequence>,
    pub dist_mx: Vec<Vec<f32>>,
    pub pair_index: uint,
    pub idx1: uint,
    pub idx2: uint,
    pub open: f32,
    pub ext: f32,
    pub lock: Option<std::sync::Arc<std::sync::Mutex<()>>>,
} // original: ThreadData (muscle/src/swdistmx.cpp)

/// Worker loop that pulls pairs `(i,j)` from the shared queue, runs
/// Smith-Waterman BLOSUM62, and fills the distance matrix in `td`.
#[track_caller]
pub fn thread_body(thread_index: uint, td: &mut ThreadData) -> String {
    let _ = thread_index;
    let input = td.input.as_ref().expect("ThreadData Input != 0").clone();
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
        log.push_str(&format!(
            "{:>10.10}  {:>10.10}  {:10.1}  {:7.4}  {path}\n",
            label_i, label_j, sw_score, norm_score
        ));
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
    };

    let log = thread_body(0, &mut td);
    assert_eq!(td.idx1, seq_count - 1);
    assert_eq!(td.idx2, seq_count);

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
