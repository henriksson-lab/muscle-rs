// Generated translation scaffold. Regenerate from CCC output when needed.
#[allow(unused_imports)]
use crate::*;

/// Pick the top-confidence ensemble columns (subject to gap and count limits) and assemble them into a representative MSA.
#[track_caller]
pub fn cmd_efa_bestcols(
    efa_file_name: &str,
    output_file_name: &str,
    min_conf: f64,
    max_gap_fract: f64,
    max_cols: uint,
) -> MultiSequence {
    assert!((0.0..=1.0).contains(&max_gap_fract));

    let mut e = Ensemble::default();
    ensemble_from_file(&mut e, efa_file_name);

    let mut confs = Vec::<f64>::new();
    let unique_ix_count = e.unique_ix_to_ixs.len() as uint;
    let mut unique_ixs = Vec::<uint>::new();
    for unique_ix in 0..unique_ix_count {
        let conf = ensemble_get_conf(&e, unique_ix);
        if conf < min_conf {
            continue;
        }
        let ix = e.unique_ixs[unique_ix as usize];
        let gap_fract = ensemble_get_gap_fract(&e, ix);
        if gap_fract > max_gap_fract {
            continue;
        }

        unique_ixs.push(unique_ix);
        confs.push(conf);
    }

    let mut order: Vec<usize> = (0..confs.len()).collect();
    if !order.is_empty() {
        let mut stack = vec![(0_i32, order.len() as i32 - 1)];
        while let Some((left, right)) = stack.pop() {
            let mut i = left;
            let mut j = right;
            let mid = (left + right) / 2;
            let pivot = confs[order[mid as usize]];

            while i <= j {
                while confs[order[i as usize]] > pivot {
                    i += 1;
                }
                while confs[order[j as usize]] < pivot {
                    j -= 1;
                }
                if i <= j {
                    order.swap(i as usize, j as usize);
                    i += 1;
                    j -= 1;
                }
            }

            if i < right {
                stack.push((i, right));
            }
            if left < j {
                stack.push((left, j));
            }
        }
    }

    let mut best_unique_ixs = Vec::<uint>::new();
    let n = std::cmp::min(order.len(), max_cols as usize);
    for i in 0..n {
        let unique_ix = unique_ixs[order[i]];
        best_unique_ixs.push(unique_ix);
    }

    let rep_aln = ensemble_make_resampled_msa(&e, &best_unique_ixs);
    msa_to_fasta_file_l103(&rep_aln, output_file_name);
    rep_aln
}
